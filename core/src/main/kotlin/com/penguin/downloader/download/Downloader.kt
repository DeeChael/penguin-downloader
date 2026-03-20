package com.penguin.downloader.download

import com.penguin.downloader.model.DownloadOptions
import com.penguin.downloader.model.SongInfo
import com.penguin.downloader.provider.MusicProvider
import com.varabyte.kotter.foundation.collections.liveListOf
import com.varabyte.kotter.foundation.liveVarOf
import com.varabyte.kotter.foundation.runUntilSignal
import com.varabyte.kotter.foundation.session
import com.varabyte.kotter.foundation.text.textLine
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import okhttp3.OkHttpClient
import okhttp3.Request
import org.jaudiotagger.audio.AudioFileIO
import org.jaudiotagger.tag.FieldKey
import org.jaudiotagger.tag.images.StandardArtwork
import java.io.File
import java.io.FileOutputStream
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Semaphore
import java.util.concurrent.atomic.AtomicInteger
import kotlin.collections.plus
import kotlin.math.abs

private data class ProgressUpdate(
    val taskId: Int,
    val line: String,
    val completed: Boolean
)

/**
 * 音乐下载器
 */
class Downloader(
    private val provider: MusicProvider,
    private val httpClient: OkHttpClient = OkHttpClient(),
    private val baseOutputDir: File = File("."),
    private val notifier: DownloadNotifier = StdoutDownloadNotifier
) {

    private var totalCount = 0
    private val successCount = AtomicInteger(0)
    private val failCount = AtomicInteger(0)

    private fun formatSize(bytes: Long): String {
        return when {
            bytes < 1024 -> "$bytes B"
            bytes < 1024 * 1024 -> String.format("%.1f KB", bytes / 1024.0)
            bytes < 1024 * 1024 * 1024 -> String.format("%.1f MB", bytes / (1024.0 * 1024))
            else -> String.format("%.2f GB", bytes / (1024.0 * 1024 * 1024)) // 不会有人的音乐会大于 1T 吧。。。
        }
    }

    /**
     * 下载单曲
     *
     * @param songInfo 音乐信息
     * @param options 下载参数
     */
    suspend fun downloadSong(songInfo: SongInfo, options: DownloadOptions): File? {
        val outputDir = File(baseOutputDir, "songs")
        val format = options.format ?: "{title} - {artist}" // 默认格式：<歌曲名> - <艺术家>
        val result = downloadInternal(songInfo, outputDir, format, 0, options)
        if (result != null) {
            successCount.incrementAndGet()
        } else {
            failCount.incrementAndGet()
        }
        return result
    }

    /**
     * 内部下载函数
     */
    private suspend fun downloadInternal(
        songInfo: SongInfo,
        outputDir: File,
        format: String,
        trackNumber: Int,
        options: DownloadOptions
    ): File? {
        val extensions = listOf("flac", "mflac", "mp3", "m4a", "ogg", "mgg")

        if (!options.force) {
            for (ext in extensions) {
                val fileName = formatFileName(format, songInfo, trackNumber, ext)
                val existingFile = File(outputDir, fileName)
                if (existingFile.exists()) {
                    if (!options.checkSize) {
                        val size = formatSize(existingFile.length())
                        notifier.info("已存在: ${songInfo.title} [$ext] ($size)")
                        return existingFile
                    }
                    // 不在这里检查文件大小是因为需要先从服务器获取到文件大小才能判断
                }
            }
        }

        // 从音乐源获取音乐的实际下载链接
        val urlResult = provider.getSongUrl(songInfo.mid, options.quality)

        if (urlResult == null) {
            notifier.error("获取链接失败: ${songInfo.title} - 未知错误")
            return null
        }

        if (!urlResult.isSuccess()) {
            val errorMsg = urlResult.errorMessage ?: "未知错误"
            notifier.error("获取链接失败: ${songInfo.title} - $errorMsg")
            return null
        }

        val url = urlResult.url!!

        val extension = when {
            url.contains(".flac", ignoreCase = true) -> "flac"
            url.contains(".m4a", ignoreCase = true) -> "m4a"
            url.contains(".ogg", ignoreCase = true) -> "ogg"
            url.contains(".mgg", ignoreCase = true) -> "mgg"
            else -> "mp3"
        }

        val fileName = formatFileName(format, songInfo, trackNumber, extension)

        if (!outputDir.exists()) {
            outputDir.mkdirs()
        }

        val outputFile = File(outputDir, fileName)

        if (!options.force && options.checkSize && outputFile.exists()) {
            val expectedSize = getRemoteFileSize(url)
            val actualSize = outputFile.length()
            val tolerance = 20 * 1024L
            
            if (expectedSize != null && abs(actualSize - expectedSize) <= tolerance) {
                val size = formatSize(actualSize)
                notifier.info("已存在: ${songInfo.title} [$extension] ($size)")
                return outputFile
            }
            
            notifier.info("文件大小不匹配，重新下载: ${songInfo.title}")
        }

        val qualityName = urlResult.quality ?: "未知音质"
        try {
            downloadFileWithProgress(url, outputFile, songInfo.title, qualityName)

            val coverData = songInfo.cover?.let { downloadImage(it) }

            embedMetadata(outputFile, coverData, songInfo, trackNumber)
        } catch (_: Exception) {
            return null
        }

        if (options.downloadLyrics) {
            downloadLyric(songInfo, outputDir, fileName.removeSuffix(".$extension"))
        }

        return outputFile
    }
    
    private suspend fun downloadFileSilent(
        songInfo: SongInfo,
        outputDir: File,
        format: String,
        trackNumber: Int,
        options: DownloadOptions,
        progressChannel: Channel<ProgressUpdate>,
        taskId: Int
    ): File? {
        val extensions = listOf("flac", "mflac", "mp3", "m4a", "ogg", "mgg")

        if (!options.force) {
            for (ext in extensions) {
                val fileName = formatFileName(format, songInfo, trackNumber, ext)
                val existingFile = File(outputDir, fileName)
                if (existingFile.exists() && !options.checkSize) {
                    val size = formatSize(existingFile.length())
                    progressChannel.send(ProgressUpdate(taskId, "已存在: ${songInfo.title} [$ext] ($size)", true))
                    return existingFile
                }
            }
        }

        val urlResult = provider.getSongUrl(songInfo.mid, options.quality)

        if (urlResult == null) {
            progressChannel.send(ProgressUpdate(taskId, "获取链接失败: ${songInfo.title}", true))
            return null
        }

        if (!urlResult.isSuccess()) {
            val errorMsg = urlResult.errorMessage ?: "未知错误"
            progressChannel.send(ProgressUpdate(taskId, "获取链接失败: ${songInfo.title} - $errorMsg", true))
            return null
        }

        val url = urlResult.url!!
        val extension = when {
            url.contains(".flac", ignoreCase = true) -> "flac"
            url.contains(".mflac", ignoreCase = true) -> "mflac"
            url.contains(".mp3", ignoreCase = true) -> "mp3"
            url.contains(".m4a", ignoreCase = true) -> "m4a"
            url.contains(".ogg", ignoreCase = true) -> "ogg"
            url.contains(".mgg", ignoreCase = true) -> "mgg"
            else -> "mp3"
        }

        val fileName = formatFileName(format, songInfo, trackNumber, extension)

        if (!outputDir.exists()) {
            outputDir.mkdirs()
        }

        val outputFile = File(outputDir, fileName)

        if (!options.force && options.checkSize && outputFile.exists()) {
            val expectedSize = getRemoteFileSize(url)
            val actualSize = outputFile.length()
            val tolerance = 20 * 1024L
            
            if (expectedSize != null && abs(actualSize - expectedSize) <= tolerance) {
                val size = formatSize(actualSize)
                progressChannel.send(ProgressUpdate(taskId, "已存在: ${songInfo.title} [$extension] ($size)", true))
                return outputFile
            }
        }

        val qualityName = urlResult.quality ?: "未知音质"
        try {
            downloadFileWithProgressChannel(url, outputFile, songInfo.title, qualityName, progressChannel, taskId)

            val coverData = songInfo.cover?.let { downloadImage(it) }

            embedMetadata(outputFile, coverData, songInfo, trackNumber)

            val size = formatSize(outputFile.length())
            progressChannel.send(ProgressUpdate(taskId, "已下载: ${songInfo.title} [$qualityName] ($size)", true))
        } catch (e: Exception) {
            progressChannel.send(ProgressUpdate(taskId, "下载失败: ${songInfo.title} - ${e.message}", true))
            return null
        }

        if (options.downloadLyrics) {
            downloadLyric(songInfo, outputDir, fileName.removeSuffix(".$extension"))
        }

        return outputFile
    }

    private suspend fun getRemoteFileSize(url: String): Long? {
        return withContext(Dispatchers.IO) {
            try {
                val request = Request.Builder()
                    .url(url)
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                    .header("Referer", "https://music.163.com/")
                    .head()
                    .build()
                val response = httpClient.newCall(request).execute()
                if (response.isSuccessful) {
                    response.body?.contentLength()
                } else {
                    null
                }
            } catch (e: Exception) {
                null
            }
        }
    }

    private suspend fun downloadFileWithProgress(url: String, outputFile: File, title: String, qualityName: String) {
        withContext(Dispatchers.IO) {
            val request = Request.Builder()
                .url(url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Referer", "https://music.163.com/")
                .build()
            val response = httpClient.newCall(request).execute()

            if (!response.isSuccessful) {
                notifier.error("下载失败: HTTP ${response.code}")
                throw Exception("HTTP ${response.code}")
            }

            val body = response.body ?: throw Exception("空响应体")
            val totalBytes = body.contentLength()
            val totalSize = if (totalBytes > 0) formatSize(totalBytes) else "未知"

            var downloadedBytes = 0L
            var lastPrintTime = 0L

            outputFile.outputStream().use { output ->
                body.byteStream().use { input ->
                    val buffer = ByteArray(8192)
                    var bytesRead: Int

                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        output.write(buffer, 0, bytesRead)
                        downloadedBytes += bytesRead

                        val now = System.currentTimeMillis()
                        if (now - lastPrintTime >= 100) {
                            val downloadedSize = formatSize(downloadedBytes)
                            if (totalBytes > 0) {
                                val percent = (downloadedBytes * 100 / totalBytes).toInt()
                                notifier.progress("正在下载: $title [$qualityName] ($downloadedSize/$totalSize) $percent%", overwrite = true)
                            } else {
                                notifier.progress("正在下载: $title [$qualityName] ($downloadedSize/$totalSize)", overwrite = true)
                            }
                            lastPrintTime = now
                        }
                    }
                }
            }
            notifier.progress("正在下载: $title [$qualityName] ($totalSize/$totalSize) 100%", overwrite = true)
        }
    }
    
    private suspend fun downloadFileWithProgressChannel(
        url: String, 
        outputFile: File, 
        title: String, 
        qualityName: String,
        progressChannel: Channel<ProgressUpdate>,
        taskId: Int
    ) {
        withContext(Dispatchers.IO) {
            val request = Request.Builder()
                .url(url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Referer", "https://music.163.com/")
                .build()
            val response = httpClient.newCall(request).execute()
            
            if (!response.isSuccessful) {
                progressChannel.send(ProgressUpdate(taskId, "下载失败: HTTP ${response.code}", true))
                return@withContext
            }

            val body = response.body ?: throw Exception("空响应体")
            val totalBytes = body.contentLength()
            val totalSize = if (totalBytes > 0) formatSize(totalBytes) else "未知"

            var downloadedBytes = 0L
            var lastPrintTime = 0L

            outputFile.outputStream().use { output ->
                body.byteStream().use { input ->
                    val buffer = ByteArray(8192)
                    var bytesRead: Int

                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        output.write(buffer, 0, bytesRead)
                        downloadedBytes += bytesRead

                        val now = System.currentTimeMillis()
                        if (now - lastPrintTime >= 200) {
                            val downloadedSize = formatSize(downloadedBytes)
                            val line = if (totalBytes > 0) {
                                val percent = (downloadedBytes * 100 / totalBytes).toInt()
                                "正在下载: $title [$qualityName] ($downloadedSize/$totalSize) $percent%"
                            } else {
                                "正在下载: $title [$qualityName] ($downloadedSize/$totalSize)"
                            }
                            progressChannel.send(ProgressUpdate(taskId, line, false))
                            lastPrintTime = now
                        }
                    }
                }
            }
            progressChannel.send(ProgressUpdate(taskId, "正在下载: $title [$qualityName] ($totalSize/$totalSize) 100%", false))
        }
    }

    private suspend fun embedMetadata(
        audioFile: File,
        coverData: ByteArray?,
        info: SongInfo,
        trackNumber: Int
    ) {
        withContext(Dispatchers.IO) {
            try {
                val audioFileObj = AudioFileIO.read(audioFile)
                val tag = audioFileObj.tagOrCreateAndSetDefault

                tag.setField(FieldKey.TITLE, info.title)
                if (!info.subtitle.isNullOrBlank()) {
                    tag.setField(FieldKey.SUBTITLE, info.subtitle)
                }
                if (!info.artist.isNullOrBlank()) {
                    tag.setField(FieldKey.ARTIST, info.artist)
                }
                if (!info.album.isNullOrBlank()) {
                    tag.setField(FieldKey.ALBUM, info.album)
                }
                if (trackNumber > 0) {
                    tag.setField(FieldKey.TRACK, trackNumber.toString())
                }

                if (coverData != null) {
                    val artwork = StandardArtwork()
                    artwork.binaryData = coverData
                    artwork.mimeType = "image/jpeg"
                    artwork.pictureType = 3
                    tag.setField(artwork)
                }

                audioFileObj.commit()
            } catch (e: Exception) {
            }
        }
    }

    private suspend fun downloadImage(url: String): ByteArray? {
        return withContext(Dispatchers.IO) {
            try {
                val request = Request.Builder().url(url).build()
                val response = httpClient.newCall(request).execute()
                if (response.isSuccessful) {
                    response.body?.bytes()
                } else {
                    tryFallbackCover(url)
                }
            } catch (e: Exception) {
                tryFallbackCover(url)
            }
        }
    }

    private fun tryFallbackCover(originalUrl: String): ByteArray? {
        val sizes = listOf(500, 300, 150)
        val coverPattern = Regex("""T002R(\d+)x(\d+)M000(.+)\.jpg$""")
        val match = coverPattern.find(originalUrl) ?: return null
        val mid = match.groupValues[3]

        for (size in sizes) {
            val fallbackUrl = "https://y.gtimg.cn/music/photo_new/T002R${size}x${size}M000$mid.jpg"
            try {
                val request = Request.Builder().url(fallbackUrl).build()
                val response = httpClient.newCall(request).execute()
                if (response.isSuccessful) {
                    return response.body?.bytes()
                }
            } catch (e: Exception) {
                continue
            }
        }
        return null
    }

    private suspend fun downloadLyric(songInfo: SongInfo, outputDir: File, baseName: String) {
        try {
            val lyric = provider.getLyric(mid = songInfo.mid)
            if (lyric != null && !lyric.lrc.isNullOrBlank()) {
                val lrcFile = File(outputDir, "$baseName.lrc")
                if (!lrcFile.exists()) {
                    withContext(Dispatchers.IO) {
                        FileOutputStream(lrcFile).use { output ->
                            output.write(lyric.lrc!!.toByteArray(Charsets.UTF_8))
                        }
                    }
                }
            }
        } catch (e: Exception) {
        }
    }

    suspend fun downloadPlaylist(playlistId: Long, options: DownloadOptions): List<File> {
        val playlist = provider.getPlaylistSongs(playlistId)
        if (playlist == null) {
            notifier.error("获取歌单失败")
            return emptyList()
        }

        totalCount = playlist.songCount
        successCount.set(0)
        failCount.set(0)

        val playlistName = playlist.title.ifBlank { "未知歌单" }
        notifier.info("正在下载歌单: $playlistName ($totalCount 首歌曲)")

        val outputDir = File(File(baseOutputDir, "playlists"), sanitizeFileName(playlistName))
        val format = options.format ?: "{title} - {artist}"

        if (options.threads <= 1) {
            return downloadSequential(playlist.songs, outputDir, format, options)
        }

        return downloadParallel(playlist.songs, outputDir, format, options)
    }

    suspend fun downloadAlbum(albumMid: String, options: DownloadOptions): List<File> {
        val songs = provider.getAlbumSongs(albumMid)
        if (songs.isEmpty()) {
            notifier.error("获取专辑失败")
            return emptyList()
        }

        totalCount = songs.size
        successCount.set(0)
        failCount.set(0)

        val albumName = songs.firstOrNull()?.album?.ifBlank { "未知专辑" } ?: "未知专辑"
        notifier.info("正在下载专辑: $albumName ($totalCount 首歌曲)")

        val outputDir = File(File(baseOutputDir, "albums"), sanitizeFileName(albumName))
        val format = options.format ?: "{track} {title}"

        if (options.threads <= 1) {
            return downloadSequential(songs, outputDir, format, options)
        }

        return downloadParallel(songs, outputDir, format, options)
    }
    
    suspend fun downloadPlaylistByGlobalId(globalId: String, options: DownloadOptions): List<File> {
        val playlist = provider.getPlaylistSongsByGlobalId(globalId)
        if (playlist == null) {
            notifier.error("获取歌单失败")
            return emptyList()
        }

        totalCount = playlist.songCount
        successCount.set(0)
        failCount.set(0)

        val playlistName = playlist.title.ifBlank { "未知歌单" }
        notifier.info("正在下载歌单: $playlistName ($totalCount 首歌曲)")

        val outputDir = File(File(baseOutputDir, "playlists"), sanitizeFileName(playlistName))
        val format = options.format ?: "{title} - {artist}"

        if (options.threads <= 1) {
            return downloadSequential(playlist.songs, outputDir, format, options)
        }

        return downloadParallel(playlist.songs, outputDir, format, options)
    }
    
    private suspend fun downloadSequential(
        songs: List<SongInfo>,
        outputDir: File,
        format: String,
        options: DownloadOptions
    ): List<File> {
        val results = mutableListOf<File>()
        songs.forEachIndexed { index, song ->
            val result = downloadInternal(song, outputDir, format, index + 1, options)
            if (result != null) {
                successCount.incrementAndGet()
                results.add(result)
            } else {
                failCount.incrementAndGet()
            }
        }
        printSummary()
        return results
    }
    
    private suspend fun downloadParallel(
        songs: List<SongInfo>,
        outputDir: File,
        format: String,
        options: DownloadOptions
    ): List<File> {
        val results = ConcurrentHashMap<Int, File>()
        val progressChannel = Channel<ProgressUpdate>(Channel.UNLIMITED)
        var completedCount = 0
        val maxConcurrent = options.threads
        val semaphore = Semaphore(maxConcurrent)

        session {
            data class DownloadState(
                val activeLines: Map<Int, String> = emptyMap(),
                val activeOrder: List<Int> = emptyList()
            )

            var activeState by liveVarOf(DownloadState())
            val completedLines = liveListOf<String>()

            section {
                completedLines.forEach { line ->
                    textLine(line)
                }
                if (completedLines.isNotEmpty()) {
                    textLine()
                }
                activeState.activeOrder.forEach { tid ->
                    textLine(activeState.activeLines[tid] ?: "")
                }
            }.runUntilSignal {
                coroutineScope {
                    launch(Dispatchers.IO) {
                        songs.mapIndexed { index, song ->
                            async(Dispatchers.IO) {
                                semaphore.acquire()
                                try {
                                    val result = downloadFileSilent(
                                        song, outputDir, format, index + 1, options, progressChannel, index
                                    )
                                    if (result != null) {
                                        successCount.incrementAndGet()
                                        results[index] = result
                                    } else {
                                        failCount.incrementAndGet()
                                    }
                                } finally {
                                    semaphore.release()
                                }
                            }
                        }.awaitAll()
                        progressChannel.close()
                    }

                    for (update in progressChannel) {
                        val taskId = update.taskId

                        if (update.completed) {
                            completedLines.add(update.line)
                            completedCount++
                            activeState = activeState.copy(
                                activeLines = activeState.activeLines - taskId,
                                activeOrder = activeState.activeOrder.filter { it != taskId }
                            )
                        } else {
                            val current = activeState
                            val newActiveLines = current.activeLines + (taskId to update.line)
                            val newActiveOrder = if (taskId in current.activeOrder) current.activeOrder else current.activeOrder + taskId
                            activeState = current.copy(
                                activeLines = newActiveLines,
                                activeOrder = newActiveOrder
                            )
                        }
                    }

                    signal()
                }
            }
        }

        printSummary()
        return results.values.toList()
    }

    private fun printSummary() {
        notifier.info("总计下载: $totalCount，成功: ${successCount.get()}，失败: ${failCount.get()}")
    }

    private fun formatFileName(format: String, info: SongInfo, trackNumber: Int, extension: String): String {
        return format
            .replace("{track}", trackNumber.toString().padStart(2, '0'))
            .replace("{title}", sanitizeFileName(info.title))
            .replace("{artist}", sanitizeFileName(info.artist ?: "未知歌手"))
            .replace("{album}", sanitizeFileName(info.album ?: "未知专辑"))
            .replace("{provider}", sanitizeFileName(provider.name))
            .let { "$it.$extension" }
    }

    private fun sanitizeFileName(name: String): String {
        return name.replace(Regex("[<>:\"/\\\\|?*]"), "_")
            .replace(Regex("\\s+"), " ")
            .trim()
    }

    fun close() {
        provider.close()
        httpClient.dispatcher.executorService.shutdown()
        httpClient.connectionPool.evictAll()
    }
}

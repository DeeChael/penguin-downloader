package com.penguin.downloader.cli

import com.penguin.downloader.download.Downloader
import com.penguin.downloader.model.DownloadOptions
import com.penguin.downloader.model.SongInfo
import com.penguin.downloader.model.UrlType
import com.penguin.downloader.plugin.ProviderRegistry
import com.penguin.downloader.provider.MusicProvider
import kotlinx.cli.ArgParser
import kotlinx.cli.ArgType
import kotlinx.cli.default
import kotlinx.coroutines.runBlocking
import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import java.util.logging.Level
import java.util.logging.Logger

fun main(args: Array<String>) {
    val juLogger = Logger.getLogger("org.jaudiotagger")
    juLogger.level = Level.OFF
    juLogger.useParentHandlers = false

    val pluginsDir = File("plugins")
    ProviderRegistry.loadPlugins(pluginsDir)

    val helpArg = extractHelpArg(args)
    if (helpArg != null) {
        printProviderHelp(helpArg)
        return
    }

    if (args.isEmpty()) {
        printHelp()
        return
    }

    val firstArg = args[0]
    if (firstArg == "--clean") {
        cleanDownloadedFiles()
        return
    }

    if (firstArg.startsWith("http://") || firstArg.startsWith("https://")) {
        handleUrlDownload(firstArg, args.drop(1).toTypedArray())
        return
    }

    val parser = ArgParser("penguin-downloader")

    val force by parser.option(
        ArgType.Boolean,
        fullName = "force",
        description = "强制重新下载，覆盖已存在的文件"
    ).default(false)

    val login by parser.option(
        ArgType.String,
        shortName = "L",
        fullName = "login",
        description = "登录指定数据源"
    )

    val logout by parser.option(
        ArgType.String,
        shortName = "x",
        fullName = "logout",
        description = "登出指定数据源"
    )

    val provider by parser.option(
        ArgType.String,
        shortName = "P",
        fullName = "provider",
        description = "API来源"
    )

    val album by parser.option(
        ArgType.String,
        shortName = "a",
        fullName = "album",
        description = "通过专辑ID或MID下载"
    )

    val checkSize by parser.option(
        ArgType.Boolean,
        shortName = "c",
        fullName = "check",
        description = "检查已下载文件大小是否正确，不正确则重新下载"
    ).default(false)

    val format by parser.option(
        ArgType.String,
        shortName = "f",
        fullName = "format",
        description = "文件名格式: {track}, {title}, {artist}, {album}"
    )

    val songId by parser.option(
        ArgType.String,
        shortName = "i",
        fullName = "id",
        description = "通过歌曲ID下载"
    )

    val lyrics by parser.option(
        ArgType.Boolean,
        shortName = "l",
        fullName = "lyrics",
        description = "同时下载歌词"
    ).default(false)

    val output by parser.option(
        ArgType.String,
        shortName = "o",
        fullName = "output",
        description = "输出目录（默认为当前目录）"
    ).default(".")

    val playlist by parser.option(
        ArgType.String,
        shortName = "p",
        fullName = "playlist",
        description = "通过歌单ID下载"
    )

    val quality by parser.option(
        ArgType.Int,
        shortName = "q",
        fullName = "quality",
        description = "音质等级（默认为最高）"
    ).default(0)

    val listPlaylists by parser.option(
        ArgType.Boolean,
        shortName = "r",
        fullName = "list-playlists",
        description = "列出用户歌单"
    ).default(false)

    val searchSong by parser.option(
        ArgType.String,
        shortName = "s",
        fullName = "search-song",
        description = "搜索歌曲"
    )

    val searchAlbum by parser.option(
        ArgType.String,
        shortName = "S",
        fullName = "search-album",
        description = "搜索专辑"
    )

    val threads by parser.option(
        ArgType.Int,
        shortName = "t",
        fullName = "threads",
        description = "下载线程数（1-8，默认1）"
    ).default(1)

    parser.parse(args)

    when {
        logout != null -> {
            val providerName = logout!!.lowercase()
            val targetProvider = ProviderRegistry.getProvider(providerName)
            if (targetProvider != null) {
                if (targetProvider.isLoggedIn) {
                    targetProvider.logout()
                    println("已登出 $providerName")
                } else {
                    println("未登录")
                }
            } else {
                println("未知的 provider: $providerName")
                println("可用: ${ProviderRegistry.getProviderNames().joinToString(", ")}")
            }
            return
        }

        login != null -> {
            val providerName = login!!.lowercase()
            val targetProvider = ProviderRegistry.getProvider(providerName)
            if (targetProvider != null) {
                println("开始登录数据源: $providerName")
                runBlocking {
                    if (targetProvider.login()) {
                        println("\n登录成功，凭证已保存")
                    } else {
                        println("\n登录失败，请检查扫码流程或稍后重试")
                    }
                }
            } else {
                println("未知的 provider: $providerName")
                println("可用: ${ProviderRegistry.getProviderNames().joinToString(", ")}")
            }
            return
        }
    }

    val providerName = provider?.lowercase() ?: run {
        println("错误: 必须指定数据源 (-P <provider>)")
        println()
        printHelp()
        return
    }

    val musicProvider = ProviderRegistry.getProvider(providerName)
    if (musicProvider == null) {
        println("未知的 provider: $providerName")
        println("可用: ${ProviderRegistry.getProviderNames().joinToString(", ")}")
        return
    }

    if (musicProvider.requiresLogin && !musicProvider.isLoggedIn) {
        println("未登录 $providerName，请先使用 -L $providerName 登录")
        return
    }

    val baseOutputDir = File(output)
    val actualThreads = threads.coerceIn(1, 8)
    val actualQuality = if (quality == 0) {
        musicProvider.qualityLevels.keys.maxOrNull() ?: 1
    } else {
        quality
    }
    val downloadOptions = DownloadOptions(actualQuality, format, lyrics, actualThreads, checkSize, force)

    println("使用数据源: $providerName")

    val downloader = Downloader(musicProvider, baseOutputDir = baseOutputDir)

    runBlocking {
        when {
            searchSong != null -> {
                interactiveSearch(musicProvider, downloader, searchSong!!, downloadOptions)
            }

            searchAlbum != null -> {
                interactiveSearchAlbum(musicProvider, downloader, searchAlbum!!, downloadOptions)
            }

            songId != null -> {
                val id = songId!!.toLongOrNull()
                if (id != null) {
                    downloadById(musicProvider, downloader, id, downloadOptions)
                } else {
                    println("无效的歌曲ID")
                }
            }

            playlist != null -> {
                val playlistId = playlist!!.toLongOrNull()
                if (playlistId != null) {
                    downloader.downloadPlaylist(playlistId, downloadOptions)
                } else {
                    println("无效的歌单ID")
                }
            }

            listPlaylists -> {
                interactivePlaylistSelect(musicProvider, downloader, downloadOptions)
            }

            album != null -> {
                downloader.downloadAlbum(album!!, downloadOptions)
            }

            else -> {
                printHelp()
            }
        }
    }

    downloader.close()
}

private suspend fun interactiveSearch(
    provider: MusicProvider,
    downloader: Downloader,
    keyword: String,
    options: DownloadOptions
) {
    val reader = BufferedReader(InputStreamReader(System.`in`))
    var currentPage = 1
    val pageSize = 10

    while (true) {
        println("\n正在搜索: $keyword (第 $currentPage 页)")
        val result = provider.searchSongs(keyword, currentPage, pageSize)

        if (result.code != 200) {
            println("搜索失败: ${result.message}")
            return
        }

        val songs = result.songs
        if (songs.isEmpty()) {
            if (currentPage == 1) {
                println("未找到相关歌曲")
            } else {
                println("已经是最后一页了")
            }
            return
        }

        println("\n搜索结果:\n")

        val headers = listOf("序号", "歌曲名", "艺术家", "专辑", "歌曲 ID")
        val rows = songs.mapIndexed { index, song ->
            listOf(
                ((currentPage - 1) * pageSize + index + 1).toString(),
                song.title,
                song.artist ?: "未知",
                song.album ?: "未知",
                song.id.toString()
            )
        }

        fun displayWidth(s: String): Int {
            return s.sumOf { if (it.code > 127) 2 else 1 }
        }

        fun padRight(s: String, width: Int): String {
            val currentWidth = displayWidth(s)
            return if (currentWidth >= width) s else s + " ".repeat(width - currentWidth)
        }

        val colWidths = headers.mapIndexed { colIndex, header ->
            val maxDataWidth = rows.maxOfOrNull { displayWidth(it[colIndex]) } ?: 0
            maxOf(displayWidth(header), maxDataWidth)
        }

        fun printRow(cells: List<String>) {
            val padded = cells.mapIndexed { i, cell -> padRight(cell, colWidths[i]) }
            println("| ${padded.joinToString(" | ")} |")
        }

        fun printSeparator() {
            val parts = colWidths.map { "-".repeat(it + 2) }
            println("|${parts.joinToString("+")}|")
        }

        printRow(headers)
        printSeparator()
        rows.forEach { printRow(it) }

        println("输入数字下载对应歌曲，输入 n 下一页，输入 p 上一页，输入 q 取消")
        print("> ")

        val input = reader.readLine()?.trim()?.lowercase() ?: "q"

        when (input) {
            "q" -> {
                println("已取消")
                return
            }

            "n" -> {
                currentPage++
            }

            "p" -> {
                if (currentPage > 1) currentPage-- else println("已经是第一页了")
            }

            else -> {
                val num = input.toIntOrNull()
                if (num != null && num in 1..songs.size) {
                    val selected = songs[num - 1]
                    println("\n已选择: ${selected.title} - ${selected.artist}")
                    downloader.downloadSong(selected, options)
                    return
                } else {
                    println("无效的输入")
                }
            }
        }
    }
}

private suspend fun interactiveSearchAlbum(
    provider: MusicProvider,
    downloader: Downloader,
    keyword: String,
    options: DownloadOptions
) {
    val reader = BufferedReader(InputStreamReader(System.`in`))
    var currentPage = 1
    val pageSize = 10

    while (true) {
        println("\n正在搜索专辑: $keyword (第 $currentPage 页)")
        val result = provider.searchAlbums(keyword, currentPage, pageSize)

        if (result == null) {
            println("该数据源不支持搜索专辑")
            return
        }

        if (result.code != 200) {
            println("搜索失败: ${result.message}")
            return
        }

        val albums = result.albums
        if (albums.isEmpty()) {
            if (currentPage == 1) {
                println("未找到相关专辑")
            } else {
                println("已经是最后一页了")
            }
            return
        }

        println("\n搜索结果:\n")

        val headers = listOf("序号", "专辑名", "艺术家", "曲目数", "专辑 ID")
        val rows = albums.mapIndexed { index, album ->
            listOf(
                ((currentPage - 1) * pageSize + index + 1).toString(),
                album.title,
                album.artist ?: "未知",
                album.songCount?.toString() ?: "-",
                album.id.toString()
            )
        }

        fun displayWidth(s: String): Int {
            return s.sumOf { if (it.code > 127) 2 else 1 }
        }

        fun padRight(s: String, width: Int): String {
            val currentWidth = displayWidth(s)
            return if (currentWidth >= width) s else s + " ".repeat(width - currentWidth)
        }

        val colWidths = headers.mapIndexed { colIndex, header ->
            val maxDataWidth = rows.maxOfOrNull { displayWidth(it[colIndex]) } ?: 0
            maxOf(displayWidth(header), maxDataWidth)
        }

        fun printRow(cells: List<String>) {
            val padded = cells.mapIndexed { i, cell -> padRight(cell, colWidths[i]) }
            println("| ${padded.joinToString(" | ")} |")
        }

        fun printSeparator() {
            val parts = colWidths.map { "-".repeat(it + 2) }
            println("|${parts.joinToString("+")}|")
        }

        printRow(headers)
        printSeparator()
        rows.forEach { printRow(it) }

        println("输入数字下载对应专辑，输入 n 下一页，输入 p 上一页，输入 q 取消")
        print("> ")

        val input = reader.readLine()?.trim()?.lowercase() ?: "q"

        when (input) {
            "q" -> {
                println("已取消")
                return
            }

            "n" -> {
                currentPage++
            }

            "p" -> {
                if (currentPage > 1) currentPage-- else println("已经是第一页了")
            }

            else -> {
                val num = input.toIntOrNull()
                if (num != null && num in 1..albums.size) {
                    val selected = albums[num - 1]
                    println("\n已选择: ${selected.title} - ${selected.artist}")
                    downloader.downloadAlbum(selected.mid, options)
                    return
                } else {
                    println("无效的输入")
                }
            }
        }
    }
}

private fun cleanDownloadedFiles() {
    val currentDir = File(".")
    val dirsToClean = listOf("songs", "albums", "playlists")
    var totalDeleted = 0L
    var fileCount = 0
    var dirCount = 0

    println("正在清理已下载的文件...")

    for (dirName in dirsToClean) {
        val dir = File(currentDir, dirName)
        if (dir.exists() && dir.isDirectory) {
            dir.walkTopDown().forEach { file ->
                if (file.isFile) {
                    totalDeleted += file.length()
                    fileCount++
                } else if (file != dir) {
                    dirCount++
                }
            }
            dir.deleteRecursively()
            println("已清理: ${dir.path}/")
        }
    }

    if (fileCount > 0 || dirCount > 0) {
        val sizeStr = if (totalDeleted < 1024) {
            "$totalDeleted B"
        } else if (totalDeleted < 1024 * 1024) {
            String.format("%.1f KB", totalDeleted / 1024.0)
        } else {
            String.format("%.1f MB", totalDeleted / (1024.0 * 1024))
        }
        println("\n已删除 $fileCount 个文件，$dirCount 个文件夹，共释放 $sizeStr 空间")
    } else {
        println("没有找到已下载的文件")
    }
}

private suspend fun interactivePlaylistSelect(
    provider: MusicProvider,
    downloader: Downloader,
    options: DownloadOptions
) {
    println("正在获取用户歌单列表...")
    val playlists = provider.getUserPlaylists()

    if (playlists == null || playlists.isEmpty()) {
        println("未找到歌单")
        return
    }

    println("\n我的歌单:\n")

    fun displayWidth(s: String): Int {
        return s.sumOf { if (it.code > 127) 2 else 1 }
    }

    fun padRight(s: String, width: Int): String {
        val currentWidth = displayWidth(s)
        return if (currentWidth >= width) s else s + " ".repeat(width - currentWidth)
    }

    val numWidth = displayWidth("${playlists.size}").coerceAtLeast(displayWidth("序号"))
    val titleWidth = playlists.maxOfOrNull { displayWidth(it.title) }?.coerceAtLeast(displayWidth("歌单名"))
        ?: displayWidth("歌单名")
    val countWidth = playlists.maxOfOrNull { displayWidth("${it.songCount} 首") }?.coerceAtLeast(displayWidth("歌曲数"))
        ?: displayWidth("歌曲数")

    fun formatRow(p1: Triple<String, String, String>, p2: Triple<String, String, String>?): String {
        val n1 = padRight(p1.first, numWidth)
        val t1 = padRight(p1.second, titleWidth)
        val c1 = padRight(p1.third, countWidth)
        val left = "$n1 | $t1 | $c1"

        val right = if (p2 != null) {
            val n2 = padRight(p2.first, numWidth)
            val t2 = padRight(p2.second, titleWidth)
            val c2 = padRight(p2.third, countWidth)
            "$n2 | $t2 | $c2"
        } else {
            " ".repeat(numWidth + titleWidth + countWidth + 6)
        }

        return "| $left | $right |"
    }

    fun formatSeparator(): String {
        val n = "-".repeat(numWidth + 2)
        val t = "-".repeat(titleWidth + 2)
        val c = "-".repeat(countWidth + 2)
        return "|$n+$t+$c+$n+$t+$c|"
    }

    println(formatRow(Triple("序号", "歌单名", "歌曲数"), Triple("序号", "歌单名", "歌曲数")))
    println(formatSeparator())

    for (i in playlists.indices step 2) {
        val left = Triple((i + 1).toString(), playlists[i].title, "${playlists[i].songCount} 首")
        val right = if (i + 1 < playlists.size) {
            Triple((i + 2).toString(), playlists[i + 1].title, "${playlists[i + 1].songCount} 首")
        } else {
            null
        }
        println(formatRow(left, right))
    }

    println("\n输入数字下载对应歌单，输入 q 取消")
    print("> ")

    val reader = BufferedReader(InputStreamReader(System.`in`))
    val input = reader.readLine()?.trim()?.lowercase() ?: "q"

    if (input == "q") {
        println("已取消")
        return
    }

    val num = input.toIntOrNull()
    if (num != null && num in 1..playlists.size) {
        val selected = playlists[num - 1]
        println("\n已选择: ${selected.title}")
        downloader.downloadPlaylist(selected.id, options)
    } else {
        println("无效的输入")
    }
}

private suspend fun downloadById(
    provider: MusicProvider,
    downloader: Downloader,
    id: Long,
    options: DownloadOptions
) {
    val detail = provider.getSongDetail(id = id, mid = null)

    if (detail != null) {
        val songInfo = SongInfo(
            id = detail.id,
            mid = detail.mid,
            title = detail.title,
            subtitle = detail.subtitle,
            artist = detail.artist,
            album = detail.album,
            cover = detail.cover,
            duration = detail.duration
        )
        downloader.downloadSong(songInfo, options)
    } else {
        println("获取歌曲失败")
    }
}

private suspend fun downloadByMid(
    provider: MusicProvider,
    downloader: Downloader,
    mid: String,
    options: DownloadOptions
) {
    val detail = provider.getSongDetail(id = null, mid = mid)

    if (detail != null) {
        val songInfo = SongInfo(
            id = detail.id,
            mid = detail.mid,
            title = detail.title,
            subtitle = detail.subtitle,
            artist = detail.artist,
            album = detail.album,
            cover = detail.cover,
            duration = detail.duration
        )
        downloader.downloadSong(songInfo, options)
    } else {
        println("获取歌曲失败")
    }
}

private fun extractHelpArg(args: Array<String>): String? {
    for (i in args.indices) {
        if (args[i] == "-h" || args[i] == "--help") {
            return if (i + 1 < args.size && !args[i + 1].startsWith("-")) {
                args[i + 1]
            } else {
                ""
            }
        }
        if (args[i].startsWith("-h=") || args[i].startsWith("--help=")) {
            return args[i].substringAfter("=")
        }
    }
    return null
}

private fun printHelp() {
    println("penguin-downloader - 音乐下载器")
    println()
    println("用法:")
    println("  penguin-downloader <URL> [选项]     通过链接下载（自动识别数据源）")
    println("  penguin-downloader [选项]           通过命令行参数下载")
    println()
    println("选项:")
    println("  -h, --help <provider>    显示指定 provider 的详细信息")
    println("  --clean                  清除所有已下载的歌曲")
    println("  --force                  强制重新下载，覆盖已存在的文件")
    println("  -L, --login <名称>       登录指定数据源")
    println("  -P, --provider <名称>    API来源，必选")
    println("  -a, --album <ID/MID>     通过专辑ID或MID下载")
    println("  -c, --check              检查已下载文件大小是否正确")
    println("  -f, --format <格式>      文件名格式，关键词: {track}=曲目, {title}=歌名,")
    println("                           {artist}=歌手, {album}=专辑, {provider}=来源")
    println("  -i, --id <ID>            通过歌曲ID下载")
    println("  -l, --lyrics             同时下载歌词")
    println("  -o, --output <目录>      输出目录（默认为当前目录）")
    println("  -p, --playlist <ID>      通过歌单ID下载")
    println("  -q, --quality <等级>     音质等级（默认为最高）")
    println("  -r, --list-playlists     列出用户歌单")
    println("  -s, --search-song <关键词>    搜索并下载歌曲（交互式选择）")
    println("  -S, --search-album <关键词>   搜索并下载专辑（交互式选择）")
    println("  -t, --threads <1-8>      下载线程数（默认1）")
    println("  -x, --logout <名称>      登出指定数据源")
    println()
    println("下载目录结构:")
    println("  单曲: <输出目录>/songs/")
    println("  专辑: <输出目录>/albums/<专辑名>/")
    println("  歌单: <输出目录>/playlists/<歌单名>/")
    println()
    val providers = ProviderRegistry.getProviderNames()
    if (providers.isNotEmpty()) {
        println("可用的 provider: ${providers.joinToString(", ")}")
        println("使用 -h <provider> 查看详细信息和支持的链接格式")
    } else {
        println("未加载任何 provider，请将 provider JAR 文件放入 plugins 目录")
    }
}

private fun printProviderHelp(providerName: String) {
    if (providerName.isEmpty()) {
        println("用法: penguin-downloader -h <provider>")
        val providers = ProviderRegistry.getProviderNames()
        if (providers.isNotEmpty()) {
            println("可用的 provider: ${providers.joinToString(", ")}")
        } else {
            println("未加载任何 provider")
        }
        return
    }

    val provider = ProviderRegistry.getProvider(providerName)
    if (provider == null) {
        println("未知的 provider: $providerName")
        val providers = ProviderRegistry.getProviderNames()
        if (providers.isNotEmpty()) {
            println("可用的 provider: ${providers.joinToString(", ")}")
        }
        return
    }

    println("Provider: ${provider.name}")
    println()
    println("需要登录: ${if (provider.requiresLogin) "是" else "否"}")
    println()
    println("音质等级:")
    val qualityLevels = provider.qualityLevels.toSortedMap()
    if (qualityLevels.isNotEmpty()) {
        qualityLevels.forEach { (level, name) ->
            println("  $level   - $name")
        }
    } else {
        println("  无音质等级信息")
    }

    if (provider.supportedUrlDomains.isNotEmpty()) {
        println()
        println("支持的链接域名:")
        provider.supportedUrlDomains.forEach { domain ->
            println("  $domain")
        }
    }
}

private fun handleUrlDownload(url: String, remainingArgs: Array<String>) {
    val provider = ProviderRegistry.getProviderNames()
        .mapNotNull { name -> ProviderRegistry.getProvider(name) }
        .firstOrNull { it.canHandleUrl(url) }

    if (provider == null) {
        println("无法识别的链接，没有找到支持的数据源")
        println("支持的链接格式:")
        ProviderRegistry.getProviderNames().forEach { name ->
            val p = ProviderRegistry.getProvider(name)
            if (p != null && p.supportedUrlDomains.isNotEmpty()) {
                println("  ${p.name}: ${p.supportedUrlDomains.joinToString(", ")}")
            }
        }
        return
    }

    if (provider.requiresLogin && !provider.isLoggedIn) {
        println("未登录 ${provider.name}，请先使用 -L ${provider.name} 登录")
        return
    }

    println("正在解析链接...")

    val parseResult = runBlocking { provider.parseUrl(url) }
    if (parseResult == null) {
        println("无法解析链接内容")
        return
    }

    val parser = ArgParser("penguin-downloader")
    val force by parser.option(ArgType.Boolean, fullName = "force").default(false)
    val checkSize by parser.option(ArgType.Boolean, shortName = "c", fullName = "check").default(false)
    val format by parser.option(ArgType.String, shortName = "f", fullName = "format")
    val lyrics by parser.option(ArgType.Boolean, shortName = "l", fullName = "lyrics").default(false)
    val output by parser.option(ArgType.String, shortName = "o", fullName = "output").default(".")
    val quality by parser.option(ArgType.Int, shortName = "q", fullName = "quality").default(0)
    val threads by parser.option(ArgType.Int, shortName = "t", fullName = "threads").default(1)

    parser.parse(remainingArgs)

    val baseOutputDir = File(output)
    val actualThreads = threads.coerceIn(1, 8)
    val actualQuality = if (quality == 0) {
        provider.qualityLevels.keys.maxOrNull() ?: 1
    } else {
        quality
    }
    val downloadOptions = DownloadOptions(actualQuality, format, lyrics, actualThreads, checkSize, force)

    println("使用数据源: ${provider.name}")

    val downloader = Downloader(provider, baseOutputDir = baseOutputDir)

    runBlocking {
        when (parseResult.type) {
            UrlType.SONG -> {
                val id = parseResult.id?.toLongOrNull()
                val mid = parseResult.mid
                println("检测到单曲链接")
                if (id != null) {
                    downloadById(provider, downloader, id, downloadOptions)
                } else if (mid != null) {
                    downloadByMid(provider, downloader, mid, downloadOptions)
                } else {
                    println("无法获取歌曲ID")
                }
            }

            UrlType.ALBUM -> {
                val albumId = parseResult.id ?: parseResult.albumId
                if (albumId != null) {
                    println("检测到专辑链接")
                    downloader.downloadAlbum(albumId, downloadOptions)
                } else {
                    println("无法获取专辑ID")
                }
            }

            UrlType.PLAYLIST -> {
                val globalId = parseResult.id
                if (globalId != null) {
                    val longId = globalId.toLongOrNull()
                    if (longId != null) {
                        println("检测到歌单链接")
                        downloader.downloadPlaylist(longId, downloadOptions)
                    } else {
                        println("检测到歌单链接: $globalId")
                        downloader.downloadPlaylistByGlobalId(globalId, downloadOptions)
                    }
                } else {
                    println("无法获取歌单ID")
                }
            }
        }
    }

    downloader.close()
}

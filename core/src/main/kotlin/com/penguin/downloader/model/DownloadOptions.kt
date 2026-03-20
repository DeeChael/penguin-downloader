package com.penguin.downloader.model

/**
 * 下载参数
 *
 * @param quality 音乐的品质，由音乐源提供
 * @param format 下载歌曲的文件命名格式
 * @param downloadLyrics 是否下载歌词
 * @param threads 下载线程数
 * @param checkSize 对下载的文件进行检查，如果文件不符合大小重新下载
 * @param force 不论文件是否已下载都重新下载
 */
data class DownloadOptions(
    val quality: Int = 7, // 国内音乐软件应该最多就 7 中分级吧
    val format: String? = null,
    val downloadLyrics: Boolean = false,
    val threads: Int = 1,
    val checkSize: Boolean = false,
    val force: Boolean = false
)

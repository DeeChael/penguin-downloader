package com.penguin.downloader.model

data class SongDetail(
    val id: Long,
    val mid: String,
    val title: String,
    val subtitle: String? = null,
    val artist: String? = null,
    val album: String? = null,
    val cover: String? = null,
    val duration: Int? = null,
    val publishDate: String? = null,
    val extras: Map<String, Any> = emptyMap()
)

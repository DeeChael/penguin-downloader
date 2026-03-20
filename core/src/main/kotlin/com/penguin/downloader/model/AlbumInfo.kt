package com.penguin.downloader.model

data class AlbumInfo(
    val id: Long,
    val mid: String,
    val title: String,
    val artist: String?,
    val cover: String?,
    val songCount: Int?,
    val publishTime: String?
)

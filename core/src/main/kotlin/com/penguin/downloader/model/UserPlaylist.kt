package com.penguin.downloader.model

data class UserPlaylist(
    val id: Long,
    val title: String,
    val songCount: Int,
    val cover: String?
)

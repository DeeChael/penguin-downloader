package com.penguin.downloader.model

data class PlaylistResult(
    val title: String,
    val cover: String? = null,
    val songCount: Int,
    val songs: List<SongInfo>
)

package com.penguin.downloader.model

data class SearchResult(
    val code: Int,
    val message: String,
    val songs: List<SongInfo>
)

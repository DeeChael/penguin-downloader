package com.penguin.downloader.model

data class AlbumSearchResult(
    val code: Int,
    val message: String?,
    val albums: List<AlbumInfo>
)

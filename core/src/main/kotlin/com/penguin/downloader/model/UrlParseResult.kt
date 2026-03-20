package com.penguin.downloader.model

enum class UrlType {
    SONG,
    ALBUM,
    PLAYLIST
}

data class UrlParseResult(
    val type: UrlType,
    val id: String? = null,
    val mid: String? = null,
    val albumId: String? = null
)

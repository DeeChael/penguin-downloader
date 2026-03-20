package com.penguin.downloader.model

data class SongUrlResult(
    val url: String?,
    val quality: String?,
    val encrypted: Boolean = false,
    val errorMessage: String? = null,
    val extras: Map<String, Any> = emptyMap()
) {
    fun isSuccess(): Boolean = !url.isNullOrEmpty()
}

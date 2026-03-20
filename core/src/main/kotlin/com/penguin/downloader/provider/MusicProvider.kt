package com.penguin.downloader.provider

import com.penguin.downloader.model.*

interface MusicProvider {
    val name: String
    val qualityLevels: Map<Int, String>
    val requiresLogin: Boolean
    val supportedUrlDomains: List<String>
        get() = emptyList()
    
    val isLoggedIn: Boolean
        get() = false
    
    suspend fun login(): Boolean = false
    
    fun logout() {}
    
    fun canHandleUrl(url: String): Boolean = false
    
    suspend fun parseUrl(url: String): UrlParseResult? = null
    
    suspend fun searchSongs(keyword: String, page: Int, num: Int): SearchResult
    suspend fun searchAlbums(keyword: String, page: Int, num: Int): AlbumSearchResult? = null
    suspend fun getSongUrl(mid: String, quality: Int): SongUrlResult?
    suspend fun getSongDetail(id: Long? = null, mid: String? = null): SongDetail?
    suspend fun getLyric(id: Long? = null, mid: String? = null): LyricResult?
    suspend fun getAlbumSongs(mid: String): List<SongInfo>
    suspend fun getPlaylistSongs(id: Long): PlaylistResult?
    suspend fun getPlaylistSongsByGlobalId(globalId: String): PlaylistResult? = null
    suspend fun getUserPlaylists(): List<UserPlaylist>? = null
    
    fun close()
    
    fun getQualityName(level: Int): String {
        return qualityLevels[level] ?: "未知音质"
    }
}

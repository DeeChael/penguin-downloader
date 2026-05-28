use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::CoreError;
use crate::login::LoginMethod;
use crate::models::*;

pub type Result<T> = std::result::Result<T, CoreError>;

#[async_trait]
pub trait MusicProvider: Send + Sync {
    fn info(&self) -> Result<ProviderInfo>;

    fn list_qualities(&self) -> Result<Vec<i64>>;

    fn list_login_methods(&self) -> Result<Vec<LoginMethod>>;

    fn get_username(&self, credential: &str) -> Option<String>;

    async fn get_song_info(
        &self,
        id: &str,
        credential: Option<String>,
    ) -> Result<SongInfo>;

    async fn search_songs(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    async fn search_songs_by_lyrics(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    async fn search_albums(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<AlbumInfo>>;

    async fn search_playlists(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<PlaylistInfo>>;

    async fn search_artists(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<ArtistInfo>>;

    async fn list_album_songs(
        &self,
        album: AlbumRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    async fn list_playlist_songs(
        &self,
        playlist: PlaylistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    async fn list_artist_songs(
        &self,
        artist: ArtistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    async fn list_artist_albums(
        &self,
        artist: ArtistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<AlbumInfo>>;

    async fn get_song_urls(
        &self,
        song_quality_map: HashMap<SongInfo, i64>,
        credential: Option<String>,
    ) -> Result<HashMap<String, String>>;

    fn get_verbatim_provider(&self) -> Result<Option<Box<dyn VerbatimProvider>>>;

    fn get_download_referer(&self) -> Result<String>;

    fn get_extra_formatting_names(&self) -> Result<Vec<String>>;

    fn format_file_name(&self, filename: &str) -> String;

    async fn get_lyrics(
        &self,
        song: SongRef,
        verbatim: bool,
        translation: bool,
        roma: bool,
        credential: Option<String>,
    ) -> Result<LyricsResult>;
}

pub trait MetadataProvider: Send + Sync {
    fn info(&self) -> ProviderInfo;
}

pub trait VerbatimProvider: Send + Sync {
    fn get_extension_name(&self) -> String;
    fn decrypt(&self, input: &str) -> String;
}

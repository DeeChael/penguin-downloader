use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::CoreError;
use crate::login::LoginMethod;
use crate::models::*;

/// 库内部使用的 Result 类型。
pub type Result<T> = std::result::Result<T, CoreError>;

/// 音源提供者（Music Provider）trait。
///
/// 实现此 trait 以注册一个音源服务，提供歌曲搜索、专辑/歌单管理、
/// 歌词获取和下载 URL 获取等功能。
#[async_trait]
pub trait MusicProvider: Send + Sync {
    /// 获取提供者的基本信息（ID 和版本号）。
    fn info(&self) -> Result<ProviderInfo>;

    /// 获取此提供者支持的所有音质等级列表。
    ///
    /// 数字越大表示音质越好。
    fn list_qualities(&self) -> Result<Vec<i64>>;

    /// 获取此提供者支持的登录方式列表。
    fn list_login_methods(&self) -> Result<Vec<LoginMethod>>;

    /// 通过登录凭证获取用户名。
    fn get_username(&self, credential: &str) -> Option<String>;

    /// 通过歌曲 ID 获取 `SongInfo`。
    async fn get_song_info(
        &self,
        id: &str,
        credential: Option<String>,
    ) -> Result<SongInfo>;

    /// 根据关键词搜索歌曲。
    async fn search_songs(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    /// 根据歌词内容搜索歌曲。
    async fn search_songs_by_lyrics(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    /// 根据关键词搜索专辑。
    async fn search_albums(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<AlbumInfo>>;

    /// 根据关键词搜索歌单。
    async fn search_playlists(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<PlaylistInfo>>;

    /// 根据关键词搜索艺术家。
    async fn search_artists(
        &self,
        keyword: &str,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<ArtistInfo>>;

    /// 获取专辑下的所有歌曲（需处理分页）。
    async fn list_album_songs(
        &self,
        album: AlbumRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    /// 获取歌单下的所有歌曲（需处理分页）。
    async fn list_playlist_songs(
        &self,
        playlist: PlaylistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    /// 获取艺术家的所有歌曲（需处理分页）。
    async fn list_artist_songs(
        &self,
        artist: ArtistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<SongInfo>>;

    /// 获取艺术家的所有专辑（需处理分页）。
    async fn list_artist_albums(
        &self,
        artist: ArtistRef,
        pagination: Option<Pagination>,
        credential: Option<String>,
    ) -> Result<SearchResult<AlbumInfo>>;

    /// 批量获取歌曲的下载 URL。
    ///
    /// `song_quality_map` 为歌曲与请求音质的映射。当歌曲不支持请求的音质时，
    /// 先寻找小于请求音质的最高可用音质，再寻找大于请求音质的最低可用音质。
    /// 返回值中键为歌曲 ID，值为下载 URL。
    async fn get_song_urls(
        &self,
        song_quality_map: HashMap<SongInfo, i64>,
        credential: Option<String>,
    ) -> Result<HashMap<String, String>>;

    /// 获取逐字歌词提供者。返回 `None` 表示不支持逐字歌词。
    fn get_verbatim_provider(&self) -> Result<Option<Box<dyn VerbatimProvider>>>;

    /// 获取下载时需要的 Referer 头信息，用于防盗链。
    fn get_download_referer(&self) -> Result<String>;

    /// 获取此提供者支持的额外格式化变量名列表。
    fn get_extra_formatting_names(&self) -> Result<Vec<String>>;

    /// 对文件名中的提供者自定义格式化变量进行替换。
    fn format_file_name(&self, filename: &str) -> String;

    /// 批量获取歌曲歌词。
    ///
    /// 如果请求的歌词类型不支持，则返回错误。
    async fn get_lyrics(
        &self,
        song: SongRef,
        verbatim: bool,
        translation: bool,
        roma: bool,
        credential: Option<String>,
    ) -> Result<LyricsResult>;

    /// 获取此音源提供者内置的元数据提供者。
    ///
    /// 返回的 `MetadataProvider` 的 ID 与此 `MusicProvider` 相同，且不会被注册在 `PenguinCore` 中。
    /// 返回 `None` 表示此音源提供者没有内置的元数据提供者。
    fn get_integrated_metadata_provider(&self) -> Option<Arc<dyn MetadataProvider>> {
        None
    }
}

/// 元数据提供者（Metadata Provider）trait。
///
/// 提供元数据查询功能，用于为已下载的音频文件嵌入标签。
#[async_trait]
pub trait MetadataProvider: Send + Sync {
    /// 获取提供者的基本信息。
    fn info(&self) -> Result<ProviderInfo>;

    /// 获取此提供者支持的登录方式列表。
    fn list_login_methods(&self) -> Result<Vec<LoginMethod>>;

    /// 通过登录凭证获取用户名。
    fn get_username(&self, credential: &str) -> Option<String>;

    /// 获取指定歌曲的元数据。
    async fn get_metadata(
        &self,
        song: &SongInfo,
        credential: Option<String>,
    ) -> Result<SongMetadata>;
}

/// 逐字歌词提供者（Verbatim Provider）trait。
///
/// 处理逐字歌词的解密和文件扩展名。
pub trait VerbatimProvider: Send + Sync {
    /// 获取逐字歌词文件的扩展名（如 `"vtt"`）。
    fn get_extension_name(&self) -> String;
    /// 解密输入的歌词内容，返回解密后的字符串。
    fn decrypt(&self, input: &str) -> String;
}

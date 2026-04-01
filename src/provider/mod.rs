mod option_value;

pub use option_value::ProviderOptionValue;

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::model::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMethod {
    None,
    QR,
    URL,
    Account,
    Code
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
}

impl ProviderInfo {
    pub fn new(
        id: impl Into<String>,
        plugin_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoginStatus {
    Pending,
    Scanned,
    Success,
    Failed(String),
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone)]
pub enum QrLoginData {
    Image(Vec<u8>),
    Url(String),
}

pub trait QrLoginCallback: Send + Sync {
    fn on_qr_data(&self, data: QrLoginData);
}

pub trait UrlLoginCallback: Send + Sync {
    fn on_url(&self, url: String);
}

pub trait CodeLoginCallback: Send + Sync {
    /// 请求验证码
    /// 
    /// # Arguments
    /// * `url` - 可选的 URL，某些平台可能需要用户访问此 URL 完成人机验证
    fn request_code(&self, url: Option<&str>) -> String;
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub per_page: i32,
    pub page: i32,
}

impl Pagination {
    pub fn new(per_page: i32, page: i32) -> Self {
        Self {
            per_page: per_page.max(1),
            page: page.max(1),
        }
    }

    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }
}

impl Pagination {
    pub fn default_search() -> Self {
        Self {
            per_page: 10,
            page: 1,
        }
    }

    pub fn default_list() -> Self {
        Self {
            per_page: 100,
            page: 1,
        }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::default_search()
    }
}

#[async_trait]
pub trait MusicProvider: Send + Sync {
    fn name(&self) -> &str;

    fn quality_levels(&self) -> &HashMap<i32, String>;

    fn get_highest_quality(&self) -> i32 {
        self.quality_levels()
            .keys()
            .copied()
            .max()
            .unwrap_or(0)
    }

    fn info(&self) -> ProviderInfo;

    fn supported_login_methods(&self) -> Vec<LoginMethod> {
        vec![LoginMethod::None]
    }

    async fn start_qr_login(
        &self,
        _callback: &dyn QrLoginCallback,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("QR login not supported".to_string()))
    }

    async fn start_url_login(
        &self,
        _callback: &dyn UrlLoginCallback,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("URL login not supported".to_string()))
    }

    async fn start_account_login(
        &self,
        _username: &str,
        _password: &str,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("Account login not supported".to_string()))
    }

    async fn start_code_login(
        &self,
        _account: &str,
        _callback: &dyn CodeLoginCallback,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("Code login not supported".to_string()))
    }

    async fn refresh_and_validate(
        &self,
        _credential: &str,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("Refresh not supported".to_string()))
    }

    fn requires_login(&self) -> bool {
        !matches!(self.supported_login_methods().as_slice(), [LoginMethod::None])
    }

    fn has_metadata(&self) -> bool {
        false
    }

    async fn search_songs(
        &self,
        keyword: &str,
        pagination: Pagination,
        credential: Option<&str>,
    ) -> crate::Result<SearchResult>;

    async fn search_albums(
        &self,
        keyword: &str,
        pagination: Pagination,
        credential: Option<&str>,
    ) -> crate::Result<AlbumSearchResult> {
        let _ = (keyword, pagination, credential);
        Err(crate::Error::NoDataExists)
    }

    async fn get_song_url(
        &self,
        id: &str,
        quality: i32,
        credential: Option<&str>,
    ) -> crate::Result<SongUrlResult>;

    async fn get_song_detail(
        &self,
        id: &str,
        credential: Option<&str>,
    ) -> crate::Result<SongInfo> {
        let _ = (id, credential);
        Err(crate::Error::NoDataExists)
    }

    async fn get_lyric(
        &self,
        id: &str,
        verbatim: bool,
        trans: bool,
        roma: bool,
        credential: Option<&str>,
    ) -> crate::Result<LyricResult> {
        let _ = (id, verbatim, trans, roma, credential);
        Err(crate::Error::NoDataExists)
    }

    fn supports_verbatim_lyrics(&self) -> bool {
        false
    }

    fn verbatim_lyric_extension(&self) -> &'static str {
        ""
    }

    fn decrypt_verbatim_lyric(&self, encrypted: &str) -> String {
        encrypted.to_string()
    }

    async fn get_album_songs(
        &self,
        id: &str,
        pagination: Pagination,
        credential: Option<&str>,
    ) -> crate::Result<Vec<SongInfo>>;

    async fn get_playlist_songs(
        &self,
        id: &str,
        pagination: Pagination,
        credential: Option<&str>,
    ) -> crate::Result<PlaylistResult>;

    async fn get_user_playlists(
        &self,
        credential: Option<&str>,
    ) -> crate::Result<Vec<UserPlaylist>> {
        let _ = credential;
        Err(crate::Error::NoDataExists)
    }

    fn download_referer(&self) -> String {
        String::new()
    }

    fn format_file_name_custom(
        &self,
        format: &str,
        _info: &SongInfo,
    ) -> String {
        format.to_string()
    }

    fn close(&self) {}

    fn get_quality_name(&self, level: i32) -> String {
        self.quality_levels()
            .get(&level)
            .cloned()
            .unwrap_or_else(|| "未知音质".to_string())
    }
}
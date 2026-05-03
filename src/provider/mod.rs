//! Provider module

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::model::*;

// 基础类型
pub mod types;

// 登录相关（包括 LoginStatus 和 QrLoginData）
pub mod login;

// 选项相关
pub mod option;

// Re-exports for convenience
pub use types::{Pagination};
pub use login::{LoginMethodType, LoginStatus, QrLoginData, QrLoginCallback, UrlLoginCallback, CodeLoginCallback, QrLoginHandler, UrlLoginHandler, AccountLoginHandler, CodeLoginHandler, LoginMethod, QrLoginMethod, UrlLoginMethod, AccountLoginMethod, CodeLoginMethod};
pub use option::{ProviderOptionValue, ProviderOptionDefinition, ProviderOptionType, NumberRange, EnumVariantMap};

/// Provider 信息
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

/// MusicProvider trait - 音乐提供者接口
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

    fn list_login_methods(&self) -> Vec<Box<dyn LoginMethod>>;

    /// List extra options that can be configured for this provider
    fn list_extra_options(&self) -> Vec<ProviderOptionDefinition> {
        Vec::new()
    }

    /// List enum variant mappings for an enum-type option
    /// Returns None if the option is not an enum type
    fn list_enum_options(&self, option: &ProviderOptionDefinition) -> Option<EnumVariantMap> {
        if !option.is_enum() {
            return None;
        }
        // Default implementation returns empty map
        // Providers should override this to return actual enum variants
        Some(HashMap::new())
    }

    async fn refresh_and_validate(
        &self,
        _credential: &str,
        _timeout: Duration,
    ) -> crate::Result<String> {
        Err(crate::Error::NotSupported("Refresh not supported".to_string()))
    }

    fn requires_login(&self) -> bool;

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
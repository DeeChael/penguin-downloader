use super::AlbumInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSearchResult {
    pub code: i32,
    pub message: Option<String>,
    pub albums: Vec<AlbumInfo>,
}

impl AlbumSearchResult {
    pub fn success(albums: Vec<AlbumInfo>) -> Self {
        Self {
            code: 200,
            message: Some("success".to_string()),
            albums,
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: Some(message.into()),
            albums: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.code == 200
    }
}

use super::SongInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub code: i32,
    pub message: String,
    pub songs: Vec<SongInfo>,
}

impl SearchResult {
    pub fn success(songs: Vec<SongInfo>) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            songs,
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            songs: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.code == 200
    }
}

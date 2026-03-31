use super::SongInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResult {
    pub title: String,
    #[serde(default)]
    pub cover: Option<String>,
    pub song_count: i32,
    pub songs: Vec<SongInfo>,
}

impl PlaylistResult {
    pub fn new(title: impl Into<String>, song_count: i32, songs: Vec<SongInfo>) -> Self {
        Self {
            title: title.into(),
            cover: None,
            song_count,
            songs,
        }
    }
}

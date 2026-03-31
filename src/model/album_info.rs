use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub song_count: Option<i32>,
    #[serde(default)]
    pub publish_time: Option<String>,
}

impl AlbumInfo {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: None,
            cover: None,
            song_count: None,
            publish_time: None,
        }
    }
}

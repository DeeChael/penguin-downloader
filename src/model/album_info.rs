use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artists: Vec<String>,
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
            artists: Vec::new(),
            cover: None,
            song_count: None,
            publish_time: None,
        }
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artists.push(artist.into());
        self
    }
}

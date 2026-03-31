use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlaylist {
    pub id: String,
    pub title: String,
    pub song_count: i32,
    #[serde(default)]
    pub cover: Option<String>,
}

impl UserPlaylist {
    pub fn new(id: impl Into<String>, title: impl Into<String>, song_count: i32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            song_count,
            cover: None,
        }
    }
}

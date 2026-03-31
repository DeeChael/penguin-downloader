use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub duration: Option<i32>,
    #[serde(default)]
    pub publish_date: Option<String>,
    #[serde(default)]
    pub track_number: Option<i32>,
    #[serde(default)]
    pub disc_number: Option<i32>,
    #[serde(default)]
    pub extras: HashMap<String, serde_json::Value>,
}

impl SongInfo {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            artist: None,
            album: None,
            cover: None,
            duration: None,
            publish_date: None,
            track_number: None,
            disc_number: None,
            extras: HashMap::new(),
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    pub fn with_cover(mut self, cover: impl Into<String>) -> Self {
        self.cover = Some(cover.into());
        self
    }

    pub fn with_duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_publish_date(mut self, date: impl Into<String>) -> Self {
        self.publish_date = Some(date.into());
        self
    }

    pub fn with_track_number(mut self, track_number: i32) -> Self {
        self.track_number = Some(track_number);
        self
    }

    pub fn with_disc_number(mut self, disc_number: i32) -> Self {
        self.disc_number = Some(disc_number);
        self
    }
}

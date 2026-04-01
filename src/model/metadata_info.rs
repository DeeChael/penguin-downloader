#[derive(Debug, Clone)]
pub struct MetadataInfo {
    pub title: String,
    pub subtitle: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub cover: Option<String>,
    pub publish_date: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
}

impl MetadataInfo {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            artist: None,
            album: None,
            cover: None,
            publish_date: None,
            track_number: None,
            disc_number: None,
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

    pub fn with_publish_date(mut self, date: impl Into<String>) -> Self {
        self.publish_date = Some(date.into());
        self
    }

    pub fn with_track_number(mut self, track: i32) -> Self {
        self.track_number = Some(track);
        self
    }

    pub fn with_disc_number(mut self, disc: i32) -> Self {
        self.disc_number = Some(disc);
        self
    }
}

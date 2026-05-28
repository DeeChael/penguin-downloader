use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum ProviderInfo {
    Music { id: String, version: i64 },
    Metadata { id: String, version: i64 },
}

impl ProviderInfo {
    pub fn id(&self) -> &str {
        match self {
            ProviderInfo::Music { id, .. } => id,
            ProviderInfo::Metadata { id, .. } => id,
        }
    }

    pub fn version(&self) -> i64 {
        match self {
            ProviderInfo::Music { version, .. } => *version,
            ProviderInfo::Metadata { version, .. } => *version,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreferredQuality {
    Highest,
    Lowest,
    Specific(i64),
}

#[derive(Clone, Debug)]
pub struct DownloadOptions {
    pub preferred_quality: PreferredQuality,
    pub format: Option<String>,
    pub lyrics: LyricsType,
    pub lyrics_translation: bool,
    pub lyrics_roma: bool,
    pub force: bool,
    pub on_size_mismatch: Option<fn(u64, u64) -> i8>,
}

#[derive(Clone, Debug)]
pub struct Pagination {
    pub page_size: u64,
    pub page: u64,
}

#[derive(Clone, Debug)]
pub struct SearchResult<T> {
    pub page_size: u64,
    pub page: u64,
    pub total: u64,
    pub results: Vec<T>,
}

#[derive(Clone, Debug)]
pub struct SongInfo {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub artists: Vec<ArtistInfo>,
    pub album: Option<AlbumInfo>,
    pub cover: Option<String>,
    pub duration: Option<u64>,
    pub published_date: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub qualities: Vec<i64>,
    pub size: HashMap<i64, u64>,
    pub extras: HashMap<String, serde_json::Value>,
}

impl std::hash::Hash for SongInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for SongInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SongInfo {}

#[derive(Clone, Debug)]
pub enum SongRef {
    Id(String),
    Info(SongInfo),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LyricsType {
    None,
    Normal,
    Verbatim,
}

#[derive(Clone, Debug)]
pub struct AlbumInfo {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover: Option<String>,
    pub song_count: Option<i32>,
    pub publish_time: Option<String>,
}

#[derive(Clone, Debug)]
pub enum AlbumRef {
    Id(String),
    Info(AlbumInfo),
}

#[derive(Clone, Debug)]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub creator: Vec<String>,
    pub cover: Option<String>,
    pub song_count: Option<i32>,
    pub create_time: Option<String>,
}

#[derive(Clone, Debug)]
pub enum PlaylistRef {
    Id(String),
    Info(PlaylistInfo),
}

#[derive(Clone, Debug)]
pub struct LyricsResult {
    pub lrc: Option<String>,
    pub trans: Option<String>,
    pub roma: Option<String>,
    pub verbatim: Option<String>,
    pub trans_verbatim: Option<String>,
    pub roma_verbatim: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ArtistInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum ArtistRef {
    Id(String),
    Info(ArtistInfo),
}

#[derive(Clone, Copy, Debug)]
pub struct DownloadCallbacks {
    pub on_start: Option<fn(&DownloadStart)>,
    pub on_progress: Option<fn(&DownloadProgress)>,
    pub on_existing: Option<fn(&DownloadExisting)>,
    pub on_complete: Option<fn(&DownloadComplete)>,
    pub on_error: Option<fn(&DownloadError)>,
}

#[derive(Clone, Debug)]
pub struct DownloadStart {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub quality: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DownloadComplete {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub quality: Option<String>,
    pub final_size: u64,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct DownloadExisting {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct DownloadProgress {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub quality: Option<String>,
    pub downloaded: u64,
    pub total_size: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DownloadError {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub error_message: String,
}

use crate::model::SongInfo;

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    /// Quality name
    pub quality: Option<String>,
    pub downloaded: u64,
    pub total_size: Option<u64>,
}

impl DownloadProgress {
    pub fn percent(&self) -> u32 {
        match self.total_size {
            Some(total) if total > 0 => ((self.downloaded * 100) / total) as u32,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadStart {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub quality: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadComplete {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub quality: Option<String>,
    pub final_size: u64,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct DownloadExisting {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub path: std::path::PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub struct DownloadError {
    pub current: i32,
    pub total: i32,
    pub song: SongInfo,
    pub error_message: String,
}

#[derive(Default, Clone)]
pub struct DownloadCallbacks {
    pub on_start: Option<fn(&DownloadStart)>,
    pub on_progress: Option<fn(&DownloadProgress)>,
    pub on_complete: Option<fn(&DownloadComplete)>,
    pub on_error: Option<fn(&DownloadError)>,
}

impl DownloadCallbacks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_start(mut self, callback: fn(&DownloadStart)) -> Self {
        self.on_start = Some(callback);
        self
    }

    pub fn with_progress(mut self, callback: fn(&DownloadProgress)) -> Self {
        self.on_progress = Some(callback);
        self
    }

    pub fn with_complete(mut self, callback: fn(&DownloadComplete)) -> Self {
        self.on_complete = Some(callback);
        self
    }

    pub fn with_error(mut self, callback: fn(&DownloadError)) -> Self {
        self.on_error = Some(callback);
        self
    }
}

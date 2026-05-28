use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    #[error("provider already exists: {0}")]
    ProviderAlreadyExists(String),

    #[error("unsupported quality")]
    UnsupportedQuality,

    #[error("no URL returned for song")]
    NoUrlReturned,

    #[error("file exists but size mismatch: expected {expected}, actual {actual}")]
    FileSizeMismatch { expected: u64, actual: u64 },

    #[error("song not found: {0}")]
    SongNotFound(String),

    #[error("{0}")]
    Custom(String),
}

impl From<&str> for CoreError {
    fn from(s: &str) -> Self {
        CoreError::Custom(s.to_string())
    }
}

impl From<String> for CoreError {
    fn from(s: String) -> Self {
        CoreError::Custom(s)
    }
}

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider '{provider}' does not support operation: {operation}")]
    ProviderNotSupported { provider: String, operation: String },

    #[error("Provider '{0}' returned error: {1}")]
    ProviderApiError(String, String),

    #[error("Plugin load error: {0}")]
    PluginLoad(String),

    #[error("Library load error: {0}")]
    LibraryLoad(String),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP error: status={status}, message={message}")]
    HttpError { status: u16, message: String },

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("File already exists: {path}")]
    AlreadyExists { path: PathBuf },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Download URL not found for song")]
    UrlNotFound,

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Download interrupted")]
    DownloadInterrupted,

    #[error("File size mismatch: expected {expected} bytes, got {actual} bytes")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Album not found: {0}")]
    AlbumNotFound(String),

    #[error("Playlist not found: {0}")]
    PlaylistNotFound(String),

    #[error("Song not found: {0}")]
    SongNotFound(String),

    #[error("Search returned no results")]
    SearchNoResults,

    #[error("Invalid search query: {0}")]
    InvalidSearchQuery(String),

    #[error("Invalid page number: {page}, total pages: {total}")]
    InvalidPage { page: i32, total: i32 },

    #[error("Page out of range: requested {requested}, available {available}")]
    PageOutOfRange { requested: i32, available: i32 },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Invalid quality level: {0}")]
    InvalidQuality(i32),

    #[error("Login required for this operation")]
    LoginRequired,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Session expired")]
    SessionExpired,

    #[error("Login timeout")]
    LoginTimeout,

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("QR code expired")]
    QrCodeExpired,

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Parse error in {context}: {message}")]
    ParseError { context: String, message: String },

    #[error("Failed to parse song info: {0}")]
    SongParseError(String),

    #[error("Failed to parse album info: {0}")]
    AlbumParseError(String),

    #[error("Failed to parse playlist info: {0}")]
    PlaylistParseError(String),

    #[error("Metadata error: {0}")]
    MetadataError(String),

    #[error("Failed to embed cover image")]
    CoverEmbedFailed,

    #[error("Failed to write metadata tag: {0}")]
    MetadataWriteFailed(String),

    #[error("Lyrics not available")]
    LyricsNotAvailable,

    #[error("No data exists")]
    NoDataExists,

    #[error("Failed to decrypt lyrics")]
    LyricsDecryptionFailed,

    #[error("Unsupported lyric format: {0}")]
    UnsupportedLyricFormat(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Operation cancelled by user")]
    Cancelled,
}

impl From<libloading::Error> for Error {
    fn from(e: libloading::Error) -> Self {
        Error::LibraryLoad(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

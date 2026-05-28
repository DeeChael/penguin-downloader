use thiserror::Error;

/// penguin-downloader 的错误类型。
#[derive(Error, Debug)]
pub enum CoreError {
    /// 未找到指定 ID 的提供者。
    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    /// 同 ID 的提供者已存在。
    #[error("provider already exists: {0}")]
    ProviderAlreadyExists(String),

    /// 不支持的音质等级。
    #[error("unsupported quality")]
    UnsupportedQuality,

    /// 无法获取歌曲的下载 URL。
    #[error("no URL returned for song")]
    NoUrlReturned,

    /// 文件已存在但大小不匹配。
    #[error("file exists but size mismatch: expected {expected}, actual {actual}")]
    FileSizeMismatch { expected: u64, actual: u64 },

    /// 未找到歌曲。
    #[error("song not found: {0}")]
    SongNotFound(String),

    /// 自定义错误信息。
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

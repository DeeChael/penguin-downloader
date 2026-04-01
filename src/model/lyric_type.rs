use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LyricType {
    None,
    Normal,
    Verbatim,
}

impl Default for LyricType {
    fn default() -> Self {
        LyricType::None
    }
}

impl std::fmt::Display for LyricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LyricType::None => write!(f, "不下载"),
            LyricType::Normal => write!(f, "普通歌词"),
            LyricType::Verbatim => write!(f, "逐字歌词"),
        }
    }
}

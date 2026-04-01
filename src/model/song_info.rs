use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 歌曲信息
///
/// 包含歌曲的元数据信息，如标题、艺术家、专辑等。
///
/// # Examples
///
/// ```rust
/// use penguin_downloader::SongInfo;
///
/// let song = SongInfo::new("123456", "Song 123")
///     .with_artist("AAA")
///     .with_album("BBB")
///     .with_duration(269);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfo {
    /// 歌曲唯一标识符
    pub id: String,
    /// 歌曲标题
    pub title: String,
    /// 副标题（如 "Live版"、"Remix版" 等）
    #[serde(default)]
    pub subtitle: Option<String>,
    /// 艺术家/歌手
    #[serde(default)]
    pub artist: Option<String>,
    /// 所属专辑
    #[serde(default)]
    pub album: Option<String>,
    /// 封面图片 URL
    #[serde(default)]
    pub cover: Option<String>,
    /// 时长（秒）
    #[serde(default)]
    pub duration: Option<i32>,
    /// 发行日期（字符串格式，如 "2003-07-31"）
    #[serde(default)]
    pub publish_date: Option<String>,
    /// 曲目编号
    #[serde(default)]
    pub track_number: Option<i32>,
    /// 光盘编号（用于多碟专辑）
    #[serde(default)]
    pub disc_number: Option<i32>,
    /// 额外的自定义数据
    #[serde(default)]
    pub extras: HashMap<String, serde_json::Value>,
}

impl SongInfo {
    /// 创建新的歌曲信息实例
    ///
    /// # Arguments
    ///
    /// * `id` - 歌曲唯一标识符
    /// * `title` - 歌曲标题
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

    /// 设置副标题
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// 设置艺术家
    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    /// 设置专辑
    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    /// 设置封面 URL
    pub fn with_cover(mut self, cover: impl Into<String>) -> Self {
        self.cover = Some(cover.into());
        self
    }

    /// 设置时长（秒）
    pub fn with_duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 设置发行日期
    pub fn with_publish_date(mut self, date: impl Into<String>) -> Self {
        self.publish_date = Some(date.into());
        self
    }

    /// 设置曲目编号
    pub fn with_track_number(mut self, track_number: i32) -> Self {
        self.track_number = Some(track_number);
        self
    }

    /// 设置光盘编号
    pub fn with_disc_number(mut self, disc_number: i32) -> Self {
        self.disc_number = Some(disc_number);
        self
    }
}

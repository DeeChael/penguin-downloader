use std::collections::HashMap;

use super::download_callbacks::DownloadCallbacks;
use super::lyric_type::LyricType;
use crate::provider::ProviderOptionValue;

/// 下载选项
///
/// 配置下载行为的选项，包括音质、文件名格式、歌词选项等。
///
/// # 音质等级
///
/// 常见的音质等级：
/// - `0` - 标准音质
/// - `1` - 较高音质
/// - `2` - 超高音质
/// - `3` - 无损音质 (FLAC)
/// - `4` - Hi-Res
///
/// 注意：不同音源支持的具体音质等级可能不同。
///
/// # Examples
///
/// ```rust
/// use penguin_downloader::{DownloadOptions, LyricType};
///
/// let options = DownloadOptions::new()
///     .with_quality(3)  // 无损音质
///     .with_format("{track} {title} - {artist}")
///     .with_lyric_type(LyricType::Lrc)
///     .with_lyric_translation(true);
/// ```
#[derive(Clone)]
pub struct DownloadOptions {
    /// 音质等级（数字越大通常音质越好）
    pub quality: i32,
    /// 文件名格式模板
    ///
    /// 可用占位符：
    /// - `{title}` - 歌曲标题
    /// - `{artist}` - 艺术家
    /// - `{album}` - 专辑名
    /// - `{track}` - 曲目编号
    /// - `{provider}` - 音源名称
    pub format: Option<String>,
    /// 歌词类型
    pub lyric_type: LyricType,
    /// 是否下载翻译歌词
    pub lyric_translation: bool,
    /// 是否下载罗马音歌词
    pub lyric_romanization: bool,
    /// 强制重新下载（忽略已存在文件）
    pub force: bool,
    /// 文件大小不匹配时的处理函数
    ///
    /// 参数：(期望大小, 实际大小)，返回是否重新下载
    pub on_size_mismatch: Option<fn(u64, u64) -> bool>,
    /// 下载进度回调
    pub callbacks: DownloadCallbacks,
    /// 额外的自定义选项
    pub extras: HashMap<String, ProviderOptionValue>,
}

impl std::fmt::Debug for DownloadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadOptions")
            .field("quality", &self.quality)
            .field("format", &self.format)
            .field("lyric_type", &self.lyric_type)
            .field("lyric_translation", &self.lyric_translation)
            .field("lyric_romanization", &self.lyric_romanization)
            .field("force", &self.force)
            .field("on_size_mismatch", &self.on_size_mismatch.is_some())
            .field("callbacks", &"DownloadCallbacks")
            .field("extras", &self.extras)
            .finish()
    }
}

impl Default for DownloadOptions {
    /// 默认选项
    ///
    /// - 音质：7（较高品质）
    /// - 不下载歌词
    /// - 不强制覆盖
    fn default() -> Self {
        Self {
            quality: 7,
            format: None,
            lyric_type: LyricType::None,
            lyric_translation: false,
            lyric_romanization: false,
            force: false,
            on_size_mismatch: None,
            callbacks: DownloadCallbacks::default(),
            extras: HashMap::new(),
        }
    }
}

impl DownloadOptions {
    /// 创建新的下载选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置音质等级
    pub fn with_quality(mut self, quality: i32) -> Self {
        self.quality = quality;
        self
    }

    /// 设置文件名格式
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// 设置歌词类型
    pub fn with_lyric_type(mut self, lyric_type: LyricType) -> Self {
        self.lyric_type = lyric_type;
        self
    }

    /// 设置是否下载翻译歌词
    pub fn with_lyric_translation(mut self, enable: bool) -> Self {
        self.lyric_translation = enable;
        self
    }

    /// 设置是否下载罗马音歌词
    pub fn with_lyric_romanization(mut self, enable: bool) -> Self {
        self.lyric_romanization = enable;
        self
    }

    /// 设置是否强制重新下载
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// 设置文件大小不匹配时的处理函数
    pub fn with_size_mismatch_handler(mut self, handler: fn(u64, u64) -> bool) -> Self {
        self.on_size_mismatch = Some(handler);
        self
    }

    /// 设置下载回调
    pub fn with_callbacks(mut self, callbacks: DownloadCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// 添加单个额外选项
    pub fn with_extra(
        mut self,
        key: impl Into<String>,
        value: impl Into<ProviderOptionValue>,
    ) -> Self {
        self.extras.insert(key.into(), value.into());
        self
    }

    /// 批量添加额外选项
    pub fn with_extras(mut self, extras: HashMap<String, ProviderOptionValue>) -> Self {
        self.extras.extend(extras);
        self
    }

    /// 获取额外选项
    pub fn get_extra(&self, key: &str) -> Option<&ProviderOptionValue> {
        self.extras.get(key)
    }
}

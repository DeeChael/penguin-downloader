use std::collections::HashMap;
use std::path::PathBuf;

/// 提供者基本信息。
#[derive(Clone, Debug)]
pub enum ProviderInfo {
    /// 音源提供者。
    Music { id: String, version: i64 },
    /// 元数据提供者。
    Metadata { id: String, version: i64 },
}

impl ProviderInfo {
    /// 获取提供者 ID。
    pub fn id(&self) -> &str {
        match self {
            ProviderInfo::Music { id, .. } => id,
            ProviderInfo::Metadata { id, .. } => id,
        }
    }

    /// 获取提供者版本号。
    pub fn version(&self) -> i64 {
        match self {
            ProviderInfo::Music { version, .. } => *version,
            ProviderInfo::Metadata { version, .. } => *version,
        }
    }
}

/// 音质选择策略。
#[derive(Clone, Debug, PartialEq)]
pub enum PreferredQuality {
    /// 选择最高音质。
    Highest,
    /// 选择最低音质。
    Lowest,
    /// 指定特定音质。若歌曲不支持该音质，会自动回退到最接近的可用音质。
    Specific(i64),
}

/// 下载配置选项。
#[derive(Clone, Debug)]
pub struct DownloadOptions {
    /// 优先考虑的音质。
    pub preferred_quality: PreferredQuality,
    /// 文件命名格式（不含拓展名）。
    /// 可用变量：`{track}`、`{disc}`、`{title}`、`{album}`、`{artists}`、`{provider}`。
    pub format: Option<String>,
    /// 选择下载歌词的类型。
    pub lyrics: LyricsType,
    /// 是否下载翻译歌词。
    pub lyrics_translation: bool,
    /// 是否下载罗马音歌词。
    pub lyrics_roma: bool,
    /// 是否强制覆盖已存在的文件。
    pub force: bool,
    /// 文件大小不匹配时的回调。
    /// 参数为（已存在文件大小，歌曲期望大小），返回正数表示重新下载，0 表示视为已存在，负数表示报错。
    pub on_size_mismatch: Option<fn(u64, u64) -> i8>,
}

/// 分页参数。
#[derive(Clone, Debug)]
pub struct Pagination {
    /// 每页记录数（必须大于 0）。
    pub page_size: u64,
    /// 当前页码（通常从 1 开始）。
    pub page: u64,
}

/// 搜索结果。
#[derive(Clone, Debug)]
pub struct SearchResult<T> {
    /// 每页记录数（请求值）。
    pub page_size: u64,
    /// 当前页码。
    pub page: u64,
    /// 匹配查询的总记录数。
    pub total: u64,
    /// 当前页的数据列表。
    pub results: Vec<T>,
}

/// 歌曲信息。
#[derive(Clone, Debug)]
pub struct SongInfo {
    /// 歌曲唯一标识符。
    pub id: String,
    /// 来源音源提供者的 ID。
    pub provider: String,
    /// 歌曲标题。
    pub title: String,
    /// 副标题。
    pub subtitle: Option<String>,
    /// 艺术家列表。
    pub artists: Vec<ArtistInfo>,
    /// 所属专辑。部分情况下 `song_count` 可能为 0，可通过 `list_album_songs` 获取实际数目。
    pub album: Option<AlbumInfo>,
    /// 封面图片 URL。
    pub cover: Option<String>,
    /// 歌曲时长（秒）。
    pub duration: Option<u64>,
    /// 发布日期，格式 `"2023-01-01"`。
    pub published_date: Option<String>,
    /// 歌曲在专辑中的序号。
    pub track_number: Option<String>,
    /// 碟号（多碟专辑）。
    pub disc_number: Option<String>,
    /// 可用音质列表。
    pub qualities: Vec<i64>,
    /// 各音质对应的文件大小。
    pub size: HashMap<i64, u64>,
    /// 额外扩展字段。
    pub extras: HashMap<String, serde_json::Value>,
}

impl std::hash::Hash for SongInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.provider.hash(state);
    }
}

impl PartialEq for SongInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.provider == other.provider
    }
}

impl Eq for SongInfo {}

/// 歌曲引用（通过 ID 或完整信息）。
#[derive(Clone, Debug)]
pub enum SongRef {
    /// 仅通过 ID 引用。
    Id(String),
    /// 通过完整信息引用。
    Info(SongInfo),
}

/// 歌词类型。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LyricsType {
    /// 不下载歌词。
    None,
    /// 普通 LRC 歌词。
    Normal,
    /// 逐字歌词。
    Verbatim,
}

/// 专辑信息。
#[derive(Clone, Debug)]
pub struct AlbumInfo {
    /// 专辑唯一标识符。
    pub id: String,
    /// 来源音源提供者的 ID。
    pub provider: String,
    /// 专辑标题。
    pub title: String,
    /// 副标题。
    pub subtitle: Option<String>,
    /// 封面图片 URL。
    pub cover: Option<String>,
    /// 歌曲数量。
    pub song_count: Option<i32>,
    /// 发布时间。
    pub publish_time: Option<String>,
}

/// 专辑引用（通过 ID 或完整信息）。
#[derive(Clone, Debug)]
pub enum AlbumRef {
    /// 仅通过 ID 引用。
    Id(String),
    /// 通过完整信息引用。
    Info(AlbumInfo),
}

/// 歌单信息。
#[derive(Clone, Debug)]
pub struct PlaylistInfo {
    /// 歌单唯一标识符。
    pub id: String,
    /// 来源音源提供者的 ID。
    pub provider: String,
    /// 歌单标题。
    pub title: String,
    /// 创建者列表。
    pub creator: Vec<String>,
    /// 封面图片 URL。
    pub cover: Option<String>,
    /// 歌曲数量。
    pub song_count: Option<i32>,
    /// 创建时间。
    pub create_time: Option<String>,
}

/// 歌单引用（通过 ID 或完整信息）。
#[derive(Clone, Debug)]
pub enum PlaylistRef {
    /// 仅通过 ID 引用。
    Id(String),
    /// 通过完整信息引用。
    Info(PlaylistInfo),
}

/// 歌词结果。
#[derive(Clone, Debug)]
pub struct LyricsResult {
    /// 普通原文歌词（LRC 格式）。
    pub lrc: Option<String>,
    /// 普通翻译歌词（LRC 格式）。
    pub trans: Option<String>,
    /// 普通罗马音歌词（LRC 格式）。
    pub roma: Option<String>,
    /// 逐字原文歌词。
    pub verbatim: Option<String>,
    /// 逐字翻译歌词。
    pub trans_verbatim: Option<String>,
    /// 逐字罗马音歌词。
    pub roma_verbatim: Option<String>,
}

/// 艺术家信息。
#[derive(Clone, Debug)]
pub struct ArtistInfo {
    /// 艺术家唯一标识符。
    pub id: String,
    /// 来源音源提供者的 ID。
    pub provider: String,
    /// 艺术家名称。
    pub name: String,
}

/// 艺术家引用（通过 ID 或完整信息）。
#[derive(Clone, Debug)]
pub enum ArtistRef {
    /// 仅通过 ID 引用。
    Id(String),
    /// 通过完整信息引用。
    Info(ArtistInfo),
}

/// 下载回调集合。
#[derive(Clone, Copy, Debug)]
pub struct DownloadCallbacks {
    /// 下载开始时的回调。
    pub on_start: Option<fn(&DownloadStart)>,
    /// 下载进度更新的回调。
    pub on_progress: Option<fn(&DownloadProgress)>,
    /// 文件已存在时的回调。
    pub on_existing: Option<fn(&DownloadExisting)>,
    /// 下载完成时的回调。
    pub on_complete: Option<fn(&DownloadComplete)>,
    /// 下载出错时的回调。
    pub on_error: Option<fn(&DownloadError)>,
}

/// 下载开始事件。
#[derive(Clone, Debug)]
pub struct DownloadStart {
    /// 当前下载项序号。
    pub current: i32,
    /// 总下载项数。
    pub total: i32,
    /// 正在下载的歌曲。
    pub song: SongInfo,
    /// 所选音质标签。
    pub quality: Option<String>,
}

/// 下载完成事件。
#[derive(Clone, Debug)]
pub struct DownloadComplete {
    /// 当前下载项序号。
    pub current: i32,
    /// 总下载项数。
    pub total: i32,
    /// 已下载的歌曲。
    pub song: SongInfo,
    /// 所选音质标签。
    pub quality: Option<String>,
    /// 最终文件大小。
    pub final_size: u64,
    /// 文件保存路径。
    pub path: PathBuf,
}

/// 文件已存在事件。
#[derive(Clone, Debug)]
pub struct DownloadExisting {
    /// 当前下载项序号。
    pub current: i32,
    /// 总下载项数。
    pub total: i32,
    /// 歌曲信息。
    pub song: SongInfo,
    /// 文件路径。
    pub path: PathBuf,
    /// 已存在文件的大小。
    pub size: u64,
}

/// 下载进度事件。
#[derive(Clone, Debug)]
pub struct DownloadProgress {
    /// 当前下载项序号。
    pub current: i32,
    /// 总下载项数。
    pub total: i32,
    /// 正在下载的歌曲。
    pub song: SongInfo,
    /// 所选音质标签。
    pub quality: Option<String>,
    /// 已下载的字节数。
    pub downloaded: u64,
    /// 文件总大小（如果已知）。
    pub total_size: Option<u64>,
}

/// 下载错误事件。
#[derive(Clone, Debug)]
pub struct DownloadError {
    /// 当前下载项序号。
    pub current: i32,
    /// 总下载项数。
    pub total: i32,
    /// 出错的歌曲。
    pub song: SongInfo,
    /// 错误描述。
    pub error_message: String,
}

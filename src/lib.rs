//! # Penguin Downloader
//!
//! 一个可扩展的音乐下载库，支持多种音源插件。
//!
//! ## 主要功能
//!
//! - **多音源支持**: 通过插件系统多种音源
//! - **智能元数据**: 自动获取歌曲封面、歌词、专辑信息等元数据
//! - **音质选择**: 支持根据音乐源选择音质
//! - **批量下载**: 支持专辑、歌单批量下载
//! - **自定义输出**: 支持自定义文件名格式和输出目录
//!
//! ## 基本用法
//!
//! ```rust,no_run
//! use penguin_downloader::{PenguinCore, Downloader, DownloadOptions};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 创建核心实例
//!     let core = PenguinCore::new();
//!     
//!     // 加载插件
//!     core.load_plugins_from_dir("./plugins").await?;
//!     
//!     // 获取音源提供者
//!     let provider = core.get_provider("example_provider")
//!         .expect("插件未加载");
//!     
//!     // 创建下载器
//!     let downloader = Downloader::new(provider, None);
//!     
//!     // 搜索歌曲
//!     let result = provider.search_songs("周杰伦", Default::default(), None).await?;
//!     // 下载第一首歌
//!     if let Some(song) = result.songs.first() {
//!         let options = DownloadOptions::default();
//!         downloader.download_song(song, &options).await?;
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod model;
pub mod provider;
pub mod tagger;
pub mod plugin;
pub mod download;
pub mod error;
pub mod core;

pub use model::*;
pub use provider::{
    MusicProvider, ProviderInfo, LoginStatus, QrLoginData, LoginMethod,
    QrLoginCallback, UrlLoginCallback, CodeLoginCallback, Pagination,
    ProviderOptionValue,
};
pub use tagger::{Tagger, TaggerInfo};
pub use plugin::{
    PenguinPlugin, PluginInfo, ProviderOption, ProviderOptionType,
    PluginRegistry,
};
pub use download::Downloader;
pub use error::{Error, Result};
pub use core::PenguinCore;
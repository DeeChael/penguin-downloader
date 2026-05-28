//! `penguin-downloader` — 一个音乐服务访问库。
//!
//! 通过 [`PenguinCore`] 使用此库的全部 API，包括音源提供者的注册、查询
//! 以及通过 [`PenguinDownloader`] 进行歌曲下载。

pub mod core;
pub mod downloader;
pub mod error;
pub mod login;
pub mod models;
pub mod traits;

pub use core::PenguinCore;
pub use downloader::PenguinDownloader;
pub use error::CoreError;
pub use login::*;
pub use models::*;
pub use traits::*;

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
    load_plugins, get_provider, list_provider_names, register_provider,
    get_tagger, list_tagger_names, register_tagger
};
pub use download::Downloader;
pub use error::{Error, Result};
pub use core::PenguinCore;
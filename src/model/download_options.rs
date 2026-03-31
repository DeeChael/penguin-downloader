use std::collections::HashMap;

use super::download_callbacks::DownloadCallbacks;
use super::lyric_type::LyricType;
use crate::provider::ProviderOptionValue;

#[derive(Clone)]
pub struct DownloadOptions {
    pub quality: i32,
    pub format: Option<String>,
    pub lyric_type: LyricType,
    pub lyric_translation: bool,
    pub lyric_romanization: bool,
    pub force: bool,
    pub on_size_mismatch: Option<fn(u64, u64) -> bool>,
    pub callbacks: DownloadCallbacks,
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

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_quality(mut self, quality: i32) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn with_lyric_type(mut self, lyric_type: LyricType) -> Self {
        self.lyric_type = lyric_type;
        self
    }

    pub fn with_lyric_translation(mut self, enable: bool) -> Self {
        self.lyric_translation = enable;
        self
    }

    pub fn with_lyric_romanization(mut self, enable: bool) -> Self {
        self.lyric_romanization = enable;
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn with_size_mismatch_handler(mut self, handler: fn(u64, u64) -> bool) -> Self {
        self.on_size_mismatch = Some(handler);
        self
    }

    pub fn with_callbacks(mut self, callbacks: DownloadCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn with_extra(
        mut self,
        key: impl Into<String>,
        value: impl Into<ProviderOptionValue>,
    ) -> Self {
        self.extras.insert(key.into(), value.into());
        self
    }

    pub fn with_extras(mut self, extras: HashMap<String, ProviderOptionValue>) -> Self {
        self.extras.extend(extras);
        self
    }

    pub fn get_extra(&self, key: &str) -> Option<&ProviderOptionValue> {
        self.extras.get(key)
    }

}

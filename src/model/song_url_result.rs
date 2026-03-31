use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongUrlResult {
    pub url: Option<String>,
    pub quality: Option<String>,
    #[serde(default)]
    pub encrypted: bool,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub extras: HashMap<String, serde_json::Value>,
}

impl SongUrlResult {
    pub fn success(url: impl Into<String>, quality: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            quality: Some(quality.into()),
            encrypted: false,
            error_message: None,
            extras: HashMap::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            url: None,
            quality: None,
            encrypted: false,
            error_message: Some(message.into()),
            extras: HashMap::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.url.is_some() && !self.url.as_ref().unwrap().is_empty()
    }
}

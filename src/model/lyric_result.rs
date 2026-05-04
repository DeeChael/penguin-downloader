use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricResult {
    #[serde(default)]
    pub lrc: Option<String>,
    #[serde(default)]
    pub verbatim: Option<String>,
    #[serde(default)]
    pub trans: Option<String>,
    #[serde(default)]
    pub roma: Option<String>,
    #[serde(default)]
    pub trans_verbatim: Option<String>,
    #[serde(default)]
    pub roma_verbatim: Option<String>,
}

impl LyricResult {
    pub fn new() -> Self {
        Self {
            lrc: None,
            verbatim: None,
            trans: None,
            roma: None,
            trans_verbatim: None,
            roma_verbatim: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lrc.is_none()
            && self.verbatim.is_none()
            && self.trans.is_none()
            && self.roma.is_none()
            && self.trans_verbatim.is_none()
            && self.roma_verbatim.is_none()
    }

    pub fn with_lrc(mut self, lrc: impl Into<String>) -> Self {
        self.lrc = Some(lrc.into());
        self
    }

    pub fn with_verbatim(mut self, verbatim: impl Into<String>) -> Self {
        self.verbatim = Some(verbatim.into());
        self
    }

    pub fn with_trans(mut self, trans: impl Into<String>) -> Self {
        self.trans = Some(trans.into());
        self
    }

    pub fn with_roma(mut self, roma: impl Into<String>) -> Self {
        self.roma = Some(roma.into());
        self
    }

    pub fn with_trans_verbatim(mut self, trans_verbatim: impl Into<String>) -> Self {
        self.trans_verbatim = Some(trans_verbatim.into());
        self
    }

    pub fn with_roma_verbatim(mut self, roma_verbatim: impl Into<String>) -> Self {
        self.roma_verbatim = Some(roma_verbatim.into());
        self
    }
}

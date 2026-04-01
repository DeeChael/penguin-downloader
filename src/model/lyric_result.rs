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
}

impl LyricResult {
    pub fn new() -> Self {
        Self {
            lrc: None,
            verbatim: None,
            trans: None,
            roma: None,
        }
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
}

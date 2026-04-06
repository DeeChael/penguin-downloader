use async_trait::async_trait;

use crate::model::{MetadataInfo, SongInfo};

#[derive(Debug, Clone)]
pub struct TaggerInfo {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
}

impl TaggerInfo {
    pub fn new(
        id: impl Into<String>,
        plugin_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            name: name.into(),
        }
    }
}

#[async_trait]
pub trait Tagger: Send + Sync {

    fn info(&self) -> TaggerInfo;

    async fn fetch_metadata(
        &self,
        song_info: &SongInfo,
    ) -> Option<MetadataInfo>;

}

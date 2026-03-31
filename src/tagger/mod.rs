mod tagger_info;

pub use tagger_info::TaggerInfo;

use async_trait::async_trait;

use crate::model::{MetadataInfo, SongInfo};

#[async_trait]
pub trait Tagger: Send + Sync {

    fn info(&self) -> TaggerInfo;

    async fn fetch_metadata(
        &self,
        song_info: &SongInfo,
    ) -> Option<MetadataInfo>;

}
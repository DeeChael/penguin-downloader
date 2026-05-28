use std::collections::HashMap;
use std::sync::Arc;

use crate::downloader::PenguinDownloader;
use crate::error::CoreError;
use crate::traits::{MetadataProvider, MusicProvider};

pub type Result<T> = std::result::Result<T, CoreError>;

pub struct PenguinCore {
    music_providers: HashMap<String, Arc<dyn MusicProvider>>,
    metadata_providers: HashMap<String, Arc<dyn MetadataProvider>>,
}

impl PenguinCore {
    pub fn new() -> Self {
        Self {
            music_providers: HashMap::new(),
            metadata_providers: HashMap::new(),
        }
    }

    pub fn get_version(&self) -> i64 {
        4000000
    }

    pub fn list_music_providers(&self) -> Vec<Arc<dyn MusicProvider>> {
        self.music_providers.values().cloned().collect()
    }

    pub fn list_metadata_providers(&self) -> Vec<Arc<dyn MetadataProvider>> {
        self.metadata_providers.values().cloned().collect()
    }

    pub fn get_music_provider(&self, id: &str) -> Option<Arc<dyn MusicProvider>> {
        self.music_providers.get(id).cloned()
    }

    pub fn get_metadata_provider(&self, id: &str) -> Option<Arc<dyn MetadataProvider>> {
        self.metadata_providers.get(id).cloned()
    }

    pub fn register_music_provider(
        &mut self,
        provider: Arc<dyn MusicProvider>,
    ) -> Result<()> {
        let info = provider.info()?;
        let id = info.id().to_string();
        if self.music_providers.contains_key(&id)
            || self.metadata_providers.contains_key(&id)
        {
            return Err(CoreError::ProviderAlreadyExists(id));
        }
        self.music_providers.insert(id, provider);
        Ok(())
    }

    pub fn register_metadata_provider(
        &mut self,
        provider: Arc<dyn MetadataProvider>,
    ) -> Result<()> {
        let info = provider.info();
        let id = info.id().to_string();
        if self.music_providers.contains_key(&id)
            || self.metadata_providers.contains_key(&id)
        {
            return Err(CoreError::ProviderAlreadyExists(id));
        }
        self.metadata_providers.insert(id, provider);
        Ok(())
    }

    pub fn create_downloader(
        &self,
        music_provider: Arc<dyn MusicProvider>,
        credential: Option<&str>,
    ) -> PenguinDownloader {
        PenguinDownloader::new(
            music_provider,
            credential.map(|s| s.to_string()),
        )
    }
}

impl Default for PenguinCore {
    fn default() -> Self {
        Self::new()
    }
}

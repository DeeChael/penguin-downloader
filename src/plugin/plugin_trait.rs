use async_trait::async_trait;
use std::collections::HashMap;

use crate::provider::{MusicProvider, ProviderOptionValue};
use crate::tagger::Tagger;
use super::PluginInfo;

#[async_trait]
pub trait PenguinPlugin: Send + Sync {

    fn info(&self) -> PluginInfo;

    async fn init(
        &mut self,
        config: HashMap<String, ProviderOptionValue>,
    ) -> crate::Result<()> {
        let _ = config;
        Ok(())
    }

    fn register_providers(
        &self,
    ) -> Vec<(String, std::sync::Arc<dyn MusicProvider>)> {
        Vec::new()
    }

    fn register_taggers(
        &self,
    ) -> Vec<(String, std::sync::Arc<dyn Tagger>)> {
        Vec::new()
    }

    fn shutdown(&self) {}

}

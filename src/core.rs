use std::path::Path;
use std::sync::Arc;

use tracing::{info, warn};

use crate::download::Downloader;
use crate::plugin::PluginRegistry;
use crate::provider::MusicProvider;

pub struct PenguinCore {
    registry: PluginRegistry,
}

impl PenguinCore {
    pub fn new() -> Self {
        info!("[核心] 创建 PenguinCore");
        
        Self {
            registry: PluginRegistry::new(),
        }
    }

    pub async fn load_plugins_from_dir(&self, plugins_dir: impl AsRef<Path>) -> crate::Result<()> {
        let plugins_dir = plugins_dir.as_ref();
        info!("[核心] 从目录加载插件: {:?}", plugins_dir);

        std::fs::create_dir_all(plugins_dir)?;

        self.registry.load_plugins(plugins_dir).await?;
        
        info!("[核心] 插件加载完成");
        Ok(())
    }

    pub async fn load_plugin_from_file(&self, file_path: impl AsRef<Path>) -> crate::Result<()> {
        let file_path = file_path.as_ref();
        info!("[核心] 从文件加载插件: {:?}", file_path);
        
        self.registry.load_plugin_from_file(file_path).await?;
        
        info!("[核心] 插件加载完成: {:?}", file_path);
        Ok(())
    }

    pub async fn refresh_provider(
        &self,
        name: &str,
        credential: &str,
        timeout: std::time::Duration,
    ) -> crate::Result<String> {
        let provider = self.get_provider(name)
            .ok_or_else(|| crate::Error::ProviderNotFound(name.to_string()))?;
        
        provider.refresh_and_validate(credential, timeout).await
    }

    pub fn get_downloader(&self, provider: Arc<dyn MusicProvider>, credential: Option<String>) -> Downloader {
        info!("[下载] 创建下载器: provider={}", provider.name());
        Downloader::new(provider, credential)
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn MusicProvider>> {
        info!("[核心] 获取音源: name={}", name);
        let provider = self.registry.get_provider(name);
        if provider.is_none() {
            warn!("[核心] 音源未找到: name={}", name);
        }
        provider
    }

    pub fn list_provider_names(&self) -> Vec<String> {
        let names = self.registry.list_provider_names();
        info!("[核心] 可用音源列表: {:?}", names);
        names
    }

    pub fn register_provider(&self, name: &str, provider: Arc<dyn MusicProvider>) {
        info!("[核心] 注册音源: name={}", name);
        self.registry.register(name, provider);
    }

    pub fn list_providers(&self) -> Vec<(String, Arc<dyn MusicProvider>)> {
        self.registry.list_providers()
    }
}

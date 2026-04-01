use std::path::Path;
use std::sync::Arc;

use tracing::{info, warn};

use crate::download::Downloader;
use crate::plugin::PluginRegistry;
use crate::provider::MusicProvider;

/// Penguin Downloader 核心结构
///
/// 这是库的主要入口点，负责管理插件注册表和提供核心功能。
/// 每个 Core 实例都有自己的插件注册表，实例之间相互独立。
pub struct PenguinCore {
    registry: PluginRegistry,
}

impl PenguinCore {
    /// 创建一个新的 PenguinCore 实例
    ///
    /// # Examples
    ///
    /// ```rust
    /// use penguin_downloader::PenguinCore;
    ///
    /// let core = PenguinCore::new();
    /// ```
    pub fn new() -> Self {
        info!("[核心] 创建 PenguinCore");
        
        Self {
            registry: PluginRegistry::new(),
        }
    }

    /// 从指定目录加载所有插件
    ///
    /// 扫描目录中的动态库文件（.dll/.so/.dylib）并加载为插件。
    /// 如果目录不存在，会自动创建。
    ///
    /// # Arguments
    ///
    /// * `plugins_dir` - 插件目录路径
    ///
    /// # Errors
    ///
    /// 如果插件加载失败，会返回错误但继续加载其他插件
    pub async fn load_plugins_from_dir(&self, plugins_dir: impl AsRef<Path>) -> crate::Result<()> {
        let plugins_dir = plugins_dir.as_ref();
        info!("[核心] 从目录加载插件: {:?}", plugins_dir);

        std::fs::create_dir_all(plugins_dir)?;

        self.registry.load_plugins(plugins_dir).await?;
        
        info!("[核心] 插件加载完成");
        Ok(())
    }

    /// 从单个文件加载插件
    ///
    /// # Arguments
    ///
    /// * `file_path` - 插件文件路径
    pub async fn load_plugin_from_file(&self, file_path: impl AsRef<Path>) -> crate::Result<()> {
        let file_path = file_path.as_ref();
        info!("[核心] 从文件加载插件: {:?}", file_path);
        
        self.registry.load_plugin_from_file(file_path).await?;
        
        info!("[核心] 插件加载完成: {:?}", file_path);
        Ok(())
    }

    /// 刷新并验证登录凭证
    ///
    /// # Arguments
    ///
    /// * `name` - 音源名称
    /// * `credential` - 当前凭证
    /// * `timeout` - 超时时间
    ///
    /// # Returns
    ///
    /// 返回新的凭证字符串（base64 编码）
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

    /// 创建下载器
    ///
    /// # Arguments
    ///
    /// * `provider` - 音源提供者
    /// * `credential` - 可选的登录凭证
    ///
    /// # Returns
    ///
    /// 返回配置好的 Downloader 实例
    pub fn get_downloader(&self, provider: Arc<dyn MusicProvider>, credential: Option<String>) -> Downloader {
        info!("[下载] 创建下载器: provider={}", provider.name());
        Downloader::new(provider, credential)
    }

    /// 获取指定名称的音源提供者
    ///
    /// # Arguments
    ///
    /// * `name` - 音源 ID
    ///
    /// # Returns
    ///
    /// 如果找到返回 Some(Arc<dyn MusicProvider>)，否则返回 None
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn MusicProvider>> {
        info!("[核心] 获取音源: name={}", name);
        let provider = self.registry.get_provider(name);
        if provider.is_none() {
            warn!("[核心] 音源未找到: name={}", name);
        }
        provider
    }

    /// 列出所有可用的音源名称
    pub fn list_provider_names(&self) -> Vec<String> {
        let names = self.registry.list_provider_names();
        info!("[核心] 可用音源列表: {:?}", names);
        names
    }

    /// 注册自定义音源提供者
    pub fn register_provider(&self, name: &str, provider: Arc<dyn MusicProvider>) {
        info!("[核心] 注册音源: name={}", name);
        self.registry.register(name, provider);
    }

    /// 列出所有音源提供者及其名称
    pub fn list_providers(&self) -> Vec<(String, Arc<dyn MusicProvider>)> {
        self.registry.list_providers()
    }
}

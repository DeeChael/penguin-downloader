use std::collections::HashMap;
use std::sync::Arc;

use crate::downloader::PenguinDownloader;
use crate::error::CoreError;
use crate::traits::{MetadataProvider, MusicProvider};

/// 库内部使用的 Result 类型。
pub type Result<T> = std::result::Result<T, CoreError>;

/// penguin-downloader 的核心入口。
///
/// 管理音源提供者和元数据提供者的注册与查询，并可用已注册的提供者创建下载器。
pub struct PenguinCore {
    music_providers: HashMap<String, Arc<dyn MusicProvider>>,
    metadata_providers: HashMap<String, Arc<dyn MetadataProvider>>,
}

impl PenguinCore {
    /// 创建一个新的 `PenguinCore` 实例。
    pub fn new() -> Self {
        Self {
            music_providers: HashMap::new(),
            metadata_providers: HashMap::new(),
        }
    }

    /// 获取当前 penguin-downloader API 版本号。
    ///
    /// 版本号命名规则：x.y.z 对应数字 xyyyzzz。
    /// 例如 4.0.0 对应 4000000。
    pub fn get_version(&self) -> i64 {
        4000001
    }

    /// 列出所有已注册的音源提供者。
    pub fn list_music_providers(&self) -> Vec<Arc<dyn MusicProvider>> {
        self.music_providers.values().cloned().collect()
    }

    /// 列出所有已注册的元数据提供者。
    pub fn list_metadata_providers(&self) -> Vec<Arc<dyn MetadataProvider>> {
        self.metadata_providers.values().cloned().collect()
    }

    /// 根据 ID 获取音源提供者。
    pub fn get_music_provider(&self, id: &str) -> Option<Arc<dyn MusicProvider>> {
        self.music_providers.get(id).cloned()
    }

    /// 根据 ID 获取元数据提供者。
    pub fn get_metadata_provider(&self, id: &str) -> Option<Arc<dyn MetadataProvider>> {
        self.metadata_providers.get(id).cloned()
    }

    /// 注册一个新的音源提供者。
    ///
    /// 音源提供者和元数据提供者的 ID 不能互相重复。
    /// 如果 ID 已存在，返回 [`CoreError::ProviderAlreadyExists`]。
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

    /// 注册一个新的元数据提供者。
    ///
    /// 音源提供者和元数据提供者的 ID 不能互相重复。
    /// 如果 ID 已存在，返回 [`CoreError::ProviderAlreadyExists`]。
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

    /// 使用指定的音源提供者和可选凭据创建下载器。
    ///
    /// 如果提供了 `credential`，下载器会在后续调用中将其传递给 `MusicProvider` 的相应方法。
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

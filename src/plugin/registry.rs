use std::path::Path;
use std::sync::Arc;

use dashmap::DashMap;
use libloading::{Library, Symbol};
use tracing::{info, warn};

use super::PluginInfo;
use crate::error::{Error, Result};
use crate::provider::MusicProvider;
use crate::tagger::Tagger;

pub struct LoadedPlugin {
    pub info: PluginInfo,
    pub library: Library,
}

pub struct PluginRegistry {
    providers: DashMap<String, Arc<dyn MusicProvider>>,
    taggers: DashMap<String, Arc<dyn Tagger>>,
    loaded_plugins: DashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            taggers: DashMap::new(),
            loaded_plugins: DashMap::new(),
        }
    }

    pub fn register(&self, name: &str, provider: Arc<dyn MusicProvider>) {
        info!("[注册] 注册音源: name={}", name);
        self.providers.insert(name.to_lowercase(), provider);
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn MusicProvider>> {
        self.providers.get(&name.to_lowercase()).map(|p| p.clone())
    }

    pub fn list_provider_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.providers.iter().map(|e| e.key().clone()).collect();
        names.sort();
        names
    }

    pub fn list_providers(&self) -> Vec<(String, Arc<dyn MusicProvider>)> {
        self.providers
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub fn register_tagger(&self, name: &str, tagger: Arc<dyn Tagger>) {
        info!("[注册] 注册 tagger: name={}", name);
        self.taggers.insert(name.to_lowercase(), tagger);
    }

    pub fn get_tagger(&self, name: &str) -> Option<Arc<dyn Tagger>> {
        self.taggers.get(&name.to_lowercase()).map(|t| t.clone())
    }

    pub fn list_tagger_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.taggers.iter().map(|e| e.key().clone()).collect();
        names.sort();
        names
    }

    pub fn list_taggers(&self) -> Vec<(String, Arc<dyn Tagger>)> {
        self.taggers
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub async fn load_plugin_from_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<()> {
        let path = file_path.as_ref();
        let lib_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        match self.load_plugin(path, lib_name).await {
            Ok(_) => {
                info!("Loaded plugin from {:?}", path);
                Ok(())
            }
            Err(e) => {
                warn!("[注册] 加载插件失败: path={:?}, error={}", path, e);
                Err(e)
            }
        }
    }

    pub async fn load_plugins<P: AsRef<Path>>(&self,
        plugins_dir: P,
    ) -> Result<()> {
        let plugins_dir = plugins_dir.as_ref();

        if !plugins_dir.exists() {
            info!("[注册] 创建插件目录: {:?}", plugins_dir);
            std::fs::create_dir_all(plugins_dir)?;
        }

        #[cfg(target_os = "windows")]
        const LIB_EXTENSION: &str = "dll";
        #[cfg(target_os = "linux")]
        const LIB_EXTENSION: &str = "so";
        #[cfg(target_os = "macos")]
        const LIB_EXTENSION: &str = "dylib";

        let mut plugin_count = 0;

        for entry in std::fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == LIB_EXTENSION {
                        let lib_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown");

                        match self.load_plugin(&path, lib_name).await {
                            Ok(_) => {
                                plugin_count += 1;
                            }
                            Err(e) => {
                                warn!("[注册] 加载插件失败: path={:?}, error={}", path, e);
                            }
                        }
                    }
                }
            }
        }

        if plugin_count == 0 {
            warn!(
                "No plugins found in {:?} (looking for .{} files)",
                plugins_dir, LIB_EXTENSION
            );
        } else {
            info!("Successfully loaded {} plugin(s)", plugin_count);
        }

        Ok(())
    }

    async fn load_plugin(
        &self,
        path: &Path,
        _lib_name: &str,
    ) -> Result<()> {
        unsafe {
            let library = Library::new(path).map_err(|e| {
                Error::PluginLoad(format!("Failed to load library {:?}: {}", path, e))
            })?;

            type CreatePluginFn = unsafe extern "C" fn() -> *mut dyn super::PenguinPlugin;
            let create_fn: Symbol<CreatePluginFn> =
                library.get(b"penguin_plugin_create").map_err(|e| {
                    Error::PluginLoad(format!(
                        "Failed to find 'penguin_plugin_create' symbol in {:?}: {}. Make sure the plugin implements the PenguinPlugin trait.",
                        path, e
                    ))
                })?;

            info!(
                "Loading plugin using PenguinPlugin interface from {:?}",
                path
            );
            let plugin_ptr = create_fn();
            if plugin_ptr.is_null() {
                return Err(Error::PluginLoad(format!(
                    "Plugin creation function returned null"
                )));
            }

            let plugin: Box<dyn super::PenguinPlugin> = Box::from_raw(plugin_ptr);
            let plugin_info = plugin.info();
            let plugin_id = plugin_info.id.clone();

            let providers = plugin.register_providers();

            for (_provider_id, provider) in providers {
                let info = provider.info();
                info!(
                    "Registering provider '{}' from plugin '{}'",
                    info.id, plugin_id
                );
                self.register(&info.id, provider);
            }

            let taggers = plugin.register_taggers();
            for (tagger_name, tagger) in taggers {
                info!(
                    "Registering tagger '{}' from plugin '{}'",
                    tagger_name, plugin_id
                );
                self.register_tagger(&tagger_name, tagger);
            }

            self.loaded_plugins.insert(
                plugin_id.clone(),
                LoadedPlugin {
                    info: plugin_info,
                    library,
                },
            );

            Ok(())
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
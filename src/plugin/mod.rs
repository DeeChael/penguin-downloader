mod plugin_info;
mod plugin_trait;
mod registry;

pub use plugin_info::{PluginInfo, ProviderOption, ProviderOptionType};
pub use plugin_trait::PenguinPlugin;
pub use registry::PluginRegistry;
pub use registry::{get_provider, list_provider_names, load_plugins, register_provider, get_tagger, list_tagger_names, register_tagger};
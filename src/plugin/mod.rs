mod plugin_info;
mod plugin_trait;
mod registry;

pub use plugin_info::{PluginInfo, ProviderOption, ProviderOptionType};
pub use plugin_trait::PenguinPlugin;
pub use registry::PluginRegistry;
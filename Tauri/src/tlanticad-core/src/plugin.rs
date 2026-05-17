//! S8: Plugin system for TlantiCAD
//!
//! Trait-based plugin architecture for loadable modules.

use crate::{Module, Result};
use hashbrown::HashMap;

/// Plugin trait — all modules implement this
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn module(&self) -> Module;
    fn init(&self) -> Result<()>;
    fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// Plugin registry — manages loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        tracing::info!("Registering plugin: {} v{}", name, plugin.version());
        plugin.init()?;
        self.plugins.insert(name, plugin);
        Ok(())
    }

    pub fn unregister(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            tracing::info!("Unregistering plugin: {}", name);
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|k| k.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

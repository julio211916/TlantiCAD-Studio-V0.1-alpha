//! Configuration loader and manager

use crate::AppConfig;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Configuration manager with caching
pub struct ConfigManager {
    config: RwLock<AppConfig>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create new config manager
    pub async fn new(config_path: impl Into<PathBuf>) -> tlanticad_core::Result<Self> {
        let config_path = config_path.into();
        let config = AppConfig::load(&config_path).await?;
        
        Ok(Self {
            config: RwLock::new(config),
            config_path,
        })
    }
    
    /// Get current config (read-only)
    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }
    
    /// Update config
    pub async fn update<F>(&self, f: F) -> tlanticad_core::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config = self.config.write().await;
            f(&mut config);
        }
        self.save().await
    }
    
    /// Save config to disk
    pub async fn save(&self) -> tlanticad_core::Result<()> {
        let config = self.config.read().await.clone();
        config.save(&self.config_path).await
    }
    
    /// Reload config from disk
    pub async fn reload(&self) -> tlanticad_core::Result<()> {
        let config = AppConfig::load(&self.config_path).await?;
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }
    
    /// Reset to defaults
    pub async fn reset(&self) -> tlanticad_core::Result<()> {
        let default = AppConfig::default();
        {
            let mut config = self.config.write().await;
            *config = default;
        }
        self.save().await
    }
}

/// Get default config directory
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TlantiCAD")
}

/// Get default config file path
pub fn default_config_path() -> PathBuf {
    default_config_dir().join("config.json")
}

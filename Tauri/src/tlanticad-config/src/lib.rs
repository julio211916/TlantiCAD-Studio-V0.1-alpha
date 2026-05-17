//! TlantiCAD Configuration Module
//! 
//! Reemplaza todos los archivos XML de Exocad con configuración en Rust puro
//! - buttons.xml -> ButtonConfig
//! - defaultparameters.xml -> DefaultParameters  
//! - defaultsettings.xml -> Settings
//! - colors.xml -> ColorConfig
//! - wizard.xml -> WorkflowConfig

pub mod buttons;
pub mod parameters;
pub mod settings;
pub mod colors;
pub mod workflow;
pub mod loader;

pub use buttons::*;
pub use parameters::*;
pub use settings::*;
pub use colors::*;
pub use workflow::*;
pub use loader::*;

use std::path::Path;
use tokio::fs;

/// Configuración completa del sistema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub buttons: ButtonConfig,
    pub parameters: DefaultParameters,
    pub settings: Settings,
    pub colors: ColorConfig,
    pub workflow: WorkflowConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            buttons: ButtonConfig::default(),
            parameters: DefaultParameters::default(),
            settings: Settings::default(),
            colors: ColorConfig::default(),
            workflow: WorkflowConfig::default(),
        }
    }
}

impl AppConfig {
    /// Cargar desde archivo JSON
    pub async fn load(path: impl AsRef<Path>) -> tlanticad_core::Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            let config = Self::default();
            config.save(path).await?;
            return Ok(config);
        }

        let content = fs::read_to_string(path).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;
        
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| tlanticad_core::TlantiError::Serialization(e))?;
        
        Ok(config)
    }

    /// Guardar a archivo JSON
    pub async fn save(&self, path: impl AsRef<Path>) -> tlanticad_core::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| tlanticad_core::TlantiError::Serialization(e))?;
        
        fs::write(path, content).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let cfg = AppConfig::default();
        assert!(!cfg.version.is_empty());
    }

    #[test]
    fn test_button_config_default() {
        let bc = ButtonConfig::default();
        assert!(!bc.buttons.is_empty());
    }

    #[test]
    fn test_default_parameters() {
        let p = DefaultParameters::default();
        assert!((p.crown.min_thickness - 0.4).abs() < 1e-6);
        assert!((p.crown.cement_gap - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_settings_default() {
        let s = Settings::default();
        assert!(!s.application.language.is_empty());
    }

    #[test]
    fn test_color_config_default() {
        let c = ColorConfig::default();
        assert!(!c.menu_background_selected.is_empty());
    }

    #[test]
    fn test_workflow_config_default() {
        let w = WorkflowConfig::default();
        assert!(!w.workflows.is_empty());
    }

    #[test]
    fn test_app_config_serialize_roundtrip() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, cfg.version);
    }

    #[tokio::test]
    async fn test_load_creates_default() {
        let dir = std::env::temp_dir().join("tlanticad_config_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_config.json");
        let _ = std::fs::remove_file(&path);
        let cfg = AppConfig::load(&path).await.unwrap();
        assert!(!cfg.version.is_empty());
        let _ = std::fs::remove_file(&path);
    }
}

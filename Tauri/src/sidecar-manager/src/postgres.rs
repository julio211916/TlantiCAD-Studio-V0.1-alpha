//! PostgreSQL sidecar management

use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use crate::{Result, SidecarConfig, SidecarManager};

/// PostgreSQL sidecar helper
pub struct PostgresSidecar;

impl PostgresSidecar {
    /// Create PostgreSQL sidecar configuration
    pub fn create_config(
        data_dir: PathBuf,
        port: u16,
        postgres_bin: Option<PathBuf>,
    ) -> SidecarConfig {
        let postgres_bin = postgres_bin.unwrap_or_else(|| PathBuf::from("postgres"));

        let mut env_vars = HashMap::new();
        env_vars.insert("PGDATA".to_string(), data_dir.to_string_lossy().to_string());

        SidecarConfig {
            name: "postgresql".to_string(),
            executable: postgres_bin,
            args: vec!["-p".to_string(), port.to_string()],
            working_dir: Some(data_dir.clone()),
            env_vars,
            port: Some(port),
            health_check_url: None,
            auto_restart: true,
            restart_delay_ms: 5000,
        }
    }

    /// Initialize PostgreSQL data directory
    pub fn init_db(data_dir: &PathBuf, initdb_bin: Option<PathBuf>) -> Result<()> {
        use std::process::Command;

        if data_dir.join("PG_VERSION").exists() {
            info!("PostgreSQL data directory already initialized");
            return Ok(());
        }

        std::fs::create_dir_all(data_dir)?;

        let initdb_bin = initdb_bin.unwrap_or_else(|| PathBuf::from("initdb"));

        let output = Command::new(initdb_bin)
            .arg("-D")
            .arg(data_dir)
            .arg("-E")
            .arg("UTF8")
            .arg("--no-locale")
            .output()?;

        if !output.status.success() {
            return Err(crate::SidecarError::StartError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        info!("PostgreSQL data directory initialized: {:?}", data_dir);
        Ok(())
    }

    /// Register PostgreSQL with the sidecar manager
    pub async fn register_with_manager(
        manager: &SidecarManager,
        data_dir: PathBuf,
        port: u16,
    ) -> Result<()> {
        // Initialize DB if needed
        Self::init_db(&data_dir, None)?;

        // Register configuration
        let config = Self::create_config(data_dir, port, None);
        manager.register(config).await;

        Ok(())
    }
}

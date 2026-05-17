//! Model management

use app_core::types::{EntityId, MlModel, MlModelType};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{MlError, Result};

/// Model registry for managing loaded models
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<EntityId, LoadedModel>>>,
    models_dir: PathBuf,
}

/// A loaded model with its metadata
pub struct LoadedModel {
    pub metadata: MlModel,
    pub session: Session,
}

impl ModelRegistry {
    pub fn new(models_dir: impl Into<PathBuf>) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            models_dir: models_dir.into(),
        }
    }

    /// Load a model from file
    pub async fn load_model(&self, path: &Path) -> Result<EntityId> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let model_type = match extension.to_lowercase().as_str() {
            "onnx" => MlModelType::Onnx,
            "pt" | "pth" => MlModelType::PyTorch,
            _ => MlModelType::Custom(extension.to_string()),
        };

        info!("Loading model: {} ({:?})", name, model_type);

        // Create ONNX Runtime session using ort 2.0 API
        let session = Session::builder()
            .map_err(|e| MlError::OnnxError(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| MlError::OnnxError(e.to_string()))?
            .commit_from_file(path)
            .map_err(|e| MlError::LoadError(e.to_string()))?;

        // Get input/output shapes - simplified for now
        let input_shape: Vec<i64> = Vec::new();
        let output_shape: Vec<i64> = Vec::new();

        let id = Uuid::new_v4();
        let metadata = MlModel {
            id,
            name,
            model_type,
            path: path.to_string_lossy().to_string(),
            input_shape,
            output_shape,
        };

        let loaded = LoadedModel { metadata, session };

        {
            let mut guard = self.models.write().await;
            guard.insert(id, loaded);
        }

        info!("Model loaded with ID: {}", id);
        Ok(id)
    }

    /// Get a loaded model
    pub async fn get_model(&self, id: EntityId) -> Option<Arc<RwLock<HashMap<EntityId, LoadedModel>>>> {
        let guard = self.models.read().await;
        if guard.contains_key(&id) {
            Some(self.models.clone())
        } else {
            None
        }
    }

    /// List all loaded models
    pub async fn list_models(&self) -> Vec<MlModel> {
        let guard = self.models.read().await;
        guard.values().map(|m| m.metadata.clone()).collect()
    }

    /// Unload a model
    pub async fn unload_model(&self, id: EntityId) -> Result<()> {
        let mut guard = self.models.write().await;
        guard
            .remove(&id)
            .ok_or_else(|| MlError::ModelNotFound(id.to_string()))?;
        
        info!("Model unloaded: {}", id);
        Ok(())
    }

    /// Scan models directory and return available models
    pub fn scan_available_models(&self) -> Result<Vec<PathBuf>> {
        let mut models = Vec::new();

        if !self.models_dir.exists() {
            return Ok(models);
        }

        for entry in std::fs::read_dir(&self.models_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext.to_lowercase().as_str(), "onnx" | "pt" | "pth") {
                    models.push(path);
                }
            }
        }

        Ok(models)
    }
}

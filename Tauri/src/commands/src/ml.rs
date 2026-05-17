//! ML-related Tauri commands

use ml_runtime::InferenceEngine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::{CommandError, CommandResult};

/// ML state managed by Tauri
pub struct MlState {
    pub engine: Arc<RwLock<InferenceEngine>>,
}

/// Model info for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub input_shape: Vec<i64>,
    pub output_shape: Vec<i64>,
}

/// Inference result for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResultPayload {
    pub output: Vec<f32>,
    pub shape: Vec<usize>,
    pub duration_ms: u64,
}

/// Load an ML model
#[tauri::command]
pub async fn load_model(
    state: State<'_, MlState>,
    path: String,
) -> CommandResult<ModelInfo> {
    let engine = state.engine.read().await;
    let path = PathBuf::from(&path);
    
    let id = engine.registry().load_model(&path).await?;

    let models = engine.registry().list_models().await;
    let model = models.iter().find(|m| m.id == id)
        .ok_or_else(|| CommandError {
            code: "MODEL_NOT_FOUND".to_string(),
            message: "Model not found after loading".to_string(),
        })?;

    Ok(ModelInfo {
        id: model.id.to_string(),
        name: model.name.clone(),
        model_type: format!("{:?}", model.model_type),
        input_shape: model.input_shape.clone(),
        output_shape: model.output_shape.clone(),
    })
}

/// List loaded models
#[tauri::command]
pub async fn list_models(
    state: State<'_, MlState>,
) -> CommandResult<Vec<ModelInfo>> {
    let engine = state.engine.read().await;
    let models = engine.registry().list_models().await;

    Ok(models.into_iter().map(|m| ModelInfo {
        id: m.id.to_string(),
        name: m.name,
        model_type: format!("{:?}", m.model_type),
        input_shape: m.input_shape,
        output_shape: m.output_shape,
    }).collect())
}

/// Unload a model
#[tauri::command]
pub async fn unload_model(
    state: State<'_, MlState>,
    id: String,
) -> CommandResult<()> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let engine = state.engine.read().await;
    engine.registry().unload_model(uuid).await?;

    Ok(())
}

/// Run inference on a model
#[tauri::command]
pub async fn run_inference(
    state: State<'_, MlState>,
    model_id: String,
    input_data: Vec<f32>,
    input_shape: Vec<usize>,
) -> CommandResult<InferenceResultPayload> {
    let uuid = uuid::Uuid::parse_str(&model_id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let input = ml_runtime::create_tensor_input(input_data, input_shape)?;

    let engine = state.engine.read().await;
    let result = engine.run_inference(uuid, input).await?;

    Ok(InferenceResultPayload {
        output: result.output,
        shape: result.shape,
        duration_ms: result.duration_ms,
    })
}

/// Run inference on image data
#[tauri::command]
pub async fn run_image_inference(
    state: State<'_, MlState>,
    model_id: String,
    image_data: Vec<u8>,
    target_width: u32,
    target_height: u32,
) -> CommandResult<InferenceResultPayload> {
    let uuid = uuid::Uuid::parse_str(&model_id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let input = ml_runtime::create_image_input(
        &image_data,
        (target_width, target_height),
        true,
    )?;

    let engine = state.engine.read().await;
    let result = engine.run_inference(uuid, input).await?;

    Ok(InferenceResultPayload {
        output: result.output,
        shape: result.shape,
        duration_ms: result.duration_ms,
    })
}

/// Scan for available models in models directory
#[tauri::command]
pub async fn scan_available_models(
    state: State<'_, MlState>,
) -> CommandResult<Vec<String>> {
    let engine = state.engine.read().await;
    let paths = engine.registry().scan_available_models()?;

    Ok(paths.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

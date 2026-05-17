//! AI model runtime abstraction
//!
//! Provides a unified interface for running inference models.
//! In production, this wraps ONNX Runtime. For now, uses heuristic fallbacks.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Invalid input shape: expected {expected:?}, got {got:?}")]
    ShapeMismatch { expected: Vec<usize>, got: Vec<usize> },
}

/// Supported model formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    /// ONNX model file
    Onnx,
    /// TensorFlow Lite
    TfLite,
    /// Custom heuristic (no model file needed)
    Heuristic,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub format: ModelFormat,
    pub version: String,
    pub input_shapes: Vec<Vec<usize>>,
    pub output_shapes: Vec<Vec<usize>>,
    pub path: Option<PathBuf>,
}

/// Inference input tensor (flattened f32 data)
#[derive(Debug, Clone)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

impl Tensor {
    pub fn new(shape: Vec<usize>, data: Vec<f32>) -> Self {
        Self { shape, data }
    }

    pub fn zeros(shape: Vec<usize>) -> Self {
        let len: usize = shape.iter().product();
        Self { shape, data: vec![0.0; len] }
    }

    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }
}

/// AI model runtime
pub struct ModelRuntime {
    models: std::collections::HashMap<String, ModelInfo>,
}

impl ModelRuntime {
    pub fn new() -> Self {
        Self {
            models: std::collections::HashMap::new(),
        }
    }

    /// Register a model for inference
    pub fn register_model(&mut self, info: ModelInfo) {
        self.models.insert(info.name.clone(), info);
    }

    /// List registered models
    pub fn list_models(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// Run inference on a model
    pub fn infer(&self, model_name: &str, _inputs: &[Tensor]) -> Result<Vec<Tensor>, RuntimeError> {
        let model = self.models.get(model_name)
            .ok_or_else(|| RuntimeError::ModelNotFound(model_name.into()))?;

        match model.format {
            ModelFormat::Onnx => {
                // TODO: integrate ort (ONNX Runtime) crate when ready
                Err(RuntimeError::InferenceFailed("ONNX runtime not yet linked".into()))
            }
            ModelFormat::TfLite => {
                Err(RuntimeError::InferenceFailed("TFLite not yet linked".into()))
            }
            ModelFormat::Heuristic => {
                // Produce output tensors with same shapes as declared
                let outputs: Vec<Tensor> = model.output_shapes.iter()
                    .map(|shape| Tensor::zeros(shape.clone()))
                    .collect();
                Ok(outputs)
            }
        }
    }

    /// Get built-in dental models
    pub fn register_dental_models(&mut self) {
        // Margin detection model
        self.register_model(ModelInfo {
            name: "margin_detection".into(),
            format: ModelFormat::Heuristic,
            version: "0.1.0".into(),
            input_shapes: vec![vec![1, 3, 256, 256]], // point cloud batch
            output_shapes: vec![vec![1, 1, 256]],     // per-point probability
            path: None,
        });

        // Tooth segmentation model
        self.register_model(ModelInfo {
            name: "tooth_segmentation".into(),
            format: ModelFormat::Heuristic,
            version: "0.1.0".into(),
            input_shapes: vec![vec![1, 6, 4096]], // xyz + normals
            output_shapes: vec![vec![1, 33, 4096]], // 33 classes per point
            path: None,
        });

        // Quality assessment model
        self.register_model(ModelInfo {
            name: "quality_assessment".into(),
            format: ModelFormat::Heuristic,
            version: "0.1.0".into(),
            input_shapes: vec![vec![1, 12]], // feature vector
            output_shapes: vec![vec![1, 1]], // quality score
            path: None,
        });
    }
}

impl Default for ModelRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_zeros() {
        let t = Tensor::zeros(vec![2, 3]);
        assert_eq!(t.numel(), 6);
        assert!(t.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_tensor_new() {
        let t = Tensor::new(vec![1, 3], vec![1.0, 2.0, 3.0]);
        assert_eq!(t.numel(), 3);
        assert_eq!(t.data.len(), 3);
    }

    #[test]
    fn test_register_model() {
        let mut rt = ModelRuntime::new();
        rt.register_model(ModelInfo {
            name: "test".into(),
            format: ModelFormat::Heuristic,
            version: "1.0".into(),
            input_shapes: vec![vec![1, 3]],
            output_shapes: vec![vec![1, 1]],
            path: None,
        });
        assert_eq!(rt.list_models().len(), 1);
    }

    #[test]
    fn test_infer_heuristic() {
        let mut rt = ModelRuntime::new();
        rt.register_model(ModelInfo {
            name: "test".into(),
            format: ModelFormat::Heuristic,
            version: "1.0".into(),
            input_shapes: vec![vec![1, 3]],
            output_shapes: vec![vec![1, 2]],
            path: None,
        });
        let input = Tensor::new(vec![1, 3], vec![1.0, 2.0, 3.0]);
        let outputs = rt.infer("test", &[input]).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].numel(), 2);
    }

    #[test]
    fn test_infer_model_not_found() {
        let rt = ModelRuntime::new();
        let input = Tensor::new(vec![1], vec![1.0]);
        let result = rt.infer("nonexistent", &[input]);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_dental_models() {
        let mut rt = ModelRuntime::new();
        rt.register_dental_models();
        assert_eq!(rt.list_models().len(), 3);
    }

    #[test]
    fn test_default() {
        let rt = ModelRuntime::default();
        assert_eq!(rt.list_models().len(), 0);
    }
}

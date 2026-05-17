//! Inference engine

use app_core::types::EntityId;
use ndarray::{Array, ArrayD, IxDyn};
use ort::value::Tensor;
use std::time::Instant;
use tracing::info;

use crate::{MlError, ModelRegistry, Result};

/// Inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub output: Vec<f32>,
    pub shape: Vec<usize>,
    pub duration_ms: u64,
}

/// Inference engine
pub struct InferenceEngine {
    registry: ModelRegistry,
}

impl InferenceEngine {
    pub fn new(registry: ModelRegistry) -> Self {
        Self { registry }
    }

    /// Run inference on a model
    pub async fn run_inference(
        &self,
        model_id: EntityId,
        input: ArrayD<f32>,
    ) -> Result<InferenceResult> {
        let start = Instant::now();

        let models = self
            .registry
            .get_model(model_id)
            .await
            .ok_or_else(|| MlError::ModelNotFound(model_id.to_string()))?;

        let mut models_write = models.write().await;
        let loaded = models_write
            .get_mut(&model_id)
            .ok_or_else(|| MlError::ModelNotFound(model_id.to_string()))?;

        // Get input shape for tensor creation
        let input_shape: Vec<i64> = input.shape().iter().map(|&s| s as i64).collect();
        let input_data: Vec<f32> = input.iter().copied().collect();

        // Create tensor from raw data using ort 2.0 API - needs (shape, owned_data)
        let tensor = Tensor::<f32>::from_array((input_shape, input_data))
            .map_err(|e: ort::Error| MlError::InferenceError(e.to_string()))?;

        // Run inference
        let outputs = loaded
            .session
            .run(ort::inputs!["input" => tensor])
            .map_err(|e: ort::Error| MlError::InferenceError(e.to_string()))?;

        // Extract output
        let output_value = outputs
            .iter()
            .next()
            .map(|(_name, value)| value)
            .ok_or_else(|| MlError::InferenceError("No output tensor".to_string()))?;

        let (_shape_info, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e: ort::Error| MlError::InferenceError(e.to_string()))?;

        // Get output dimensions
        let shape: Vec<usize> = Vec::new();  // Shape info not directly accessible, can be improved later
        let output: Vec<f32> = data.to_vec();

        let duration = start.elapsed();
        info!(
            "Inference completed in {:?} (model: {})",
            duration, model_id
        );

        Ok(InferenceResult {
            output,
            shape,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Run batch inference
    pub async fn run_batch_inference(
        &self,
        model_id: EntityId,
        inputs: Vec<ArrayD<f32>>,
    ) -> Result<Vec<InferenceResult>> {
        let mut results = Vec::with_capacity(inputs.len());

        for input in inputs {
            results.push(self.run_inference(model_id, input).await?);
        }

        Ok(results)
    }

    /// Get the model registry
    pub fn registry(&self) -> &ModelRegistry {
        &self.registry
    }
}

/// Create input array from image bytes
pub fn create_image_input(
    image_bytes: &[u8],
    target_size: (u32, u32),
    normalize: bool,
) -> Result<ArrayD<f32>> {
    use image::GenericImageView;

    let img = image::load_from_memory(image_bytes)
        .map_err(|e| MlError::PreprocessingError(e.to_string()))?;

    let resized = img.resize_exact(
        target_size.0,
        target_size.1,
        image::imageops::FilterType::Lanczos3,
    );

    let (width, height) = resized.dimensions();
    let rgb = resized.to_rgb8();

    // Create array in NCHW format (batch, channels, height, width)
    let mut array = Array::zeros(IxDyn(&[1, 3, height as usize, width as usize]));

    for y in 0..height as usize {
        for x in 0..width as usize {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            if normalize {
                // Normalize to [0, 1]
                array[[0, 0, y, x]] = r / 255.0;
                array[[0, 1, y, x]] = g / 255.0;
                array[[0, 2, y, x]] = b / 255.0;
            } else {
                array[[0, 0, y, x]] = r;
                array[[0, 1, y, x]] = g;
                array[[0, 2, y, x]] = b;
            }
        }
    }

    Ok(array)
}

/// Create input array from raw float data
pub fn create_tensor_input(data: Vec<f32>, shape: Vec<usize>) -> Result<ArrayD<f32>> {
    let expected_size: usize = shape.iter().product();
    if data.len() != expected_size {
        return Err(MlError::PreprocessingError(format!(
            "Data length {} doesn't match shape {:?} (expected {})",
            data.len(),
            shape,
            expected_size
        )));
    }

    Array::from_shape_vec(IxDyn(&shape), data)
        .map_err(|e| MlError::PreprocessingError(e.to_string()))
}

//! AI inference pipeline: chains preprocessing → inference → postprocessing

use crate::runtime::{ModelRuntime, Tensor, RuntimeError};
use serde::{Deserialize, Serialize};

/// A processing step in the pipeline
pub trait PipelineStep: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, input: PipelineData) -> Result<PipelineData, PipelineError>;
}

/// Data flowing through the pipeline
#[derive(Debug, Clone)]
pub enum PipelineData {
    /// Raw point cloud (Nx3 f32)
    PointCloud(Vec<[f32; 3]>),
    /// Tensor data for model input/output
    Tensors(Vec<Tensor>),
    /// Per-vertex labels (segmentation results)
    Labels(Vec<u32>),
    /// Per-vertex scores (confidence/probability)
    Scores(Vec<f32>),
    /// 3D points result
    Points(Vec<[f64; 3]>),
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Step failed: {step} - {message}")]
    StepFailed { step: String, message: String },
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,
    pub model_name: String,
    pub batch_size: usize,
    pub confidence_threshold: f32,
}

/// An inference pipeline that chains steps
pub struct InferencePipeline {
    config: PipelineConfig,
    pre_steps: Vec<Box<dyn PipelineStep>>,
    post_steps: Vec<Box<dyn PipelineStep>>,
}

impl InferencePipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            pre_steps: Vec::new(),
            post_steps: Vec::new(),
        }
    }

    pub fn add_pre_step(&mut self, step: Box<dyn PipelineStep>) {
        self.pre_steps.push(step);
    }

    pub fn add_post_step(&mut self, step: Box<dyn PipelineStep>) {
        self.post_steps.push(step);
    }

    /// Run the full pipeline
    pub fn run(&self, runtime: &ModelRuntime, input: PipelineData) -> Result<PipelineData, PipelineError> {
        // 1. Preprocessing
        let mut data = input;
        for step in &self.pre_steps {
            data = step.process(data)?;
        }

        // 2. Inference
        let tensors = match data {
            PipelineData::Tensors(t) => t,
            PipelineData::PointCloud(pts) => {
                // Convert point cloud to tensor
                let flat: Vec<f32> = pts.iter().flat_map(|p| p.iter().copied()).collect();
                vec![Tensor::new(vec![1, 3, pts.len()], flat)]
            }
            _ => return Err(PipelineError::StepFailed {
                step: "inference".into(),
                message: "Expected Tensors or PointCloud input".into(),
            }),
        };

        let output_tensors = runtime.infer(&self.config.model_name, &tensors)?;
        let mut data = PipelineData::Tensors(output_tensors);

        // 3. Postprocessing
        for step in &self.post_steps {
            data = step.process(data)?;
        }

        Ok(data)
    }

    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }
}

// --- Built-in pipeline steps ---

/// Normalize point cloud to unit sphere
pub struct NormalizePointCloud;

impl PipelineStep for NormalizePointCloud {
    fn name(&self) -> &str { "normalize_points" }

    fn process(&self, input: PipelineData) -> Result<PipelineData, PipelineError> {
        match input {
            PipelineData::PointCloud(pts) => {
                if pts.is_empty() {
                    return Ok(PipelineData::PointCloud(pts));
                }
                // Find centroid
                let n = pts.len() as f32;
                let cx = pts.iter().map(|p| p[0]).sum::<f32>() / n;
                let cy = pts.iter().map(|p| p[1]).sum::<f32>() / n;
                let cz = pts.iter().map(|p| p[2]).sum::<f32>() / n;

                // Center and find max distance
                let centered: Vec<[f32; 3]> = pts.iter()
                    .map(|p| [p[0] - cx, p[1] - cy, p[2] - cz])
                    .collect();
                let max_dist = centered.iter()
                    .map(|p| (p[0]*p[0] + p[1]*p[1] + p[2]*p[2]).sqrt())
                    .fold(0.0f32, f32::max);

                let scale = if max_dist > 0.0 { 1.0 / max_dist } else { 1.0 };
                let normalized: Vec<[f32; 3]> = centered.iter()
                    .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
                    .collect();

                Ok(PipelineData::PointCloud(normalized))
            }
            other => Ok(other),
        }
    }
}

/// Apply confidence threshold to convert scores into labels
pub struct ThresholdStep {
    pub threshold: f32,
}

impl PipelineStep for ThresholdStep {
    fn name(&self) -> &str { "threshold" }

    fn process(&self, input: PipelineData) -> Result<PipelineData, PipelineError> {
        match input {
            PipelineData::Scores(scores) => {
                let labels: Vec<u32> = scores.iter()
                    .map(|&s| if s >= self.threshold { 1 } else { 0 })
                    .collect();
                Ok(PipelineData::Labels(labels))
            }
            PipelineData::Tensors(tensors) => {
                if let Some(t) = tensors.first() {
                    let labels: Vec<u32> = t.data.iter()
                        .map(|&s| if s >= self.threshold { 1 } else { 0 })
                        .collect();
                    Ok(PipelineData::Labels(labels))
                } else {
                    Ok(PipelineData::Labels(Vec::new()))
                }
            }
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_point_cloud() {
        let pts = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let step = NormalizePointCloud;
        let result = step.process(PipelineData::PointCloud(pts)).unwrap();
        if let PipelineData::PointCloud(normalized) = result {
            // Check centered: mean should be ~0
            let n = normalized.len() as f32;
            let cx: f32 = normalized.iter().map(|p| p[0]).sum::<f32>() / n;
            let cy: f32 = normalized.iter().map(|p| p[1]).sum::<f32>() / n;
            assert!(cx.abs() < 1e-5);
            assert!(cy.abs() < 1e-5);
        } else {
            panic!("Expected PointCloud");
        }
    }

    #[test]
    fn test_normalize_empty() {
        let step = NormalizePointCloud;
        let result = step.process(PipelineData::PointCloud(vec![])).unwrap();
        if let PipelineData::PointCloud(pts) = result {
            assert!(pts.is_empty());
        } else {
            panic!("Expected empty PointCloud");
        }
    }

    #[test]
    fn test_normalize_passthrough_labels() {
        let step = NormalizePointCloud;
        let result = step.process(PipelineData::Labels(vec![1, 2, 3])).unwrap();
        if let PipelineData::Labels(labels) = result {
            assert_eq!(labels, vec![1, 2, 3]);
        } else {
            panic!("Expected Labels passthrough");
        }
    }

    #[test]
    fn test_threshold_scores() {
        let step = ThresholdStep { threshold: 0.5 };
        let scores = vec![0.2, 0.6, 0.8, 0.1, 0.5];
        let result = step.process(PipelineData::Scores(scores)).unwrap();
        if let PipelineData::Labels(labels) = result {
            assert_eq!(labels, vec![0, 1, 1, 0, 1]);
        } else {
            panic!("Expected Labels");
        }
    }

    #[test]
    fn test_threshold_tensors() {
        let step = ThresholdStep { threshold: 0.5 };
        let tensor = crate::runtime::Tensor::new(vec![1, 3], vec![0.1, 0.9, 0.5]);
        let result = step.process(PipelineData::Tensors(vec![tensor])).unwrap();
        if let PipelineData::Labels(labels) = result {
            assert_eq!(labels, vec![0, 1, 1]);
        } else {
            panic!("Expected Labels");
        }
    }

    #[test]
    fn test_pipeline_config() {
        let config = PipelineConfig {
            name: "test".into(),
            model_name: "model".into(),
            batch_size: 32,
            confidence_threshold: 0.5,
        };
        let pipeline = InferencePipeline::new(config);
        assert_eq!(pipeline.config().name, "test");
        assert_eq!(pipeline.config().batch_size, 32);
    }

    #[test]
    fn test_step_name() {
        assert_eq!(NormalizePointCloud.name(), "normalize_points");
        assert_eq!((ThresholdStep { threshold: 0.5 }).name(), "threshold");
    }
}

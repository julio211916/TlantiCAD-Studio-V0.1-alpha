//! ML inference engine for dental analysis

use serde::{Deserialize, Serialize};
use crate::tensor::DentalTensor;

/// Type of dental ML inference task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceTask {
    /// Segment teeth from CBCT volume
    ToothSegmentation,
    /// Detect implant position from scan
    ImplantDetection,
    /// Classify caries from image tensor
    CariesDetection,
    /// Estimate bone density from HU values
    BoneDensityEstimation,
    /// Predict crown morphology
    CrownMorphologyPrediction,
    /// Nerve canal tracking in CBCT
    NerveCanalDetection,
}

/// ONNX model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub task: InferenceTask,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

/// Inference result for tooth segmentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationResult {
    pub mask: DentalTensor,
    pub tooth_ids: Vec<u8>,
    pub confidence: Vec<f32>,
}

/// Bone density estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneDensityResult {
    pub hounsfield_mean: f32,
    pub hounsfield_std: f32,
    pub density_class: BoneDensityClass,
    pub confidence: f32,
}

/// Misch bone density classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoneDensityClass {
    /// D1: Dense cortical bone (>1250 HU)
    D1,
    /// D2: Thick cortical with coarse trabecular (850-1250 HU)
    D2,
    /// D3: Thin cortical with fine trabecular (350-850 HU)
    D3,
    /// D4: Fine trabecular (<350 HU)
    D4,
}

impl BoneDensityClass {
    pub fn from_hounsfield(hu: f32) -> Self {
        match hu as i32 {
            h if h > 1250 => BoneDensityClass::D1,
            h if h > 850  => BoneDensityClass::D2,
            h if h > 350  => BoneDensityClass::D3,
            _              => BoneDensityClass::D4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BoneDensityClass::D1 => "Dense cortical bone — excellent implant stability",
            BoneDensityClass::D2 => "Cortical + coarse trabecular — good prognosis",
            BoneDensityClass::D3 => "Thin cortical + fine trabecular — requires care",
            BoneDensityClass::D4 => "Fine trabecular — poor density, consider grafting",
        }
    }
}

/// Software inference engine (CPU-only, no ONNX Runtime required)
/// For production use, swap internals with ort crate
pub struct InferenceEngine {
    pub metadata: ModelMetadata,
}

impl InferenceEngine {
    pub fn new(metadata: ModelMetadata) -> Self {
        Self { metadata }
    }

    /// Estimate bone density from HU value slice
    pub fn estimate_bone_density(&self, hu_values: &[f32]) -> BoneDensityResult {
        let mean = hu_values.iter().sum::<f32>() / hu_values.len() as f32;
        let variance = hu_values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / hu_values.len() as f32;
        let std = variance.sqrt();
        BoneDensityResult {
            hounsfield_mean: mean,
            hounsfield_std: std,
            density_class: BoneDensityClass::from_hounsfield(mean),
            confidence: 0.85,
        }
    }

    /// Mock tooth segmentation — returns labeled regions
    pub fn segment_cbct(&self, _volume: &DentalTensor) -> SegmentationResult {
        // In production: run ONNX model; here we return empty result
        SegmentationResult {
            mask: DentalTensor::zeros(crate::tensor::TensorShape::Vector(1)),
            tooth_ids: (11u8..=48).filter(|&t| (t % 10) >= 1 && (t % 10) <= 8).collect(),
            confidence: vec![0.9; 32],
        }
    }
}

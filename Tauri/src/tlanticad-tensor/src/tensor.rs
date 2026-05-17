//! Core tensor types for dental ML operations

use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Dental-domain tensor shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorShape {
    /// 1D feature vector [features]
    Vector(usize),
    /// 2D matrix [rows, cols]
    Matrix(usize, usize),
    /// 3D volumetric data [depth, height, width] — for CBCT volumes
    Volume(usize, usize, usize),
    /// 4D batched volume [batch, depth, height, width]
    BatchedVolume(usize, usize, usize, usize),
}

/// Generic dental tensor holding f32 data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DentalTensor {
    pub shape: TensorShape,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorDtype {
    F32,
    F64,
    I32,
    U8,
}

impl DentalTensor {
    /// Create a new zeroed tensor with given shape
    pub fn zeros(shape: TensorShape) -> Self {
        let size = match shape {
            TensorShape::Vector(n) => n,
            TensorShape::Matrix(r, c) => r * c,
            TensorShape::Volume(d, h, w) => d * h * w,
            TensorShape::BatchedVolume(b, d, h, w) => b * d * h * w,
        };
        Self { shape, data: vec![0.0_f32; size], dtype: TensorDtype::F32 }
    }

    /// Create tensor from flat data
    pub fn from_data(shape: TensorShape, data: Vec<f32>) -> Result<Self, String> {
        let expected = match shape {
            TensorShape::Vector(n) => n,
            TensorShape::Matrix(r, c) => r * c,
            TensorShape::Volume(d, h, w) => d * h * w,
            TensorShape::BatchedVolume(b, d, h, w) => b * d * h * w,
        };
        if data.len() != expected {
            return Err(format!("Expected {} elements, got {}", expected, data.len()));
        }
        Ok(Self { shape, data, dtype: TensorDtype::F32 })
    }

    /// Get total number of elements
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Normalize tensor values to [0, 1] range
    pub fn normalize(&mut self) {
        let min = self.data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = max - min;
        if range > f32::EPSILON {
            for v in &mut self.data {
                *v = (*v - min) / range;
            }
        }
    }

    /// Convert a CBCT slice (2D) to tensor
    pub fn from_cbct_slice(pixels: &[u16], width: usize, height: usize) -> Self {
        let data: Vec<f32> = pixels.iter().map(|&p| p as f32 / 65535.0_f32).collect();
        Self {
            shape: TensorShape::Matrix(height, width),
            data,
            dtype: TensorDtype::F32,
        }
    }

    /// Stack 2D slices into a 3D volume tensor
    pub fn stack_slices(slices: &[DentalTensor]) -> Result<Self, String> {
        if slices.is_empty() {
            return Err("No slices to stack".to_string());
        }
        let (height, width) = match slices[0].shape {
            TensorShape::Matrix(h, w) => (h, w),
            _ => return Err("Slices must be 2D matrices".to_string()),
        };
        let depth = slices.len();
        let mut data = Vec::with_capacity(depth * height * width);
        for slice in slices {
            data.extend_from_slice(&slice.data);
        }
        Self::from_data(TensorShape::Volume(depth, height, width), data)
    }
}

/// Dental-specific feature vector for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothFeatureVector {
    pub fdi_number: u8,
    pub features: Array1<f32>,
    pub feature_names: Vec<String>,
}

impl ToothFeatureVector {
    /// Create standard 64-feature vector for a tooth
    pub fn new_standard(fdi_number: u8) -> Self {
        Self {
            fdi_number,
            features: Array1::zeros(64),
            feature_names: standard_feature_names(),
        }
    }
}

fn standard_feature_names() -> Vec<String> {
    vec![
        "crown_height", "crown_width_md", "crown_width_bl",
        "root_length", "root_curvature", "bone_level",
        "probe_depth_m", "probe_depth_d", "probe_depth_b", "probe_depth_l",
        "bleeding_m", "bleeding_d", "bleeding_b", "bleeding_l",
        "furcation_class", "mobility_grade", "calculus_score",
        "restoration_present", "restoration_material",
        "caries_risk_score",
    ].iter().map(|s| s.to_string()).collect()
}

//! Preprocessing utilities for ML inputs

use ndarray::{Array, ArrayD, IxDyn};

use crate::{MlError, Result};

/// Image preprocessing options
#[derive(Debug, Clone)]
pub struct ImagePreprocessOptions {
    /// Target size (width, height)
    pub target_size: (u32, u32),
    /// Whether to normalize to [0, 1]
    pub normalize: bool,
    /// Mean values for normalization (per channel)
    pub mean: Option<[f32; 3]>,
    /// Std values for normalization (per channel)
    pub std: Option<[f32; 3]>,
    /// Whether to convert BGR to RGB
    pub bgr_to_rgb: bool,
}

impl Default for ImagePreprocessOptions {
    fn default() -> Self {
        Self {
            target_size: (224, 224),
            normalize: true,
            mean: Some([0.485, 0.456, 0.406]), // ImageNet mean
            std: Some([0.229, 0.224, 0.225]),  // ImageNet std
            bgr_to_rgb: false,
        }
    }
}

/// Preprocess image with ImageNet normalization
pub fn preprocess_imagenet(image_bytes: &[u8]) -> Result<ArrayD<f32>> {
    preprocess_image(image_bytes, &ImagePreprocessOptions::default())
}

/// Preprocess image with custom options
pub fn preprocess_image(image_bytes: &[u8], options: &ImagePreprocessOptions) -> Result<ArrayD<f32>> {
    use image::GenericImageView;

    let img = image::load_from_memory(image_bytes)
        .map_err(|e| MlError::PreprocessingError(e.to_string()))?;

    let resized = img.resize_exact(
        options.target_size.0,
        options.target_size.1,
        image::imageops::FilterType::Lanczos3,
    );

    let (width, height) = resized.dimensions();
    let rgb = resized.to_rgb8();

    // Create array in NCHW format
    let mut array = Array::zeros(IxDyn(&[1, 3, height as usize, width as usize]));

    for y in 0..height as usize {
        for x in 0..width as usize {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            
            let (r, g, b) = if options.bgr_to_rgb {
                (pixel[2] as f32, pixel[1] as f32, pixel[0] as f32)
            } else {
                (pixel[0] as f32, pixel[1] as f32, pixel[2] as f32)
            };

            let channels = if options.normalize {
                [r / 255.0, g / 255.0, b / 255.0]
            } else {
                [r, g, b]
            };

            // Apply mean/std normalization if provided
            let (mean, std) = (
                options.mean.unwrap_or([0.0, 0.0, 0.0]),
                options.std.unwrap_or([1.0, 1.0, 1.0]),
            );

            array[[0, 0, y, x]] = (channels[0] - mean[0]) / std[0];
            array[[0, 1, y, x]] = (channels[1] - mean[1]) / std[1];
            array[[0, 2, y, x]] = (channels[2] - mean[2]) / std[2];
        }
    }

    Ok(array)
}

/// Normalize point cloud data
pub fn preprocess_point_cloud(
    points: &[f32],
    center: bool,
    scale: bool,
) -> Result<ArrayD<f32>> {
    if points.len() % 3 != 0 {
        return Err(MlError::PreprocessingError(
            "Point cloud data must have length divisible by 3".to_string(),
        ));
    }

    let num_points = points.len() / 3;
    let mut processed = points.to_vec();

    if center {
        // Calculate centroid
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut cz = 0.0f32;

        for i in 0..num_points {
            cx += points[i * 3];
            cy += points[i * 3 + 1];
            cz += points[i * 3 + 2];
        }

        cx /= num_points as f32;
        cy /= num_points as f32;
        cz /= num_points as f32;

        // Center points
        for i in 0..num_points {
            processed[i * 3] -= cx;
            processed[i * 3 + 1] -= cy;
            processed[i * 3 + 2] -= cz;
        }
    }

    if scale {
        // Calculate max distance from origin
        let mut max_dist = 0.0f32;
        for i in 0..num_points {
            let x = processed[i * 3];
            let y = processed[i * 3 + 1];
            let z = processed[i * 3 + 2];
            let dist = (x * x + y * y + z * z).sqrt();
            max_dist = max_dist.max(dist);
        }

        // Scale to unit sphere
        if max_dist > 0.0 {
            for p in processed.iter_mut() {
                *p /= max_dist;
            }
        }
    }

    // Create array in shape [1, num_points, 3]
    Array::from_shape_vec(IxDyn(&[1, num_points, 3]), processed)
        .map_err(|e| MlError::PreprocessingError(e.to_string()))
}

/// Softmax activation
pub fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = logits.iter().map(|x| (x - max).exp()).sum();
    logits.iter().map(|x| (x - max).exp() / exp_sum).collect()
}

/// Argmax
pub fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

/// Top-k indices and values
pub fn top_k(values: &[f32], k: usize) -> Vec<(usize, f32)> {
    let mut indexed: Vec<_> = values.iter().cloned().enumerate().collect();
    indexed.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    indexed.into_iter().take(k).collect()
}

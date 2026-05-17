//! Common tensor operations for dental image processing

use crate::tensor::DentalTensor;

/// Apply 3×3 Gaussian blur to 2D tensor
pub fn gaussian_blur_2d(tensor: &DentalTensor, sigma: f32) -> Result<DentalTensor, String> {
    let (height, width) = match tensor.shape {
        crate::tensor::TensorShape::Matrix(h, w) => (h, w),
        _ => return Err("gaussian_blur_2d requires a 2D matrix tensor".to_string()),
    };
    let kernel = gaussian_kernel(sigma);
    let mut result = vec![0.0f32; tensor.data.len()];
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let mut sum = 0.0f32;
            for ky in 0..3usize {
                for kx in 0..3usize {
                    let py = y + ky - 1;
                    let px = x + kx - 1;
                    sum += tensor.data[py * width + px] * kernel[ky * 3 + kx];
                }
            }
            result[y * width + x] = sum;
        }
    }
    DentalTensor::from_data(tensor.shape, result)
}

fn gaussian_kernel(sigma: f32) -> [f32; 9] {
    let mut kernel = [0.0f32; 9];
    let mut sum = 0.0;
    for (i, k) in kernel.iter_mut().enumerate() {
        let x = (i % 3) as f32 - 1.0;
        let y = (i / 3) as f32 - 1.0;
        *k = (-(x * x + y * y) / (2.0 * sigma * sigma)).exp();
        sum += *k;
    }
    for k in &mut kernel {
        *k /= sum;
    }
    kernel
}

/// Threshold a tensor to binary mask
pub fn threshold(tensor: &DentalTensor, threshold_val: f32) -> DentalTensor {
    let data: Vec<f32> = tensor.data.iter().map(|&v| if v >= threshold_val { 1.0 } else { 0.0 }).collect();
    DentalTensor { shape: tensor.shape, data, dtype: tensor.dtype }
}

/// Compute histogram of tensor values
pub fn histogram(tensor: &DentalTensor, bins: usize) -> Vec<u32> {
    let min = tensor.data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = tensor.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = max - min;
    let mut hist = vec![0u32; bins];
    if range > f32::EPSILON {
        for &v in &tensor.data {
            let bin = ((v - min) / range * (bins - 1) as f32) as usize;
            hist[bin.min(bins - 1)] += 1;
        }
    }
    hist
}

/// Compute mean Hounsfield Unit value from 16-bit pixel data
pub fn mean_hu(pixels: &[i16]) -> f32 {
    if pixels.is_empty() { return 0.0; }
    pixels.iter().map(|&p| p as f32).sum::<f32>() / pixels.len() as f32
}

/// Window/Level adjustment for CBCT display (dental: C=1000, W=3000)
pub fn window_level(tensor: &DentalTensor, center: f32, width: f32) -> DentalTensor {
    let min = center - width / 2.0;
    let max = center + width / 2.0;
    let data: Vec<f32> = tensor.data.iter().map(|&v| {
        ((v - min) / (max - min)).clamp(0.0, 1.0)
    }).collect();
    DentalTensor { shape: tensor.shape, data, dtype: tensor.dtype }
}

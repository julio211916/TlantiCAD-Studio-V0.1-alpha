//! CBCT Volume reconstruction and processing

use serde::{Deserialize, Serialize};
use crate::parser::DicomMetadata;

/// A single CBCT/CT slice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomSlice {
    pub metadata: DicomMetadata,
    pub pixels: Vec<i16>,
}

impl DicomSlice {
    pub fn new(metadata: DicomMetadata, pixels: Vec<i16>) -> Self {
        Self { metadata, pixels }
    }

    /// Get Hounsfield Unit at (row, col)
    pub fn hu_at(&self, row: u32, col: u32) -> Option<f64> {
        let idx = (row * self.metadata.columns + col) as usize;
        self.pixels.get(idx).map(|&v| {
            crate::parser::to_hounsfield(v, self.metadata.rescale_slope, self.metadata.rescale_intercept)
        })
    }

    /// Compute mean HU of region of interest
    pub fn mean_hu_roi(&self, x: u32, y: u32, width: u32, height: u32) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;
        for row in y..(y + height).min(self.metadata.rows) {
            for col in x..(x + width).min(self.metadata.columns) {
                if let Some(hu) = self.hu_at(row, col) {
                    sum += hu;
                    count += 1;
                }
            }
        }
        if count > 0 { sum / count as f64 } else { 0.0 }
    }
}

/// Full CBCT volume (ordered axial slices)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbctVolume {
    pub slices: Vec<DicomSlice>,
    pub voxel_size: [f64; 3],
    pub origin: [f64; 3],
    pub dimensions: [u32; 3],
}

impl CbctVolume {
    /// Build volume from sorted slices
    pub fn from_slices(mut slices: Vec<DicomSlice>) -> Result<Self, String> {
        if slices.is_empty() {
            return Err("No slices provided".to_string());
        }
        slices.sort_by(|a, b| a.metadata.instance_number.cmp(&b.metadata.instance_number));
        let meta = &slices[0].metadata;
        let voxel_size = [
            meta.pixel_spacing[0],
            meta.pixel_spacing[1],
            meta.slice_thickness,
        ];
        let origin = [0.0, 0.0, slices[0].metadata.slice_position];
        let dimensions = [meta.columns, meta.rows, slices.len() as u32];
        Ok(Self { slices, voxel_size, origin, dimensions })
    }

    /// Get HU value at voxel (x, y, z)
    pub fn hu_at_voxel(&self, x: u32, y: u32, z: u32) -> Option<f64> {
        self.slices.get(z as usize)?.hu_at(y, x)
    }

    /// Extract axial slice as pixel buffer
    pub fn axial_slice_hu(&self, z: u32) -> Option<Vec<f64>> {
        let slice = self.slices.get(z as usize)?;
        Some(slice.pixels.iter().map(|&v| {
            crate::parser::to_hounsfield(v, slice.metadata.rescale_slope, slice.metadata.rescale_intercept)
        }).collect())
    }

    /// Estimate bone volume above HU threshold (in mm³)
    pub fn bone_volume_mm3(&self, hu_threshold: f64) -> f64 {
        let voxel_vol = self.voxel_size[0] * self.voxel_size[1] * self.voxel_size[2];
        let mut count = 0u64;
        for slice in &self.slices {
            for &pix in &slice.pixels {
                let hu = crate::parser::to_hounsfield(pix, slice.metadata.rescale_slope, slice.metadata.rescale_intercept);
                if hu >= hu_threshold { count += 1; }
            }
        }
        count as f64 * voxel_vol
    }

    /// Total physical size in mm
    pub fn physical_size_mm(&self) -> [f64; 3] {
        [
            self.dimensions[0] as f64 * self.voxel_size[0],
            self.dimensions[1] as f64 * self.voxel_size[1],
            self.dimensions[2] as f64 * self.voxel_size[2],
        ]
    }
}

/// Multiplanar reconstruction plane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MprPlane {
    Axial,
    Coronal,
    Sagittal,
}

/// Extract an MPR slice from a CBCT volume
pub fn extract_mpr(volume: &CbctVolume, plane: MprPlane, index: u32) -> Option<Vec<f64>> {
    let [cols, rows, depth] = volume.dimensions;
    match plane {
        MprPlane::Axial => volume.axial_slice_hu(index),
        MprPlane::Coronal => {
            if index >= rows { return None; }
            let mut pixels = Vec::with_capacity((cols * depth) as usize);
            for z in 0..depth {
                for x in 0..cols {
                    pixels.push(volume.hu_at_voxel(x, index, z).unwrap_or(-1000.0));
                }
            }
            Some(pixels)
        }
        MprPlane::Sagittal => {
            if index >= cols { return None; }
            let mut pixels = Vec::with_capacity((rows * depth) as usize);
            for z in 0..depth {
                for y in 0..rows {
                    pixels.push(volume.hu_at_voxel(index, y, z).unwrap_or(-1000.0));
                }
            }
            Some(pixels)
        }
    }
}

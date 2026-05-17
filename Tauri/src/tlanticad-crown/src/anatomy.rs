//! Anatomic crown: morph library tooth to fit preparation

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Parameters for morphing a library tooth onto a preparation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphParams {
    /// Influence of library shape vs preparation (0..1)
    pub anatomy_strength: f64,
    /// Cusp height scaling
    pub cusp_height_scale: f64,
    /// Fissure depth scaling
    pub fissure_depth_scale: f64,
    /// Overall height offset (mm)
    pub height_offset: f64,
    /// Bucco-lingual width scale
    pub bl_scale: f64,
    /// Mesio-distal width scale
    pub md_scale: f64,
}

impl Default for MorphParams {
    fn default() -> Self {
        Self {
            anatomy_strength: 0.7,
            cusp_height_scale: 1.0,
            fissure_depth_scale: 1.0,
            height_offset: 0.0,
            bl_scale: 1.0,
            md_scale: 1.0,
        }
    }
}

/// Morph a library tooth onto a preparation by matching boundaries
///
/// 1. Align library crown center to preparation center
/// 2. Scale to match preparation dimensions
/// 3. Blend the lower boundary with the margin line
/// 4. Apply anatomy strength to modulate detail
pub fn morph_library_to_preparation(
    library_verts: &[Point3<f64>],
    preparation_bounds: &([f64; 3], [f64; 3]), // (min, max)
    margin_points: &[Point3<f64>],
    params: &MorphParams,
) -> Vec<Point3<f64>> {
    if library_verts.is_empty() {
        return Vec::new();
    }

    // Compute library bounds
    let (lib_min, lib_max) = compute_bounds(library_verts);
    let lib_center = [
        (lib_min[0] + lib_max[0]) / 2.0,
        (lib_min[1] + lib_max[1]) / 2.0,
        lib_min[2], // bottom of library tooth
    ];
    let lib_size = [
        lib_max[0] - lib_min[0],
        lib_max[1] - lib_min[1],
        lib_max[2] - lib_min[2],
    ];

    let prep_center = [
        (preparation_bounds.0[0] + preparation_bounds.1[0]) / 2.0,
        (preparation_bounds.0[1] + preparation_bounds.1[1]) / 2.0,
        preparation_bounds.0[2],
    ];
    let prep_size = [
        preparation_bounds.1[0] - preparation_bounds.0[0],
        preparation_bounds.1[1] - preparation_bounds.0[1],
        preparation_bounds.1[2] - preparation_bounds.0[2],
    ];

    // Scale factors
    let sx = if lib_size[0] > 0.0 { (prep_size[0] / lib_size[0]) * params.md_scale } else { 1.0 };
    let sy = if lib_size[1] > 0.0 { (prep_size[1] / lib_size[1]) * params.bl_scale } else { 1.0 };
    let sz = if lib_size[2] > 0.0 { prep_size[2] / lib_size[2] } else { 1.0 };

    let mut result: Vec<Point3<f64>> = library_verts.iter().map(|v| {
        // Center, scale, reposition
        let x = (v.x - lib_center[0]) * sx + prep_center[0];
        let y = (v.y - lib_center[1]) * sy + prep_center[1];
        let z = (v.z - lib_center[2]) * sz + prep_center[2] + params.height_offset;
        Point3::new(x, y, z)
    }).collect();

    // Blend lower vertices with margin
    if !margin_points.is_empty() {
        let _margin_z = margin_points.iter().map(|m| m.z).sum::<f64>() / margin_points.len() as f64;
        let blend_range = prep_size[2] * 0.3; // Bottom 30%

        for v in &mut result {
            let height_from_bottom = v.z - prep_center[2];
            if height_from_bottom < blend_range {
                let t = (height_from_bottom / blend_range).clamp(0.0, 1.0);
                // Find closest margin point
                let closest = margin_points.iter()
                    .min_by(|a, b| {
                        let da = (a.x - v.x).powi(2) + (a.y - v.y).powi(2);
                        let db = (b.x - v.x).powi(2) + (b.y - v.y).powi(2);
                        da.partial_cmp(&db).unwrap()
                    });
                if let Some(mp) = closest {
                    let weight = (1.0 - t) * params.anatomy_strength;
                    v.x = v.x * (1.0 - weight) + mp.x * weight;
                    v.y = v.y * (1.0 - weight) + mp.y * weight;
                }
            }
        }
    }

    result
}

fn compute_bounds(verts: &[Point3<f64>]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for v in verts {
        min[0] = min[0].min(v.x);
        min[1] = min[1].min(v.y);
        min[2] = min[2].min(v.z);
        max[0] = max[0].max(v.x);
        max[1] = max[1].max(v.y);
        max[2] = max[2].max(v.z);
    }
    (min, max)
}

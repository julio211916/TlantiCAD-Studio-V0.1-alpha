//! Symmetry modes for sculpting strokes

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use crate::brush_engine::StrokePoint;

/// Symmetry mode for sculpt brushes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymmetryMode {
    None,
    MirrorX,
    MirrorY,
    MirrorZ,
    Radial(u32),
}

/// Mirror a stroke based on the active symmetry mode.
///
/// Returns the additional mirrored/rotated copies of the original stroke.
/// Does not include the original stroke points.
pub fn mirror_stroke(stroke: &[StrokePoint], mode: &SymmetryMode) -> Vec<StrokePoint> {
    match mode {
        SymmetryMode::None => Vec::new(),
        SymmetryMode::MirrorX => stroke
            .iter()
            .map(|sp| StrokePoint {
                position: Point3::new(-sp.position.x, sp.position.y, sp.position.z),
                normal: Vector3::new(-sp.normal.x, sp.normal.y, sp.normal.z),
                pressure: sp.pressure,
                timestamp: sp.timestamp,
            })
            .collect(),
        SymmetryMode::MirrorY => stroke
            .iter()
            .map(|sp| StrokePoint {
                position: Point3::new(sp.position.x, -sp.position.y, sp.position.z),
                normal: Vector3::new(sp.normal.x, -sp.normal.y, sp.normal.z),
                pressure: sp.pressure,
                timestamp: sp.timestamp,
            })
            .collect(),
        SymmetryMode::MirrorZ => stroke
            .iter()
            .map(|sp| StrokePoint {
                position: Point3::new(sp.position.x, sp.position.y, -sp.position.z),
                normal: Vector3::new(sp.normal.x, sp.normal.y, -sp.normal.z),
                pressure: sp.pressure,
                timestamp: sp.timestamp,
            })
            .collect(),
        SymmetryMode::Radial(count) => apply_radial_symmetry(stroke, *count),
    }
}

/// Apply radial symmetry to a stroke, returning `count - 1` rotated copies.
pub fn apply_radial_symmetry(stroke: &[StrokePoint], count: u32) -> Vec<StrokePoint> {
    if count < 2 {
        return Vec::new();
    }

    let mut result = Vec::new();
    for i in 1..count {
        let angle = (i as f64) * 2.0 * std::f64::consts::PI / count as f64;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for sp in stroke {
            let px = sp.position.x * cos_a - sp.position.z * sin_a;
            let pz = sp.position.x * sin_a + sp.position.z * cos_a;
            let nx = sp.normal.x * cos_a - sp.normal.z * sin_a;
            let nz = sp.normal.x * sin_a + sp.normal.z * cos_a;

            result.push(StrokePoint {
                position: Point3::new(px, sp.position.y, pz),
                normal: Vector3::new(nx, sp.normal.y, nz),
                pressure: sp.pressure,
                timestamp: sp.timestamp,
            });
        }
    }
    result
}

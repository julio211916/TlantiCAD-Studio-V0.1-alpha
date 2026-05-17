//! Stamp/alpha brushes for surface detailing

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Stamp shape / alpha type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StampShape {
    Circle,
    Diamond,
    Square,
    Oval,
    Custom(Vec<f32>),
}

impl StampShape {
    /// Sample the alpha value at normalised coordinates (u, v) in [-1, 1]
    fn sample(&self, u: f64, v: f64) -> f64 {
        let r2 = u * u + v * v;
        match self {
            StampShape::Circle => {
                if r2 <= 1.0 { 1.0 - r2.sqrt() } else { 0.0 }
            }
            StampShape::Oval => {
                let r2_oval = u * u * 0.5 + v * v * 1.5;
                if r2_oval <= 1.0 { 1.0 - r2_oval.sqrt() } else { 0.0 }
            }
            StampShape::Square => {
                if u.abs() <= 1.0 && v.abs() <= 1.0 { 1.0 - u.abs().max(v.abs()) } else { 0.0 }
            }
            StampShape::Diamond => {
                let d = u.abs() + v.abs();
                if d <= 1.0 { 1.0 - d } else { 0.0 }
            }
            StampShape::Custom(pixels) => {
                // 16×16 custom bitmap
                let size = 16usize;
                let ix = ((u * 0.5 + 0.5) * (size - 1) as f64).clamp(0.0, (size - 1) as f64) as usize;
                let iy = ((v * 0.5 + 0.5) * (size - 1) as f64).clamp(0.0, (size - 1) as f64) as usize;
                let idx = iy * size + ix;
                pixels.get(idx).copied().unwrap_or(0.0) as f64
            }
        }
    }
}

/// Apply a stamp at the given surface position and normal.
pub fn apply_stamp(
    mesh: &mut Mesh,
    position: &Point3<f64>,
    normal: &Vector3<f64>,
    stamp: &StampShape,
    depth: f64,
    size: f64,
) {
    if size <= 0.0 {
        return;
    }

    let n = normal.normalize();
    // Build a tangent frame for the stamp
    let tangent = if n.x.abs() < 0.9 {
        n.cross(&Vector3::x()).normalize()
    } else {
        n.cross(&Vector3::y()).normalize()
    };
    let bitangent = n.cross(&tangent).normalize();

    for v in &mut mesh.vertices {
        let delta = v.coords - position.coords;
        let u = delta.dot(&tangent) / size;
        let vv = delta.dot(&bitangent) / size;
        let along_normal = delta.dot(&n);
        if along_normal.abs() > size {
            continue;
        }
        let alpha = stamp.sample(u, vv);
        if alpha > 0.0 {
            v.coords += n * (alpha * depth);
        }
    }
    mesh.calculate_normals();
}

//! Full brush engine for freeform sculpting

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Sculpt brush types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushType {
    Clay,
    Snake,
    Inflate,
    Move,
    Smooth,
    Pinch,
    Flatten,
    TrimFlat,
}

/// A sculpt brush with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SculptBrush {
    pub radius: f64,
    pub strength: f64,
    pub falloff: f64,
    pub brush_type: BrushType,
}

impl Default for SculptBrush {
    fn default() -> Self {
        Self {
            radius: 2.0,
            strength: 0.5,
            falloff: 0.8,
            brush_type: BrushType::Clay,
        }
    }
}

/// The active brush engine state
#[derive(Debug, Clone)]
pub struct BrushEngine {
    pub active_brush: SculptBrush,
    pub symmetry: crate::symmetry::SymmetryMode,
    pub pressure_sensitivity: bool,
}

impl Default for BrushEngine {
    fn default() -> Self {
        Self {
            active_brush: SculptBrush::default(),
            symmetry: crate::symmetry::SymmetryMode::None,
            pressure_sensitivity: true,
        }
    }
}

/// A single point in a sculpt stroke
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub pressure: f32,
    pub timestamp: f64,
}

/// Gaussian brush falloff function (smooth, bell-shaped)
pub fn gaussian_falloff(distance: f64, radius: f64) -> f64 {
    if distance >= radius {
        return 0.0;
    }
    let t = distance / radius;
    (-(t * t * 2.5)).exp()
}

/// Process a complete stroke on the mesh, returning the indices of affected vertices.
pub fn process_stroke(
    engine: &BrushEngine,
    mesh: &mut Mesh,
    stroke: &[StrokePoint],
) -> Vec<usize> {
    let mut affected: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Mirror stroke if symmetry is active
    let mut all_points: Vec<StrokePoint> = stroke.to_vec();
    let mirrored = crate::symmetry::mirror_stroke(stroke, &engine.symmetry);
    all_points.extend(mirrored);

    for sp in &all_points {
        let pressure_scale = if engine.pressure_sensitivity {
            sp.pressure as f64
        } else {
            1.0
        };
        let effective_strength = engine.active_brush.strength * pressure_scale;

        for (i, v) in mesh.vertices.iter_mut().enumerate() {
            let dist = (v.coords - sp.position.coords).norm();
            let w = gaussian_falloff(dist, engine.active_brush.radius) * effective_strength;
            if w < 1e-6 {
                continue;
            }

            *v = match engine.active_brush.brush_type {
                BrushType::Clay | BrushType::Snake => {
                    Point3::from(v.coords + sp.normal * w)
                }
                BrushType::Inflate => {
                    if i < mesh.normals.len() {
                        Point3::from(v.coords + mesh.normals[i] * w)
                    } else {
                        *v
                    }
                }
                BrushType::Move => {
                    let move_dir = sp.normal;
                    Point3::from(v.coords + move_dir * w)
                }
                BrushType::Smooth => {
                    *v // smoothing handled separately
                }
                BrushType::Pinch => {
                    let to_center = (sp.position.coords - v.coords).normalize();
                    Point3::from(v.coords + to_center * w)
                }
                BrushType::Flatten | BrushType::TrimFlat => {
                    let n = sp.normal.normalize();
                    let proj = v.coords - n * n.dot(&(v.coords - sp.position.coords));
                    Point3::from(v.coords + (proj - v.coords) * w)
                }
            };

            affected.insert(i);
        }
    }

    mesh.calculate_normals();
    affected.into_iter().collect()
}

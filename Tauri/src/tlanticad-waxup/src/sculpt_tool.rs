//! Sculpting tools for digital wax-up editing

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Sculpt brush type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushType {
    AddClay,
    SubtractClay,
    Smooth,
    Flatten,
    Pinch,
    Inflate,
    Crease,
}

/// Sculpt brush with size and strength parameters
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
            falloff: 0.7,
            brush_type: BrushType::AddClay,
        }
    }
}

/// Apply a sculpt brush stroke to the mesh at `center` in `direction`.
pub fn apply_brush(mesh: &mut Mesh, center: &Point3<f64>, brush: &SculptBrush, direction: &Vector3<f64>) {
    let dir = direction.normalize();
    let normals_len = mesh.normals.len();

    for i in 0..mesh.vertices.len() {
        let v = mesh.vertices[i];
        let dist = (v - center).norm();
        let w = smooth_falloff(dist, brush.radius, brush.falloff) * brush.strength;
        if w <= 1e-6 {
            continue;
        }

        mesh.vertices[i] = match brush.brush_type {
            BrushType::AddClay => Point3::from(v.coords + dir * w),
            BrushType::SubtractClay => Point3::from(v.coords - dir * w),
            BrushType::Smooth => v, // handled separately in smooth_region
            BrushType::Flatten => {
                let proj = v.coords - dir * dir.dot(&(v.coords - center.coords));
                Point3::from(v.coords + (proj - v.coords) * w)
            }
            BrushType::Pinch => {
                let to_center = (center.coords - v.coords).normalize();
                Point3::from(v.coords + to_center * w)
            }
            BrushType::Inflate => {
                if i < normals_len {
                    Point3::from(v.coords + mesh.normals[i] * w)
                } else {
                    v
                }
            }
            BrushType::Crease => {
                let to_center = (center.coords - v.coords).normalize();
                Point3::from(v.coords + (dir + to_center * 0.5) * w)
            }
        };
    }
    mesh.calculate_normals();
}

/// Apply Laplacian smoothing to a region around `center` within `radius`.
pub fn smooth_region(mesh: &mut Mesh, center: &Point3<f64>, radius: f64, iterations: u32) {
    use std::collections::HashMap;
    let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    adjacency.entry(tri[i]).or_default().push(tri[j]);
                }
            }
        }
    }

    for _ in 0..iterations {
        let orig = mesh.vertices.clone();
        for (idx, neighbors) in &adjacency {
            let v = orig[*idx as usize];
            if (v - center).norm() > radius {
                continue;
            }
            if neighbors.is_empty() {
                continue;
            }
            let avg: Vector3<f64> = neighbors.iter()
                .map(|&n| orig[n as usize].coords)
                .sum::<Vector3<f64>>() / neighbors.len() as f64;
            let w = smooth_falloff((v - center).norm(), radius, 0.7) * 0.5;
            mesh.vertices[*idx as usize] = Point3::from(v.coords.lerp(&avg, w));
        }
    }
    mesh.calculate_normals();
}

/// Push the current mesh state onto an undo stack, trimming if necessary.
pub fn undo_stack_push(stack: &mut Vec<Mesh>, mesh: &Mesh, max_undo: usize) {
    if stack.len() >= max_undo {
        stack.remove(0);
    }
    stack.push(mesh.clone());
}

fn smooth_falloff(distance: f64, radius: f64, falloff: f64) -> f64 {
    if distance >= radius {
        return 0.0;
    }
    let t = distance / radius;
    let linear = 1.0 - t;
    let cubic = linear * linear * (3.0 - 2.0 * linear);
    linear * (1.0 - falloff) + cubic * falloff
}

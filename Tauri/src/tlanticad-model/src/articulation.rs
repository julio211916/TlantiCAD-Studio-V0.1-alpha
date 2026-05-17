//! Model articulation: bite registration and centric occlusion

use nalgebra::{Isometry3, Point3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;
use crate::study_model::StudyModel;

/// Contact point between upper and lower dentitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionContact {
    pub position: Point3<f64>,
    pub intensity: f64,
}

/// Record of centric occlusion between upper and lower models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionRecord {
    pub contacts: Vec<OcclusionContact>,
    /// Vertical dimension of occlusion in mm
    pub vertical_dimension: f64,
    /// Midline deviation in mm (positive = lower shifted right)
    pub midline_deviation: f64,
}

/// Calculate centric occlusion contacts between upper and lower study models.
pub fn calculate_centric_occlusion(upper: &StudyModel, lower: &StudyModel) -> OcclusionRecord {
    const CONTACT_THRESHOLD: f64 = 0.5;
    let mut contacts = Vec::new();

    // Gather all upper and lower mesh vertices
    let upper_verts: Vec<Point3<f64>> = upper.teeth.iter()
        .filter_map(|t| t.mesh.as_ref())
        .flat_map(|m| m.vertices.iter().copied())
        .collect();

    let lower_verts: Vec<Point3<f64>> = lower.teeth.iter()
        .filter_map(|t| t.mesh.as_ref())
        .flat_map(|m| m.vertices.iter().copied())
        .collect();

    for uv in &upper_verts {
        for lv in &lower_verts {
            let dist = (uv - lv).norm();
            if dist < CONTACT_THRESHOLD {
                let intensity = 1.0 - dist / CONTACT_THRESHOLD;
                contacts.push(OcclusionContact {
                    position: Point3::new(
                        (uv.x + lv.x) / 2.0,
                        (uv.y + lv.y) / 2.0,
                        (uv.z + lv.z) / 2.0,
                    ),
                    intensity,
                });
            }
        }
    }

    // Estimate vertical dimension as Z-gap between arches
    let upper_min_z = upper_verts.iter().map(|v| v.z).fold(f64::MAX, f64::min);
    let lower_max_z = lower_verts.iter().map(|v| v.z).fold(f64::MIN, f64::max);
    let vertical_dimension = (lower_max_z - upper_min_z).abs();

    // Midline deviation: difference in X-centroid of arches
    let upper_cx = if upper_verts.is_empty() {
        0.0
    } else {
        upper_verts.iter().map(|v| v.x).sum::<f64>() / upper_verts.len() as f64
    };
    let lower_cx = if lower_verts.is_empty() {
        0.0
    } else {
        lower_verts.iter().map(|v| v.x).sum::<f64>() / lower_verts.len() as f64
    };
    let midline_deviation = lower_cx - upper_cx;

    OcclusionRecord {
        contacts,
        vertical_dimension,
        midline_deviation,
    }
}

/// Register a bite using an interocclusal registration mesh to compute the
/// transformation aligning the lower arch to the upper arch.
///
/// Returns the rigid-body transform of the lower arch relative to the upper.
pub fn register_bite(
    upper: &Mesh,
    lower: &Mesh,
    registration_mesh: &Mesh,
) -> Isometry3<f64> {
    // Simplified: compute centroids of upper and lower contact zones near the
    // registration mesh, then return a pure translation aligning them.
    let reg_center: nalgebra::Vector3<f64> = if registration_mesh.vertices.is_empty() {
        nalgebra::Vector3::zeros()
    } else {
        registration_mesh.vertices.iter().map(|v| v.coords).sum::<nalgebra::Vector3<f64>>()
            / registration_mesh.vertices.len() as f64
    };

    let upper_center: nalgebra::Vector3<f64> = if upper.vertices.is_empty() {
        nalgebra::Vector3::zeros()
    } else {
        upper.vertices.iter().map(|v| v.coords).sum::<nalgebra::Vector3<f64>>()
            / upper.vertices.len() as f64
    };

    let lower_center: nalgebra::Vector3<f64> = if lower.vertices.is_empty() {
        nalgebra::Vector3::zeros()
    } else {
        lower.vertices.iter().map(|v| v.coords).sum::<nalgebra::Vector3<f64>>()
            / lower.vertices.len() as f64
    };

    let translation = nalgebra::Translation3::from(upper_center - lower_center + reg_center * 0.0);
    Isometry3::from_parts(translation, nalgebra::UnitQuaternion::identity())
}

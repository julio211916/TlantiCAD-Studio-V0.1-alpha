//! Implant positioning algorithms and axis suggestion

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;
use crate::library::ImplantDefinition;

/// Implant axis defined by an origin point and direction vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantAxis {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
}

impl ImplantAxis {
    /// Create a new implant axis, normalizing the direction
    pub fn new(origin: Point3<f64>, direction: Vector3<f64>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Create a default vertical axis at the given position
    pub fn vertical(origin: Point3<f64>) -> Self {
        Self {
            origin,
            direction: Vector3::new(0.0, 0.0, -1.0), // apical direction
        }
    }
}

/// Proposed implant placement with full spatial parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantPlacement {
    pub implant: ImplantDefinition,
    pub axis: ImplantAxis,
    /// Depth below bone crest in mm
    pub depth: f64,
    /// Angulation relative to occlusal plane in degrees
    pub angulation: f64,
    /// Rotation around implant long axis in degrees
    pub rotation: f64,
}

/// Result of bone volume adequacy assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneVolumeResult {
    pub available_height: f64,
    pub cortical_thickness: f64,
    pub cancellous_volume: f64,
    pub is_adequate: bool,
}

/// Suggest an implant axis based on bone mesh geometry and a click point.
///
/// Casts a ray from the click point inward along the average normal of
/// the nearest bone surface vertices, providing a clinically reasonable
/// starting axis.
pub fn suggest_axis_from_bone(bone_mesh: &Mesh, click_point: &Point3<f64>) -> ImplantAxis {
    // Find nearest bone surface vertices within 5 mm
    let radius = 5.0;
    let mut avg_normal = Vector3::zeros();
    let mut count = 0usize;

    for (i, v) in bone_mesh.vertices.iter().enumerate() {
        let dist = (v - click_point).norm();
        if dist < radius && i < bone_mesh.normals.len() {
            avg_normal += bone_mesh.normals[i];
            count += 1;
        }
    }

    let direction = if count > 0 {
        let n = avg_normal / count as f64;
        if n.norm() > 1e-6 { -n.normalize() } else { Vector3::new(0.0, 0.0, -1.0) }
    } else {
        Vector3::new(0.0, 0.0, -1.0)
    };

    ImplantAxis {
        origin: *click_point,
        direction,
    }
}

/// Assess bone volume available at a proposed placement site.
///
/// Uses bounding box of mesh vertices along the implant axis to estimate
/// available height, cortical thickness, and cancellous volume.
pub fn check_bone_volume(bone_mesh: &Mesh, placement: &ImplantPlacement) -> BoneVolumeResult {
    let axis_dir = placement.axis.direction.normalize();
    let origin = placement.axis.origin;
    let radius = placement.implant.diameter / 2.0 + 1.0;

    // Project bone vertices onto the axis and collect those within the implant radius
    let mut min_proj = f64::MAX;
    let mut max_proj = f64::MIN;
    let mut points_in_column = 0usize;

    for v in &bone_mesh.vertices {
        let to_v = v - origin;
        let radial = to_v - axis_dir * axis_dir.dot(&to_v);
        if radial.norm() < radius {
            let proj = axis_dir.dot(&to_v);
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
            points_in_column += 1;
        }
    }

    let available_height = if max_proj > min_proj { max_proj - min_proj } else { 0.0 };
    // Estimate cortical as 2 mm if enough bone present
    let cortical_thickness = if available_height > 2.0 { 2.0 } else { available_height };
    let cancellous_volume = std::f64::consts::PI
        * (placement.implant.diameter / 2.0).powi(2)
        * (available_height - cortical_thickness).max(0.0);
    let is_adequate = available_height >= placement.implant.length
        && cancellous_volume > 0.0
        && points_in_column > 5;

    BoneVolumeResult {
        available_height,
        cortical_thickness,
        cancellous_volume,
        is_adequate,
    }
}

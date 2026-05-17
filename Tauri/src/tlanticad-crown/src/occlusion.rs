//! Occlusal contact analysis and occlusal height adjustment

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Type of occlusal contact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContactType {
    Centric,
    Working,
    Balancing,
    Protrusive,
}

/// A single occlusal contact between crown and antagonist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionContact {
    pub position: Point3<f64>,
    pub intensity: f64,
    pub contact_type: ContactType,
}

/// Analyse all occlusal contacts between crown and antagonist meshes.
///
/// Returns contacts where crown-antagonist vertex distance is below the
/// contact threshold (0.2 mm), sorted by ascending gap distance.
pub fn analyze_occlusion(crown: &Mesh, antagonist: &Mesh) -> Vec<OcclusionContact> {
    const CONTACT_THRESHOLD: f64 = 0.2;
    let mut contacts = Vec::new();

    for cv in &crown.vertices {
        let mut min_dist = f64::MAX;
        for av in &antagonist.vertices {
            let d = (cv - av).norm();
            if d < min_dist {
                min_dist = d;
            }
        }
        if min_dist < CONTACT_THRESHOLD {
            let intensity = 1.0 - (min_dist / CONTACT_THRESHOLD).clamp(0.0, 1.0);
            contacts.push(OcclusionContact {
                position: *cv,
                intensity,
                contact_type: ContactType::Centric,
            });
        }
    }

    contacts.sort_by(|a, b| b.intensity.partial_cmp(&a.intensity).unwrap_or(std::cmp::Ordering::Equal));
    contacts
}

/// Adjust the crown's occlusal surface so that the minimum distance to
/// the antagonist equals `target_gap` mm.
///
/// Translates all crown vertices vertically by the necessary offset.
pub fn adjust_occlusal_height(crown: &mut Mesh, antagonist: &Mesh, target_gap: f64) {
    // Find the current minimum distance between crown and antagonist
    let mut min_dist = f64::MAX;
    let mut max_z_crown = f64::MIN;

    for cv in &crown.vertices {
        max_z_crown = max_z_crown.max(cv.z);
        for av in &antagonist.vertices {
            let d = (cv - av).norm();
            if d < min_dist {
                min_dist = d;
            }
        }
    }

    if min_dist == f64::MAX || min_dist <= 0.0 {
        return;
    }

    let correction = target_gap - min_dist;
    if correction.abs() < 0.001 {
        return; // already within tolerance
    }

    // Move crown vertices away from antagonist (assume Z-axis direction)
    let sign = if max_z_crown < antagonist.vertices.iter().map(|v| v.z).fold(f64::MAX, f64::min) {
        -1.0
    } else {
        1.0
    };

    for v in &mut crown.vertices {
        v.z += correction * sign;
    }
    crown.calculate_normals();
}

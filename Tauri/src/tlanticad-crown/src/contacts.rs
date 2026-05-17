//! Proximal and occlusal contact adjustment

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Contact point target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTarget {
    /// Target contact position
    pub position: [f64; 3],
    /// Contact direction (toward adjacent/antagonist)
    pub direction: [f64; 3],
    /// Desired contact intensity (0 = none, 1 = tight)
    pub intensity: f64,
    /// Contact area radius (mm)
    pub area_radius: f64,
}

/// Adjust crown vertices to achieve proximal contacts
pub fn adjust_proximal_contacts(
    vertices: &mut [Point3<f64>],
    targets: &[ContactTarget],
    max_displacement: f64,
) {
    for target in targets {
        let target_pos = Point3::new(target.position[0], target.position[1], target.position[2]);
        let direction = Vector3::new(target.direction[0], target.direction[1], target.direction[2]).normalize();

        for v in vertices.iter_mut() {
            let dist = (v.coords - target_pos.coords).norm();
            if dist < target.area_radius {
                let falloff = 1.0 - (dist / target.area_radius).powi(2);
                let displacement = direction * falloff * target.intensity * max_displacement;
                v.coords += displacement;
            }
        }
    }
}

/// Adjust crown vertices for occlusal contact optimization
///
/// Moves occlusal surface vertices to maintain specified clearance
/// from the antagonist, while preserving cusp tips at contact targets.
pub fn adjust_occlusal_surface(
    vertices: &mut [Point3<f64>],
    antagonist_verts: &[Point3<f64>],
    occlusal_direction: &Vector3<f64>,
    min_clearance: f64,
    contact_targets: &[ContactTarget],
) {
    let dir = occlusal_direction.normalize();

    for v in vertices.iter_mut() {
        // Find closest antagonist point
        let mut min_dist = f64::MAX;
        for av in antagonist_verts {
            let diff = av - &*v;
            let proj = diff.dot(&dir);
            if proj > 0.0 && proj < min_dist {
                min_dist = proj;
            }
        }

        // Skip vertices near contact targets
        let near_contact = contact_targets.iter().any(|ct| {
            let ct_pos = Point3::new(ct.position[0], ct.position[1], ct.position[2]);
            (v.coords - ct_pos.coords).norm() < ct.area_radius
        });

        if !near_contact && min_dist < min_clearance && min_dist < f64::MAX {
            // Push vertex away from antagonist
            let push = (min_clearance - min_dist) * 0.5;
            v.coords -= dir * push;
        }
    }
}

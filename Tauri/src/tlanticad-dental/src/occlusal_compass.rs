//! S137: Occlusal compass — directional analysis of occlusal contacts.
//!
//! Classifies occlusal contacts into mediotrusive, laterotrusive, protrusive,
//! and centric directions.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Direction of an occlusal contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactDirection {
    Centric,
    Protrusive,
    Mediotrusive,
    Laterotrusive,
}

/// A classified occlusal contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompassContact {
    pub position: [f64; 3],
    pub direction: ContactDirection,
    /// Angle from occlusal plane normal (degrees).
    pub angle_deg: f64,
    /// Depth of contact (mm).
    pub depth: f64,
}

/// Occlusal compass result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusalCompassResult {
    pub contacts: Vec<CompassContact>,
    pub centric_count: usize,
    pub protrusive_count: usize,
    pub mediotrusive_count: usize,
    pub laterotrusive_count: usize,
}

/// Analyze occlusal contacts and classify them into compass directions.
///
/// `occlusal_normal` is typically (0,0,1).
/// `mesial_dir` is the mesial direction in the occlusal plane.
pub fn classify_contacts(
    contact_positions: &[Point3<f64>],
    contact_normals: &[Vector3<f64>],
    contact_depths: &[f64],
    occlusal_normal: &Vector3<f64>,
    mesial_dir: &Vector3<f64>,
) -> OcclusalCompassResult {
    let on = occlusal_normal.normalize();
    let mesial = mesial_dir.normalize();
    let buccal = on.cross(&mesial).normalize();

    let mut contacts = Vec::with_capacity(contact_positions.len());
    let mut counts = [0usize; 4]; // centric, protrusive, mediotrusive, laterotrusive

    for (i, pos) in contact_positions.iter().enumerate() {
        let normal = contact_normals[i].normalize();
        let depth = contact_depths[i];

        // Project normal onto occlusal plane
        let projected = normal - on * normal.dot(&on);
        let angle_rad = normal.dot(&on).abs().acos();
        let angle_deg = angle_rad.to_degrees();

        let direction = if projected.norm() < 1e-6 || angle_deg < 10.0 {
            ContactDirection::Centric
        } else {
            let proj_n = projected.normalize();
            let mesial_comp = proj_n.dot(&mesial);
            let buccal_comp = proj_n.dot(&buccal);

            if mesial_comp.abs() > buccal_comp.abs() {
                ContactDirection::Protrusive
            } else if buccal_comp > 0.0 {
                ContactDirection::Laterotrusive
            } else {
                ContactDirection::Mediotrusive
            }
        };

        let idx = match direction {
            ContactDirection::Centric => 0,
            ContactDirection::Protrusive => 1,
            ContactDirection::Mediotrusive => 2,
            ContactDirection::Laterotrusive => 3,
        };
        counts[idx] += 1;

        contacts.push(CompassContact {
            position: [pos.x, pos.y, pos.z],
            direction,
            angle_deg,
            depth,
        });
    }

    OcclusalCompassResult {
        contacts,
        centric_count: counts[0],
        protrusive_count: counts[1],
        mediotrusive_count: counts[2],
        laterotrusive_count: counts[3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_centric_contact() {
        let pos = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::new(0.0, 0.0, 1.0)]; // straight down
        let depths = vec![0.1];
        let on = Vector3::new(0.0, 0.0, 1.0);
        let mesial = Vector3::new(0.0, 1.0, 0.0);

        let result = classify_contacts(&pos, &normals, &depths, &on, &mesial);
        assert_eq!(result.centric_count, 1);
        assert_eq!(result.contacts[0].direction, ContactDirection::Centric);
    }

    #[test]
    fn protrusive_contact() {
        let pos = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::new(0.0, 1.0, 0.3).normalize()]; // mostly mesial
        let depths = vec![0.1];
        let on = Vector3::new(0.0, 0.0, 1.0);
        let mesial = Vector3::new(0.0, 1.0, 0.0);

        let result = classify_contacts(&pos, &normals, &depths, &on, &mesial);
        assert_eq!(result.protrusive_count, 1);
    }

    #[test]
    fn empty_contacts() {
        let result = classify_contacts(
            &[],
            &[],
            &[],
            &Vector3::new(0.0, 0.0, 1.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        assert_eq!(result.contacts.len(), 0);
    }
}

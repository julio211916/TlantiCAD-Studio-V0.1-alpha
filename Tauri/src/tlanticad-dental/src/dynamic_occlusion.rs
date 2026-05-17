//! S140: Dynamic occlusion simulation — mandibular movement paths.
//!
//! Simulate protrusive, laterotrusive, and opening/closing movements and
//! record resulting contacts over time.

use nalgebra::{Point3, Vector3, Rotation3};
use serde::{Deserialize, Serialize};

/// Kind of mandibular movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementKind {
    Opening,
    Protrusive,
    LeftLateral,
    RightLateral,
}

/// A single sample in a dynamic occlusion path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionSample {
    pub step: usize,
    pub movement: MovementKind,
    /// Contact points at this step.
    pub contacts: Vec<[f64; 3]>,
    /// Translation applied to lower arch at this step.
    pub translation: [f64; 3],
    /// Rotation angle applied (radians).
    pub rotation_rad: f64,
}

/// Dynamic occlusion result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicOcclusionResult {
    pub samples: Vec<OcclusionSample>,
    pub movement: MovementKind,
    pub max_contacts: usize,
}

/// Simulate dynamic occlusion by stepping the lower arch through a movement.
///
/// Returns contacts at each step. `steps` = number of incremental positions.
pub fn simulate_dynamic_occlusion(
    upper_verts: &[Point3<f64>],
    lower_verts: &[Point3<f64>],
    lower_indices: &[[u32; 3]],
    movement: MovementKind,
    steps: usize,
    max_angle_deg: f64,
) -> DynamicOcclusionResult {
    let steps = steps.max(1);
    let mut samples = Vec::with_capacity(steps);
    let mut max_contacts = 0usize;

    for step in 0..steps {
        let t = step as f64 / (steps - 1).max(1) as f64;
        let (translation, rotation_rad) = movement_at_t(movement, t, max_angle_deg);

        // Transform lower mesh
        let transformed: Vec<Point3<f64>> = lower_verts
            .iter()
            .map(|v| {
                let rotated = apply_rotation(v, movement, rotation_rad);
                Point3::new(
                    rotated.x + translation.x,
                    rotated.y + translation.y,
                    rotated.z + translation.z,
                )
            })
            .collect();

        // Find contacts: upper vertices close to transformed lower surface
        let contacts = find_close_contacts(upper_verts, &transformed, lower_indices, 0.5);
        max_contacts = max_contacts.max(contacts.len());

        samples.push(OcclusionSample {
            step,
            movement,
            contacts,
            translation: [translation.x, translation.y, translation.z],
            rotation_rad,
        });
    }

    DynamicOcclusionResult { samples, movement, max_contacts }
}

fn movement_at_t(movement: MovementKind, t: f64, max_angle_deg: f64) -> (Vector3<f64>, f64) {
    let angle = max_angle_deg.to_radians() * t;
    match movement {
        MovementKind::Opening => {
            let trans = Vector3::new(0.0, 0.0, -10.0 * t); // open 10mm
            (trans, angle)
        }
        MovementKind::Protrusive => {
            let trans = Vector3::new(0.0, 5.0 * t, -2.0 * t); // forward 5mm, down 2mm
            (trans, angle * 0.5)
        }
        MovementKind::LeftLateral => {
            let trans = Vector3::new(-4.0 * t, 1.0 * t, -1.0 * t);
            (trans, angle)
        }
        MovementKind::RightLateral => {
            let trans = Vector3::new(4.0 * t, 1.0 * t, -1.0 * t);
            (trans, angle)
        }
    }
}

fn apply_rotation(point: &Point3<f64>, movement: MovementKind, angle_rad: f64) -> Point3<f64> {
    let axis = match movement {
        MovementKind::Opening | MovementKind::Protrusive => Vector3::x_axis(),
        MovementKind::LeftLateral | MovementKind::RightLateral => Vector3::z_axis(),
    };
    let rot = Rotation3::from_axis_angle(&axis, angle_rad);
    rot * point
}

fn find_close_contacts(
    upper: &[Point3<f64>],
    lower: &[Point3<f64>],
    _lower_indices: &[[u32; 3]],
    threshold: f64,
) -> Vec<[f64; 3]> {
    let threshold_sq = threshold * threshold;
    let mut contacts = Vec::new();

    for uv in upper {
        for lv in lower {
            let d2 = (uv - lv).norm_squared();
            if d2 < threshold_sq {
                contacts.push([uv.x, uv.y, uv.z]);
                break; // one contact per upper vertex
            }
        }
    }

    contacts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_reduces_contacts() {
        let upper = vec![Point3::new(0.0, 0.0, 0.0)];
        let lower = vec![
            Point3::new(0.0, 0.0, 0.1),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let idx = vec![[0, 1, 2]];
        let result = simulate_dynamic_occlusion(
            &upper, &lower, &idx, MovementKind::Opening, 5, 15.0,
        );
        assert_eq!(result.samples.len(), 5);
        // First step should have contacts, last should not (opened 10mm)
        assert!(!result.samples[0].contacts.is_empty());
        assert!(result.samples[4].contacts.is_empty());
    }

    #[test]
    fn protrusive_movement() {
        let upper = vec![Point3::new(0.0, 0.0, 0.0)];
        let lower = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let idx = vec![[0, 1, 2]];
        let result = simulate_dynamic_occlusion(
            &upper, &lower, &idx, MovementKind::Protrusive, 3, 10.0,
        );
        assert_eq!(result.samples.len(), 3);
        assert_eq!(result.movement, MovementKind::Protrusive);
    }
}

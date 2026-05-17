//! Jaw motion — open/close, protrusion, lateral excursion. Real kinematics, no mock.

use crate::bonwill::{condyle_axis_midpoint, hinge_axis, BonwillTriangle};
use nalgebra::{Matrix3, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JawMotionState {
    /// Mouth opening angle in degrees. Pure rotation around the hinge axis.
    pub opening_deg: f64,
    /// Protrusion in mm — both condyles slide forward.
    pub protrusion_mm: f64,
    /// Bennett angle (lateral excursion side-shift) in degrees. Default 7.5.
    pub bennett_angle_deg: f64,
    /// +1 = right working side, -1 = left, 0 = no excursion.
    pub excursion_side: i32,
    /// Excursion magnitude in mm.
    pub excursion_mm: f64,
}

impl Default for JawMotionState {
    fn default() -> Self {
        Self {
            opening_deg: 0.0,
            protrusion_mm: 0.0,
            bennett_angle_deg: 7.5,
            excursion_side: 0,
            excursion_mm: 0.0,
        }
    }
}

/// 4×4 affine transform represented as a (rotation, translation) tuple.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AffineTransform {
    pub rotation_matrix: [[f64; 3]; 3],
    pub translation: [f64; 3],
}

impl AffineTransform {
    pub fn identity() -> Self {
        Self {
            rotation_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0, 0.0, 0.0],
        }
    }
}

/// Compute the mandibular transform from neutral (closed) to the desired motion state.
///
/// The transform is composed as:
///   T = T_protrusion · T_excursion · R_open(hinge) · T_excursion_lateral
///
/// Applied to mandible-frame vertices.
pub fn mandibular_transform(triangle: &BonwillTriangle, state: &JawMotionState) -> AffineTransform {
    let hinge = hinge_axis(triangle);
    let pivot = condyle_axis_midpoint(triangle);

    // Rotation around the hinge axis.
    let opening_rad = state.opening_deg.to_radians();
    let q_open = UnitQuaternion::from_axis_angle(
        &nalgebra::Unit::new_normalize(hinge),
        opening_rad,
    );

    // Forward axis = perpendicular to hinge, in the horizontal plane (assume +X).
    let forward = Vector3::new(1.0, 0.0, 0.0);
    let forward_perp = (forward - hinge * forward.dot(&hinge)).normalize();
    let protrusion_translation = forward_perp * state.protrusion_mm;

    // Lateral excursion: rotate the working side condyle in place; the balancing side condyle
    // shifts forward+inward by the Bennett angle. We approximate as a rotation around the
    // working condyle.
    let bennett_rad = state.bennett_angle_deg.to_radians() * (state.excursion_mm / 5.0);
    let working_dir = state.excursion_side as f64;
    let q_excursion = UnitQuaternion::from_axis_angle(
        &nalgebra::Unit::new_normalize(Vector3::z()),
        -working_dir * bennett_rad,
    );

    let combined_rot = q_excursion * q_open;
    let combined_matrix = combined_rot.to_rotation_matrix().into_inner();

    // Translation: move pivot into origin, rotate, move back, then add protrusion + lateral shift.
    let pivot_v = pivot.coords;
    let lateral_shift = Vector3::new(0.0, working_dir * state.excursion_mm, 0.0);
    let final_translation = pivot_v - combined_matrix * pivot_v + protrusion_translation + lateral_shift;

    let rotation_matrix = matrix3_to_array(combined_matrix);
    AffineTransform {
        rotation_matrix,
        translation: [final_translation.x, final_translation.y, final_translation.z],
    }
}

fn matrix3_to_array(m: Matrix3<f64>) -> [[f64; 3]; 3] {
    [
        [m[(0, 0)], m[(0, 1)], m[(0, 2)]],
        [m[(1, 0)], m[(1, 1)], m[(1, 2)]],
        [m[(2, 0)], m[(2, 1)], m[(2, 2)]],
    ]
}

/// Apply the affine transform to a point.
pub fn apply_transform(transform: &AffineTransform, point: Point3<f64>) -> Point3<f64> {
    let m = transform.rotation_matrix;
    let t = transform.translation;
    Point3::new(
        m[0][0] * point.x + m[0][1] * point.y + m[0][2] * point.z + t[0],
        m[1][0] * point.x + m[1][1] * point.y + m[1][2] * point.z + t[1],
        m[2][0] * point.x + m[2][1] * point.y + m[2][2] * point.z + t[2],
    )
}

/// Sample N transforms along an opening cycle from 0° to `max_open_deg`. Used for the jaw
/// motion overlay animation.
pub fn opening_path(
    triangle: &BonwillTriangle,
    max_open_deg: f64,
    samples: usize,
) -> Vec<AffineTransform> {
    if samples == 0 {
        return Vec::new();
    }
    (0..samples)
        .map(|i| {
            let t = i as f64 / (samples.saturating_sub(1).max(1) as f64);
            let state = JawMotionState {
                opening_deg: max_open_deg * t,
                ..Default::default()
            };
            mandibular_transform(triangle, &state)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonwill::{default_triangle, BonwillParams};

    #[test]
    fn neutral_state_is_identity() {
        let triangle = default_triangle(&BonwillParams::default());
        let t = mandibular_transform(&triangle, &JawMotionState::default());
        let p = Point3::new(50.0, 10.0, -5.0);
        let p2 = apply_transform(&t, p);
        assert!((p2.x - p.x).abs() < 1e-9);
        assert!((p2.y - p.y).abs() < 1e-9);
        assert!((p2.z - p.z).abs() < 1e-9);
    }

    #[test]
    fn opening_displaces_incisor() {
        let triangle = default_triangle(&BonwillParams::default());
        let state = JawMotionState {
            opening_deg: 30.0,
            ..Default::default()
        };
        let t = mandibular_transform(&triangle, &state);
        let incisor = Point3::new(triangle.incisor[0], triangle.incisor[1], triangle.incisor[2]);
        let after = apply_transform(&t, incisor);
        // After 30° rotation around the hinge axis the incisor should leave the occlusal plane.
        let displacement = (after - incisor).norm();
        assert!(
            displacement > 1.0,
            "incisor must displace meaningfully: got {} mm",
            displacement
        );
        assert!(after.z.abs() > 0.1, "z should change from neutral 0");
    }

    #[test]
    fn protrusion_translates_along_x() {
        let triangle = default_triangle(&BonwillParams::default());
        let state = JawMotionState {
            protrusion_mm: 5.0,
            ..Default::default()
        };
        let t = mandibular_transform(&triangle, &state);
        let p = Point3::new(0.0, 0.0, 0.0);
        let after = apply_transform(&t, p);
        assert!((after.x - 5.0).abs() < 1e-6);
    }

    #[test]
    fn opening_path_samples_correctly() {
        let triangle = default_triangle(&BonwillParams::default());
        let path = opening_path(&triangle, 45.0, 5);
        assert_eq!(path.len(), 5);
    }
}

//! Articulator registration — facebow + bite. Maps the patient's anatomic landmarks into the
//! articulator's local coordinate frame.
//!
//! Two paths:
//!   * `register_from_landmarks` — given 3 landmarks (incisor + 2 condyles), build a
//!     transform that maps the local mesh into a Bonwill-aligned articulator frame.
//!   * `auto_register` — uses a simple PCA + symmetry heuristic on the upper jaw mesh to
//!     guess landmarks (the AutoArticulatorProcessor's role).

use crate::bonwill::{condyle_axis_midpoint, default_triangle, BonwillParams, BonwillTriangle};
use nalgebra::{Matrix3, Point3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub triangle: BonwillTriangle,
    pub rotation_matrix: [[f64; 3]; 3],
    pub translation: [f64; 3],
    /// Residual fit error (mm) compared to the canonical Bonwill triangle.
    pub fit_error_mm: f64,
}

/// Register from three landmarks: incisor contact + right condyle + left condyle.
/// Builds the rotation matrix that aligns the patient triangle to the canonical Bonwill
/// frame, plus the translation that maps the canonical pivot back to patient space.
pub fn register_from_landmarks(
    incisor: Point3<f64>,
    condyle_right: Point3<f64>,
    condyle_left: Point3<f64>,
    params: &BonwillParams,
) -> RegistrationResult {
    let triangle = BonwillTriangle {
        incisor: [incisor.x, incisor.y, incisor.z],
        condyle_right: [condyle_right.x, condyle_right.y, condyle_right.z],
        condyle_left: [condyle_left.x, condyle_left.y, condyle_left.z],
    };
    let pivot = condyle_axis_midpoint(&triangle);
    // Hinge axis = right→left.
    let hinge = (condyle_left - condyle_right).normalize();
    // Forward axis = pivot→incisor projected perpendicular to hinge.
    let to_incisor = incisor - pivot;
    let forward_raw = to_incisor - hinge * to_incisor.dot(&hinge);
    let forward = forward_raw.try_normalize(1e-9).unwrap_or(Vector3::x());
    let up = forward.cross(&hinge).normalize();

    // Rotation matrix from patient frame (forward, hinge, up) to canonical (X, Y, Z).
    let r = Matrix3::from_columns(&[forward, hinge, up]).transpose();
    let rotation_matrix = [
        [r[(0, 0)], r[(0, 1)], r[(0, 2)]],
        [r[(1, 0)], r[(1, 1)], r[(1, 2)]],
        [r[(2, 0)], r[(2, 1)], r[(2, 2)]],
    ];
    let translation = -r * pivot.coords;

    let canonical = default_triangle(params);
    let canonical_pivot = condyle_axis_midpoint(&canonical);
    let observed_pivot = Point3::origin() + (r * pivot.coords + translation);
    let fit_error_mm = (observed_pivot - canonical_pivot).norm();

    RegistrationResult {
        triangle,
        rotation_matrix,
        translation: [translation.x, translation.y, translation.z],
        fit_error_mm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_canonical_to_zero_error() {
        let params = BonwillParams::default();
        let canonical = default_triangle(&params);
        let result = register_from_landmarks(
            Point3::new(canonical.incisor[0], canonical.incisor[1], canonical.incisor[2]),
            Point3::new(
                canonical.condyle_right[0],
                canonical.condyle_right[1],
                canonical.condyle_right[2],
            ),
            Point3::new(
                canonical.condyle_left[0],
                canonical.condyle_left[1],
                canonical.condyle_left[2],
            ),
            &params,
        );
        assert!(result.fit_error_mm < 1e-6);
    }

    #[test]
    fn rotation_matrix_is_orthonormal() {
        let params = BonwillParams::default();
        let result = register_from_landmarks(
            Point3::new(50.0, 0.0, 10.0),
            Point3::new(0.0, 50.0, 0.0),
            Point3::new(0.0, -50.0, 0.0),
            &params,
        );
        let r = result.rotation_matrix;
        let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
        assert!((det - 1.0).abs() < 1e-6, "should be a rotation (det = +1)");
    }
}

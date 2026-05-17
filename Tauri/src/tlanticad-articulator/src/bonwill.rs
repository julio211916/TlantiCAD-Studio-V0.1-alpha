//! Bonwill triangle — the equilateral triangle (≈ 4 inches / 10 cm per side) connecting
//! the contact point of the lower central incisors to the centers of each mandibular condyle.
//!
//! Standard textbook parameters:
//!   * Side length: 100 mm (Bonwill 1899)
//!   * Curve of Spee radius: 110 mm
//!   * Balkwill angle (between Bonwill plane and occlusal plane): ~26°
//!
//! The triangle defines the kinematic constraint for the mandible: as it opens, each condyle
//! traces a circular arc around the opposite condyle's hinge point; as it protrudes, both
//! condyles slide forward along the articular eminence.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BonwillTriangle {
    /// Mandibular incisor contact point.
    pub incisor: [f64; 3],
    /// Right (patient's right) condyle center.
    pub condyle_right: [f64; 3],
    /// Left (patient's left) condyle center.
    pub condyle_left: [f64; 3],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BonwillParams {
    /// Side length in mm. Default 100.
    pub side_length_mm: f64,
    /// Balkwill angle in degrees. Default 26.
    pub balkwill_angle_deg: f64,
    /// Curve of Spee radius (mm). Default 110.
    pub curve_of_spee_radius_mm: f64,
}

impl Default for BonwillParams {
    fn default() -> Self {
        Self {
            side_length_mm: 100.0,
            balkwill_angle_deg: 26.0,
            curve_of_spee_radius_mm: 110.0,
        }
    }
}

/// Build a default Bonwill triangle centered at the origin with the patient looking down +X.
/// Right condyle is at +Y, left condyle is at −Y, incisor at +X.
pub fn default_triangle(params: &BonwillParams) -> BonwillTriangle {
    let s = params.side_length_mm.max(1.0);
    let half_w = s / 2.0;
    // Equilateral triangle height = s * sqrt(3)/2.
    let h = s * 3f64.sqrt() / 2.0;
    BonwillTriangle {
        incisor: [h, 0.0, 0.0],
        condyle_right: [0.0, half_w, 0.0],
        condyle_left: [0.0, -half_w, 0.0],
    }
}

/// Average condyle center — used as the rotation hinge for opening/closing.
pub fn condyle_axis_midpoint(triangle: &BonwillTriangle) -> Point3<f64> {
    let r = Vector3::new(
        triangle.condyle_right[0],
        triangle.condyle_right[1],
        triangle.condyle_right[2],
    );
    let l = Vector3::new(
        triangle.condyle_left[0],
        triangle.condyle_left[1],
        triangle.condyle_left[2],
    );
    Point3::from((r + l) * 0.5)
}

/// Hinge axis (transverse condylar axis) — line from right to left condyle.
pub fn hinge_axis(triangle: &BonwillTriangle) -> Vector3<f64> {
    let r = Vector3::new(
        triangle.condyle_right[0],
        triangle.condyle_right[1],
        triangle.condyle_right[2],
    );
    let l = Vector3::new(
        triangle.condyle_left[0],
        triangle.condyle_left[1],
        triangle.condyle_left[2],
    );
    (l - r).normalize()
}

/// Distance from incisor to each condyle (should be ≈ side_length for a valid triangle).
pub fn validate(triangle: &BonwillTriangle) -> (f64, f64, f64) {
    let i = Vector3::new(triangle.incisor[0], triangle.incisor[1], triangle.incisor[2]);
    let r = Vector3::new(
        triangle.condyle_right[0],
        triangle.condyle_right[1],
        triangle.condyle_right[2],
    );
    let l = Vector3::new(
        triangle.condyle_left[0],
        triangle.condyle_left[1],
        triangle.condyle_left[2],
    );
    let ir = (i - r).norm();
    let il = (i - l).norm();
    let rl = (r - l).norm();
    (ir, il, rl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_triangle_is_equilateral() {
        let p = BonwillParams::default();
        let t = default_triangle(&p);
        let (ir, il, rl) = validate(&t);
        // ir and il are equal by symmetry, but rl = side; ir ≈ side too because the triangle is equilateral.
        assert!((rl - p.side_length_mm).abs() < 1e-6);
        assert!((ir - p.side_length_mm).abs() < 1e-6);
        assert!((il - p.side_length_mm).abs() < 1e-6);
    }

    #[test]
    fn hinge_midpoint_centered() {
        let t = default_triangle(&BonwillParams::default());
        let m = condyle_axis_midpoint(&t);
        assert!(m.x.abs() < 1e-9);
        assert!(m.y.abs() < 1e-9);
    }

    #[test]
    fn hinge_axis_unit_length() {
        let t = default_triangle(&BonwillParams::default());
        let a = hinge_axis(&t);
        assert!((a.norm() - 1.0).abs() < 1e-9);
    }
}

use nalgebra as na;
use serde::{Deserialize, Serialize};
use crate::error::{CadError, Result};

// ─── Domain newtype wrappers ───────────────────────────────────────────────

/// Length in millimeters (dental CAD unit)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Millimeters(pub f64);

impl Millimeters {
    pub fn value(self) -> f64 {
        self.0
    }
}

impl std::ops::Add for Millimeters {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Millimeters {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

/// Angle in degrees
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Degrees(pub f64);

impl Degrees {
    pub fn to_radians(self) -> f64 {
        self.0.to_radians()
    }
}

/// FDI tooth number (11–18, 21–28, 31–38, 41–48)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToothNumber(u8);

impl ToothNumber {
    pub fn new(n: u8) -> Result<Self> {
        let quadrant = n / 10;
        let position = n % 10;
        if (1..=4).contains(&quadrant) && (1..=8).contains(&position) {
            Ok(Self(n))
        } else {
            Err(CadError::InvalidToothNumber(n))
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn quadrant(self) -> u8 {
        self.0 / 10
    }

    pub fn position(self) -> u8 {
        self.0 % 10
    }

    pub fn is_maxillary(self) -> bool {
        matches!(self.quadrant(), 1 | 2)
    }

    pub fn is_mandibular(self) -> bool {
        matches!(self.quadrant(), 3 | 4)
    }
}

// ─── 3D Transform ─────────────────────────────────────────────────────────

/// 4×4 affine transform (rotation + scale + translation)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3D {
    pub matrix: na::Matrix4<f32>,
}

impl Transform3D {
    pub fn identity() -> Self {
        Self {
            matrix: na::Matrix4::identity(),
        }
    }

    pub fn from_translation(t: na::Vector3<f32>) -> Self {
        let mut m = na::Matrix4::identity();
        m[(0, 3)] = t.x;
        m[(1, 3)] = t.y;
        m[(2, 3)] = t.z;
        Self { matrix: m }
    }

    pub fn from_rotation(axis: na::Vector3<f32>, angle_rad: f32) -> Self {
        let rot = na::Rotation3::from_axis_angle(
            &na::Unit::new_normalize(axis),
            angle_rad,
        );
        Self {
            matrix: rot.to_homogeneous(),
        }
    }

    pub fn transform_point(&self, p: na::Point3<f32>) -> na::Point3<f32> {
        let hom = self.matrix * na::Vector4::new(p.x, p.y, p.z, 1.0);
        na::Point3::new(hom.x / hom.w, hom.y / hom.w, hom.z / hom.w)
    }

    pub fn transform_vector(&self, v: na::Vector3<f32>) -> na::Vector3<f32> {
        let hom = self.matrix * na::Vector4::new(v.x, v.y, v.z, 0.0);
        na::Vector3::new(hom.x, hom.y, hom.z)
    }

    pub fn inverse(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|m| Self { matrix: m })
    }

    pub fn then(&self, other: &Self) -> Self {
        Self {
            matrix: other.matrix * self.matrix,
        }
    }

    /// Flat column-major f32 array for GPU uniform buffer
    pub fn as_column_major_array(&self) -> [f32; 16] {
        let m = self.matrix;
        [
            m[(0,0)], m[(1,0)], m[(2,0)], m[(3,0)],
            m[(0,1)], m[(1,1)], m[(2,1)], m[(3,1)],
            m[(0,2)], m[(1,2)], m[(2,2)], m[(3,2)],
            m[(0,3)], m[(1,3)], m[(2,3)], m[(3,3)],
        ]
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    #[test]
    fn test_tooth_number_valid() {
        assert!(ToothNumber::new(11).is_ok());
        assert!(ToothNumber::new(48).is_ok());
        assert!(ToothNumber::new(18).is_ok());
    }

    #[test]
    fn test_tooth_number_invalid() {
        assert!(ToothNumber::new(0).is_err());
        assert!(ToothNumber::new(19).is_err());
        assert!(ToothNumber::new(50).is_err());
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform3D::identity();
        let p = Point3::new(1.0f32, 2.0, 3.0);
        let tp = t.transform_point(p);
        assert!((tp - p).norm() < 1e-6);
    }

    #[test]
    fn test_transform_translation() {
        let t = Transform3D::from_translation(nalgebra::Vector3::new(1.0f32, 0.0, 0.0));
        let p = Point3::new(0.0f32, 0.0, 0.0);
        let tp = t.transform_point(p);
        assert!((tp - Point3::new(1.0, 0.0, 0.0)).norm() < 1e-6);
    }
}

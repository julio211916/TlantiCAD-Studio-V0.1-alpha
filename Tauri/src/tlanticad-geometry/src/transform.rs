//! 3D transform operations: translate, rotate, scale, mirror

use nalgebra::{Point3, Vector3, Matrix4, Rotation3, Unit};

/// A 4×4 affine transformation
#[derive(Debug, Clone)]
pub struct Transform {
    matrix: Matrix4<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self { matrix: Matrix4::identity() }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self::default()
    }

    pub fn from_matrix(matrix: Matrix4<f64>) -> Self {
        Self { matrix }
    }

    pub fn matrix(&self) -> &Matrix4<f64> {
        &self.matrix
    }

    /// Translate by a vector
    pub fn translate(mut self, v: Vector3<f64>) -> Self {
        let t = Matrix4::new_translation(&v);
        self.matrix = t * self.matrix;
        self
    }

    /// Rotate about an axis by angle (radians)
    pub fn rotate(mut self, axis: Unit<Vector3<f64>>, angle: f64) -> Self {
        let r = Rotation3::from_axis_angle(&axis, angle);
        self.matrix = r.to_homogeneous() * self.matrix;
        self
    }

    /// Uniform scale
    pub fn scale(mut self, factor: f64) -> Self {
        let s = Matrix4::new_scaling(factor);
        self.matrix = s * self.matrix;
        self
    }

    /// Non-uniform scale
    pub fn scale_xyz(mut self, sx: f64, sy: f64, sz: f64) -> Self {
        let s = Matrix4::new_nonuniform_scaling(&Vector3::new(sx, sy, sz));
        self.matrix = s * self.matrix;
        self
    }

    /// Mirror about a plane defined by origin and normal
    pub fn mirror(mut self, _origin: Point3<f64>, normal: Vector3<f64>) -> Self {
        let n = normal.normalize();
        // Householder reflection matrix
        let reflection = Matrix4::identity()
            - 2.0
                * Matrix4::new(
                    n.x * n.x, n.x * n.y, n.x * n.z, 0.0,
                    n.y * n.x, n.y * n.y, n.y * n.z, 0.0,
                    n.z * n.x, n.z * n.y, n.z * n.z, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                );
        self.matrix = reflection * self.matrix;
        self
    }

    /// Apply the transform to a point
    pub fn apply_point(&self, p: &Point3<f64>) -> Point3<f64> {
        let h = self.matrix * nalgebra::Vector4::new(p.x, p.y, p.z, 1.0);
        Point3::new(h.x / h.w, h.y / h.w, h.z / h.w)
    }

    /// Apply the transform to a direction vector (no translation)
    pub fn apply_vector(&self, v: &Vector3<f64>) -> Vector3<f64> {
        let h = self.matrix * nalgebra::Vector4::new(v.x, v.y, v.z, 0.0);
        Vector3::new(h.x, h.y, h.z)
    }

    /// Compose two transforms: self then other
    pub fn then(self, other: &Transform) -> Self {
        Self {
            matrix: other.matrix * self.matrix,
        }
    }

    /// Inverse transform
    pub fn inverse(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|m| Self { matrix: m })
    }
}

//! Plane equation, signed distance, intersection

use nalgebra::{Point3, Vector3};

/// Infinite plane defined by a point and normal
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub origin: Point3<f64>,
    pub normal: Vector3<f64>,
}

impl Plane {
    pub fn new(origin: Point3<f64>, normal: Vector3<f64>) -> Self {
        Self { origin, normal: normal.normalize() }
    }

    /// Create plane from 3 points (counter-clockwise)
    pub fn from_points(a: &Point3<f64>, b: &Point3<f64>, c: &Point3<f64>) -> Option<Self> {
        let ab = b - a;
        let ac = c - a;
        let n = ab.cross(&ac);
        let len = n.norm();
        if len < 1e-12 { return None; }
        Some(Self { origin: *a, normal: n / len })
    }

    /// Signed distance from point to plane (positive = same side as normal)
    pub fn signed_distance(&self, point: &Point3<f64>) -> f64 {
        self.normal.dot(&(point - self.origin))
    }

    /// Absolute distance from point to plane
    pub fn distance(&self, point: &Point3<f64>) -> f64 {
        self.signed_distance(point).abs()
    }

    /// Project point onto plane
    pub fn project_point(&self, point: &Point3<f64>) -> Point3<f64> {
        point - self.normal * self.signed_distance(point)
    }

    /// Ray-plane intersection: returns parameter t (point = origin + t * dir)
    pub fn ray_intersect(&self, ray_origin: &Point3<f64>, ray_dir: &Vector3<f64>) -> Option<f64> {
        let denom = self.normal.dot(ray_dir);
        if denom.abs() < 1e-12 { return None; }
        let t = self.normal.dot(&(self.origin - ray_origin)) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }

    /// Classify point relative to plane
    pub fn classify(&self, point: &Point3<f64>, epsilon: f64) -> PlaneClassification {
        let d = self.signed_distance(point);
        if d > epsilon { PlaneClassification::Front }
        else if d < -epsilon { PlaneClassification::Back }
        else { PlaneClassification::On }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneClassification {
    Front,
    Back,
    On,
}

//! Solid primitive operations: volume, surface area, contains point, bounding box

use nalgebra::{Point3, Vector3};
use crate::Primitive;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3<f64>,
    pub max: Point3<f64>,
}

impl Aabb {
    pub fn center(&self) -> Point3<f64> {
        Point3::from((self.min.coords + self.max.coords) * 0.5)
    }

    pub fn size(&self) -> Vector3<f64> {
        self.max - self.min
    }

    pub fn contains(&self, p: &Point3<f64>) -> bool {
        p.x >= self.min.x && p.x <= self.max.x
            && p.y >= self.min.y && p.y <= self.max.y
            && p.z >= self.min.z && p.z <= self.max.z
    }

    pub fn expand(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }
}

impl Primitive {
    /// Compute the volume of the primitive
    pub fn volume(&self) -> f64 {
        use std::f64::consts::PI;
        match self {
            Primitive::Box { min, max } => {
                let s = max - min;
                s.x.abs() * s.y.abs() * s.z.abs()
            }
            Primitive::Sphere { radius, .. } => (4.0 / 3.0) * PI * radius.powi(3),
            Primitive::Cylinder { p1, p2, radius } => {
                let h = (p2 - p1).norm();
                PI * radius.powi(2) * h
            }
            Primitive::Cone { p1, p2, r1, r2 } => {
                let h = (p2 - p1).norm();
                (PI * h / 3.0) * (r1.powi(2) + r1 * r2 + r2.powi(2))
            }
            Primitive::Torus { major, minor, .. } => {
                2.0 * PI.powi(2) * major * minor.powi(2)
            }
        }
    }

    /// Compute the surface area of the primitive
    pub fn surface_area(&self) -> f64 {
        use std::f64::consts::PI;
        match self {
            Primitive::Box { min, max } => {
                let s = max - min;
                2.0 * (s.x.abs() * s.y.abs() + s.y.abs() * s.z.abs() + s.x.abs() * s.z.abs())
            }
            Primitive::Sphere { radius, .. } => 4.0 * PI * radius.powi(2),
            Primitive::Cylinder { p1, p2, radius } => {
                let h = (p2 - p1).norm();
                2.0 * PI * radius * (radius + h)
            }
            Primitive::Cone { p1, p2, r1, r2 } => {
                let h = (p2 - p1).norm();
                let slant = (h.powi(2) + (r1 - r2).powi(2)).sqrt();
                PI * (r1.powi(2) + r2.powi(2) + (r1 + r2) * slant)
            }
            Primitive::Torus { major, minor, .. } => {
                4.0 * PI.powi(2) * major * minor
            }
        }
    }

    /// Check if a point is inside the primitive
    pub fn contains(&self, point: &Point3<f64>) -> bool {
        match self {
            Primitive::Box { min, max } => {
                point.x >= min.x && point.x <= max.x
                    && point.y >= min.y && point.y <= max.y
                    && point.z >= min.z && point.z <= max.z
            }
            Primitive::Sphere { center, radius } => {
                (point - center).norm() <= *radius
            }
            Primitive::Cylinder { p1, p2, radius } => {
                let axis = p2 - p1;
                let len = axis.norm();
                if len < 1e-12 { return false; }
                let dir = axis / len;
                let v = point - p1;
                let proj = v.dot(&dir);
                if proj < 0.0 || proj > len { return false; }
                let perp = v - dir * proj;
                perp.norm() <= *radius
            }
            Primitive::Cone { p1, p2, r1, r2 } => {
                let axis = p2 - p1;
                let len = axis.norm();
                if len < 1e-12 { return false; }
                let dir = axis / len;
                let v = point - p1;
                let proj = v.dot(&dir);
                if proj < 0.0 || proj > len { return false; }
                let t = proj / len;
                let r_at = r1 + (r2 - r1) * t;
                let perp = v - dir * proj;
                perp.norm() <= r_at
            }
            Primitive::Torus { center, normal, major, minor } => {
                let n = normal.normalize();
                let v = point - center;
                let proj = v.dot(&n);
                let in_plane = v - n * proj;
                let dist_ring = (in_plane.norm() - major).powi(2) + proj.powi(2);
                dist_ring <= minor.powi(2)
            }
        }
    }

    /// Compute axis-aligned bounding box
    pub fn bounding_box(&self) -> Aabb {
        match self {
            Primitive::Box { min, max } => Aabb { min: *min, max: *max },
            Primitive::Sphere { center, radius } => Aabb {
                min: center - Vector3::new(*radius, *radius, *radius),
                max: center + Vector3::new(*radius, *radius, *radius),
            },
            Primitive::Cylinder { p1, p2, radius } => {
                let min = Point3::new(
                    p1.x.min(p2.x) - radius,
                    p1.y.min(p2.y) - radius,
                    p1.z.min(p2.z) - radius,
                );
                let max = Point3::new(
                    p1.x.max(p2.x) + radius,
                    p1.y.max(p2.y) + radius,
                    p1.z.max(p2.z) + radius,
                );
                Aabb { min, max }
            }
            Primitive::Cone { p1, p2, r1, r2 } => {
                let r = r1.max(*r2);
                Aabb {
                    min: Point3::new(p1.x.min(p2.x) - r, p1.y.min(p2.y) - r, p1.z.min(p2.z) - r),
                    max: Point3::new(p1.x.max(p2.x) + r, p1.y.max(p2.y) + r, p1.z.max(p2.z) + r),
                }
            }
            Primitive::Torus { center, major, minor, .. } => {
                let r = major + minor;
                Aabb {
                    min: center - Vector3::new(r, r, r),
                    max: center + Vector3::new(r, r, r),
                }
            }
        }
    }
}

use nalgebra as na;
use serde::{Deserialize, Serialize};

// Type aliases over nalgebra for convenience
pub type Point3f = na::Point3<f32>;
pub type Vec3f = na::Vector3<f32>;
pub type Mat4f = na::Matrix4<f32>;
pub type Quat = na::UnitQuaternion<f32>;

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Point3f,
    pub max: Point3f,
}

impl Aabb {
    pub fn new(min: Point3f, max: Point3f) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Point3f::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3f::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn expand(&mut self, p: Point3f) {
        self.min = Point3f::new(
            self.min.x.min(p.x),
            self.min.y.min(p.y),
            self.min.z.min(p.z),
        );
        self.max = Point3f::new(
            self.max.x.max(p.x),
            self.max.y.max(p.y),
            self.max.z.max(p.z),
        );
    }

    pub fn center(&self) -> Point3f {
        na::center(&self.min, &self.max)
    }

    pub fn diagonal(&self) -> Vec3f {
        self.max - self.min
    }

    pub fn contains(&self, p: Point3f) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }
}

/// A ray in 3D space: origin + direction (normalized)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Point3f,
    pub direction: Vec3f,
}

impl Ray {
    pub fn new(origin: Point3f, direction: Vec3f) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn at(&self, t: f32) -> Point3f {
        self.origin + self.direction * t
    }

    /// Ray–AABB intersection (slab method). Returns Some(t) if hit.
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv = Vec3f::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min.x - self.origin.x) * inv.x;
        let t2 = (aabb.max.x - self.origin.x) * inv.x;
        let t3 = (aabb.min.y - self.origin.y) * inv.y;
        let t4 = (aabb.max.y - self.origin.y) * inv.y;
        let t5 = (aabb.min.z - self.origin.z) * inv.z;
        let t6 = (aabb.max.z - self.origin.z) * inv.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_expand() {
        let mut aabb = Aabb::empty();
        aabb.expand(Point3f::new(1.0, 2.0, 3.0));
        aabb.expand(Point3f::new(-1.0, 0.0, 5.0));
        assert_eq!(aabb.min, Point3f::new(-1.0, 0.0, 3.0));
        assert_eq!(aabb.max, Point3f::new(1.0, 2.0, 5.0));
    }

    #[test]
    fn test_ray_aabb_hit() {
        let aabb = Aabb::new(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3f::new(0.0, 0.0, -5.0), Vec3f::new(0.0, 0.0, 1.0));
        assert!(ray.intersect_aabb(&aabb).is_some());
    }

    #[test]
    fn test_ray_aabb_miss() {
        let aabb = Aabb::new(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3f::new(5.0, 5.0, -5.0), Vec3f::new(0.0, 0.0, 1.0));
        assert!(ray.intersect_aabb(&aabb).is_none());
    }
}

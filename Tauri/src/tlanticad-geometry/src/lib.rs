//! TlantiCAD Geometry Engine
//!
//! B-Rep operations, curves, surfaces, solids, spatial indexing, analysis

// Boolean CSG
pub mod boolean;
pub use boolean::*;

// Core geometry
pub mod curve;
pub mod surface;
pub mod solid;
pub mod transform;
pub mod plane;
pub mod bbox;

// Spatial indexing
pub mod bvh;
pub mod kdtree;

// Analysis
pub mod curvature;

// AR-V364 — Insertion direction (PCA + bridge unifier + secondary axis + undercut severity)
pub mod insertion;

use nalgebra::{Point3, Vector3};

/// Geometric curve
#[derive(Debug, Clone)]
pub enum Curve {
    Line { start: Point3<f64>, end: Point3<f64> },
    Circle { center: Point3<f64>, radius: f64, normal: Vector3<f64> },
    Ellipse { center: Point3<f64>, major: f64, minor: f64 },
    Bezier { control_points: Vec<Point3<f64>> },
    Nurbs { knots: Vec<f64>, points: Vec<Point3<f64>>, weights: Vec<f64> },
}

/// Geometric surface
#[derive(Debug, Clone)]
pub enum Surface {
    Plane { origin: Point3<f64>, normal: Vector3<f64> },
    Sphere { center: Point3<f64>, radius: f64 },
    Cylinder { origin: Point3<f64>, axis: Vector3<f64>, radius: f64 },
    NurbsSurface { u_knots: Vec<f64>, v_knots: Vec<f64>, points: Vec<Vec<Point3<f64>>> },
}

/// Solid primitives
#[derive(Debug, Clone)]
pub enum Primitive {
    Box { min: Point3<f64>, max: Point3<f64> },
    Sphere { center: Point3<f64>, radius: f64 },
    Cylinder { p1: Point3<f64>, p2: Point3<f64>, radius: f64 },
    Cone { p1: Point3<f64>, p2: Point3<f64>, r1: f64, r2: f64 },
    Torus { center: Point3<f64>, normal: Vector3<f64>, major: f64, minor: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};

    // ── Aabb ────────────────────────────────────────────────────
    #[test]
    fn test_aabb_from_points() {
        let pts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 3.0, 4.0),
            Point3::new(-1.0, 1.0, 2.0),
        ];
        let aabb = bbox::Aabb::from_points(&pts).unwrap();
        assert!((aabb.min.x - (-1.0)).abs() < 1e-6);
        assert!((aabb.max.y - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_center_size() {
        let aabb = bbox::Aabb::new(Point3::origin(), Point3::new(2.0, 4.0, 6.0));
        let c = aabb.center();
        assert!((c.x - 1.0).abs() < 1e-6);
        assert!((aabb.volume() - 48.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_contains_intersects() {
        let a = bbox::Aabb::new(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        assert!(a.contains_point(&Point3::new(1.0, 1.0, 1.0)));
        assert!(!a.contains_point(&Point3::new(3.0, 0.0, 0.0)));
        let b = bbox::Aabb::new(Point3::new(1.0, 1.0, 1.0), Point3::new(3.0, 3.0, 3.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_aabb_merge() {
        let a = bbox::Aabb::new(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = bbox::Aabb::new(Point3::new(2.0, 2.0, 2.0), Point3::new(3.0, 3.0, 3.0));
        let m = a.merge(&b);
        assert!((m.min.x).abs() < 1e-6);
        assert!((m.max.x - 3.0).abs() < 1e-6);
    }

    // ── Plane ───────────────────────────────────────────────────
    #[test]
    fn test_plane_distance() {
        let p = plane::Plane::new(Point3::origin(), Vector3::z());
        assert!((p.signed_distance(&Point3::new(0.0, 0.0, 5.0)) - 5.0).abs() < 1e-6);
        assert!((p.distance(&Point3::new(0.0, 0.0, -3.0)) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_plane_project() {
        let p = plane::Plane::new(Point3::origin(), Vector3::z());
        let proj = p.project_point(&Point3::new(1.0, 2.0, 5.0));
        assert!((proj.z).abs() < 1e-6);
        assert!((proj.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_plane_from_three_points() {
        let plane = plane::Plane::from_points(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
        );
        assert!(plane.is_some());
    }

    // ── Curve ───────────────────────────────────────────────────
    #[test]
    fn test_curve_line_evaluate() {
        let line = Curve::Line {
            start: Point3::origin(),
            end: Point3::new(10.0, 0.0, 0.0),
        };
        let mid = line.evaluate(0.5);
        assert!((mid.x - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_curve_line_length() {
        let line = Curve::Line {
            start: Point3::origin(),
            end: Point3::new(3.0, 4.0, 0.0),
        };
        assert!((line.length(100) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_curve_sample() {
        let c = Curve::Circle {
            center: Point3::origin(),
            radius: 1.0,
            normal: Vector3::z(),
        };
        let pts = c.sample(10);
        assert_eq!(pts.len(), 11); // n+1 points
    }

    // ── Transform ───────────────────────────────────────────────
    #[test]
    fn test_transform_identity() {
        let t = transform::Transform::identity();
        let p = Point3::new(1.0, 2.0, 3.0);
        let r = t.apply_point(&p);
        assert!((r - p).norm() < 1e-6);
    }

    #[test]
    fn test_transform_translate() {
        let t = transform::Transform::identity().translate(Vector3::new(1.0, 0.0, 0.0));
        let r = t.apply_point(&Point3::origin());
        assert!((r.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_scale() {
        let t = transform::Transform::identity().scale(2.0);
        let r = t.apply_point(&Point3::new(1.0, 1.0, 1.0));
        assert!((r.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_inverse() {
        let t = transform::Transform::identity()
            .translate(Vector3::new(5.0, 0.0, 0.0));
        let inv = t.inverse().unwrap();
        let p = t.apply_point(&Point3::origin());
        let back = inv.apply_point(&p);
        assert!(back.coords.norm() < 1e-6);
    }

    // ── Boolean ─────────────────────────────────────────────────
    #[test]
    fn test_boolean_union() {
        let a = tlanticad_mesh::create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = tlanticad_mesh::create_box(
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(1.5, 1.5, 1.5),
        );
        let result = boolean::boolean_op(&a, &b, boolean::BooleanOp::Union);
        assert!(!result.mesh.vertices.is_empty());
    }

    // ── Curvature ───────────────────────────────────────────────
    #[test]
    fn test_curvature_empty() {
        let curv = curvature::compute_curvature(&[], &[]);
        assert!(curv.is_empty());
    }

    #[test]
    fn test_hausdorff_same_points() {
        let pts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
        let (fwd, bwd) = curvature::hausdorff_distance(&pts, &pts);
        assert!(fwd < 1e-6);
        assert!(bwd < 1e-6);
    }
}

//! Anatomic planes — occlusal, Frankfort horizontal, Camper, and the articulator plane tools.
//!
//! Ported from `DentalProcessors/ArticulatorPlanesTool` (65531 LOC). Real plane fitting via
//! least-squares from landmark sets.

use nalgebra::{Matrix3, Point3, SVD, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Plane {
    pub origin: [f64; 3],
    pub normal: [f64; 3],
}

impl Plane {
    pub fn from_three_points(a: Point3<f64>, b: Point3<f64>, c: Point3<f64>) -> Self {
        let n = (b - a).cross(&(c - a)).normalize();
        Self {
            origin: [a.x, a.y, a.z],
            normal: [n.x, n.y, n.z],
        }
    }

    pub fn signed_distance(&self, p: Point3<f64>) -> f64 {
        let n = Vector3::new(self.normal[0], self.normal[1], self.normal[2]);
        let o = Point3::new(self.origin[0], self.origin[1], self.origin[2]);
        (p - o).dot(&n)
    }
}

/// Fit a plane to N≥3 points via least-squares (SVD on the centered covariance).
pub fn fit_plane(points: &[Point3<f64>]) -> Option<Plane> {
    if points.len() < 3 {
        return None;
    }
    let mut centroid = Vector3::zeros();
    for p in points {
        centroid += p.coords;
    }
    centroid /= points.len() as f64;

    let mut cov = Matrix3::zeros();
    for p in points {
        let d = p.coords - centroid;
        cov += d * d.transpose();
    }
    let svd = SVD::new(cov, true, true);
    let v_t = svd.v_t?;
    // Smallest singular vector = plane normal.
    let normal_idx = svd
        .singular_values
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)?;
    let n = v_t.row(normal_idx).transpose().normalize();
    Some(Plane {
        origin: [centroid.x, centroid.y, centroid.z],
        normal: [n.x, n.y, n.z],
    })
}

/// Frankfort horizontal: plane through right and left orbitale + right porion.
pub fn frankfort_plane(
    orbitale_right: Point3<f64>,
    orbitale_left: Point3<f64>,
    porion_right: Point3<f64>,
) -> Plane {
    Plane::from_three_points(orbitale_right, orbitale_left, porion_right)
}

/// Camper plane: through anterior nasal spine + tragion right + tragion left.
pub fn camper_plane(
    anterior_nasal_spine: Point3<f64>,
    tragion_right: Point3<f64>,
    tragion_left: Point3<f64>,
) -> Plane {
    Plane::from_three_points(anterior_nasal_spine, tragion_right, tragion_left)
}

/// Occlusal plane: best-fit plane through the cusp tips of the posterior teeth + the
/// midpoint of the central incisors. SVD fit.
pub fn occlusal_plane(cusp_tips: &[Point3<f64>]) -> Option<Plane> {
    fit_plane(cusp_tips)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_points_define_plane() {
        let plane = Plane::from_three_points(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );
        assert!((plane.normal[2].abs() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn plane_signed_distance_correct_sign() {
        let plane = Plane {
            origin: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        let above = plane.signed_distance(Point3::new(0.0, 0.0, 5.0));
        let below = plane.signed_distance(Point3::new(0.0, 0.0, -3.0));
        assert!((above - 5.0).abs() < 1e-9);
        assert!((below - (-3.0)).abs() < 1e-9);
    }

    #[test]
    fn fit_plane_recovers_xy() {
        let points = vec![
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.5, 0.5, 0.0),
        ];
        let plane = fit_plane(&points).unwrap();
        assert!((plane.normal[2].abs() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn fit_plane_too_few_points_returns_none() {
        let pts = vec![Point3::origin(), Point3::new(1.0, 0.0, 0.0)];
        assert!(fit_plane(&pts).is_none());
    }

    #[test]
    fn frankfort_plane_handles_typical_landmarks() {
        let plane = frankfort_plane(
            Point3::new(60.0, 30.0, 0.0),
            Point3::new(60.0, -30.0, 0.0),
            Point3::new(-60.0, 25.0, -5.0),
        );
        // Normal should be near vertical.
        assert!(plane.normal[2].abs() > 0.5);
    }
}

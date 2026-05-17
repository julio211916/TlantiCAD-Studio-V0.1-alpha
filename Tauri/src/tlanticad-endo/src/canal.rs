//! Canal axis estimation — find the dominant axis of a root canal point cloud.
//!
//! Uses PCA: the smallest eigenvector of the centered covariance is the canal axis (because
//! a canal is roughly a line, so all variance is concentrated in one direction; the
//! perpendicular components are tiny). We pick the eigenvector with the LARGEST eigenvalue
//! (the line direction itself).

use nalgebra::{Matrix3, Point3, SymmetricEigen, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CanalAxis {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    /// Estimated canal length (mm) projected onto the axis.
    pub length_mm: f64,
    /// Strength of the dominant direction (largest eigenvalue / sum). 0..1.
    pub linearity: f64,
}

/// PCA on a canal point cloud — returns the dominant axis aligned to occlusal-down hint
/// (so the axis points "down" into the tooth).
pub fn estimate_canal_axis(points: &[Point3<f64>], occlusal_down: Vector3<f64>) -> Option<CanalAxis> {
    if points.len() < 3 {
        return None;
    }
    let n = points.len() as f64;
    let mut centroid = Vector3::zeros();
    for p in points {
        centroid += p.coords;
    }
    centroid /= n;
    let mut cov = Matrix3::zeros();
    let mut min_proj = f64::INFINITY;
    let mut max_proj = f64::NEG_INFINITY;
    for p in points {
        let d = p.coords - centroid;
        cov += d * d.transpose();
    }
    cov /= n;

    let eig = SymmetricEigen::new(cov);
    let (mut max_idx, mut max_val) = (0usize, eig.eigenvalues[0]);
    for i in 1..3 {
        if eig.eigenvalues[i] > max_val {
            max_val = eig.eigenvalues[i];
            max_idx = i;
        }
    }
    let mut axis = eig.eigenvectors.column(max_idx).into_owned().normalize();
    if axis.dot(&occlusal_down) < 0.0 {
        axis = -axis;
    }
    // Length along the axis.
    for p in points {
        let proj = (p.coords - centroid).dot(&axis);
        if proj < min_proj {
            min_proj = proj;
        }
        if proj > max_proj {
            max_proj = proj;
        }
    }
    let total: f64 = eig.eigenvalues.iter().copied().sum();
    let linearity = if total > 1e-12 {
        max_val / total
    } else {
        0.0
    };
    Some(CanalAxis {
        origin: [centroid.x, centroid.y, centroid.z],
        axis: [axis.x, axis.y, axis.z],
        length_mm: (max_proj - min_proj).max(0.0),
        linearity,
    })
}

// ────────────────────────────────────────────────────────────────────────
// AR-V398 — EndoCore / EndoBottom / Endo insertion direction.
//
// Ports: `DentalProcessors/EndoCoreInsertionDirectionProcessor`,
// `EndoBottomInsertionDirectionProcessor`, `EndoInsertionDirectionProcessor`.
//
// Algorithm:
//   * `compute_core_insertion_axis` — blend the canal axis with the prep-top
//     curvature gradient. The crown core must seat into the prep, so the
//     axis is biased away from the canal direction toward the dominant
//     curvature flow on the occlusal-facing prep top. The blend weight
//     depends on the curvature magnitude (cf. exocad's `AxisEndoCrownPost`
//     fall-back to `AxisInsertion`).
//   * `validate_post_insertion` — checks that a post of given diameter
//     fits inside the canal at the apex (radius). Returns a verdict with
//     a clearance margin in mm.
// ────────────────────────────────────────────────────────────────────────

/// Per-vertex curvature flow at the prep top: a tangential vector indicating
/// the direction of steepest curvature change (used to orient the core seat).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PrepTopCurvature {
    /// Mean curvature gradient direction (unit-length when valid).
    pub gradient: [f64; 3],
    /// Magnitude of the dominant curvature (mm^-1).
    pub magnitude: f64,
}

impl PrepTopCurvature {
    pub fn gradient_vec(&self) -> Vector3<f64> {
        Vector3::new(self.gradient[0], self.gradient[1], self.gradient[2])
    }
}

/// Compute the EndoCore insertion axis from a canal axis + prep-top curvature.
///
/// Returns a unit vector. Rules (faithful to the exocad fallback chain):
///   1. If the curvature magnitude is below `0.05 mm^-1` (essentially flat
///      prep top), the canal axis is used directly.
///   2. Otherwise the result is the canal axis nudged toward the
///      perpendicular component of the curvature gradient. The blend weight
///      is `clamp(magnitude / 0.5, 0.0, 0.4)` — never more than 40%
///      curvature, so the canal still dominates.
///   3. The output is renormalized; if both inputs are zero/NaN, returns
///      `Vector3::z()` as a safe default (occlusal-down convention).
pub fn compute_core_insertion_axis(
    canal_axis: Vector3<f64>,
    prep_top_curvature: PrepTopCurvature,
) -> Vector3<f64> {
    let canal = if canal_axis.norm_squared() > 1e-12 && canal_axis.iter().all(|c| c.is_finite()) {
        canal_axis.normalize()
    } else {
        return Vector3::z();
    };
    let mag = prep_top_curvature.magnitude.abs();
    if mag < 0.05 {
        return canal;
    }
    let grad = prep_top_curvature.gradient_vec();
    if grad.norm_squared() < 1e-12 || grad.iter().any(|c| !c.is_finite()) {
        return canal;
    }
    let grad = grad.normalize();
    // Project gradient onto the plane perpendicular to canal_axis to get the
    // tangential nudge.
    let tangential = grad - canal * canal.dot(&grad);
    if tangential.norm_squared() < 1e-9 {
        return canal;
    }
    let tangential = tangential.normalize();
    let weight = (mag / 0.5).clamp(0.0, 0.4);
    let blended = canal * (1.0 - weight) + tangential * weight;
    if blended.norm_squared() < 1e-12 {
        canal
    } else {
        blended.normalize()
    }
}

/// Verdict for `validate_post_insertion`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PostInsertionVerdict {
    /// Post fits with at least the recommended clearance.
    Ok,
    /// Post fits but clearance is below the recommended `0.05 mm` minimum.
    Tight,
    /// Post does not fit — would collide with canal walls at the apex.
    Collision,
    /// Inputs were not finite or non-positive.
    Invalid,
}

/// Detailed result of post-vs-canal fit check at the apex cross-section.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PostInsertionReport {
    pub verdict: PostInsertionVerdict,
    /// Clearance = canal_radius - post_radius (mm). Negative = collision.
    pub clearance_mm: f64,
    /// Angular deviation between the post axis and canal axis (degrees).
    pub angular_deviation_deg: f64,
}

/// Validate that a cylindrical post of `post_diameter` (mm) fits inside the
/// canal whose axis is `canal_axis` and apex radius is `canal_radius_at_apex`
/// (mm). The recommended minimum clearance is 0.05 mm (50 µm).
pub fn validate_post_insertion(
    canal_axis: Vector3<f64>,
    post_diameter: f64,
    canal_radius_at_apex: f64,
) -> PostInsertionReport {
    if !post_diameter.is_finite()
        || !canal_radius_at_apex.is_finite()
        || post_diameter <= 0.0
        || canal_radius_at_apex <= 0.0
        || canal_axis.iter().any(|c| !c.is_finite())
    {
        return PostInsertionReport {
            verdict: PostInsertionVerdict::Invalid,
            clearance_mm: 0.0,
            angular_deviation_deg: 0.0,
        };
    }
    let post_radius = 0.5 * post_diameter;
    let clearance = canal_radius_at_apex - post_radius;
    let verdict = if clearance < 0.0 {
        PostInsertionVerdict::Collision
    } else if clearance < 0.05 {
        PostInsertionVerdict::Tight
    } else {
        PostInsertionVerdict::Ok
    };
    // Angular deviation between the post axis (assumed = canal axis) and
    // canal axis is 0 here; we expose the slot for callers that pass a
    // different post axis later. For now we measure deviation of the canal
    // axis from a vertical reference (Z-down) just to surface it.
    let canal = if canal_axis.norm_squared() > 1e-12 {
        canal_axis.normalize()
    } else {
        Vector3::z()
    };
    let dot = canal.dot(&Vector3::z()).abs().clamp(0.0, 1.0);
    let angular_deviation_deg = dot.acos().to_degrees();
    PostInsertionReport {
        verdict,
        clearance_mm: clearance,
        angular_deviation_deg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_axis_recovers_z_for_vertical_canal() {
        let pts: Vec<Point3<f64>> = (0..50)
            .map(|i| Point3::new(0.0, 0.0, i as f64 * 0.2))
            .collect();
        let axis = estimate_canal_axis(&pts, -Vector3::z()).unwrap();
        // Note: occlusal_down is -Z so axis flips to point in -Z.
        assert!((axis.axis[2] + 1.0).abs() < 0.05 || (axis.axis[2] - 1.0).abs() < 0.05);
        assert!(axis.linearity > 0.95);
        assert!(axis.length_mm > 9.0);
    }

    #[test]
    fn estimate_axis_too_few_points_returns_none() {
        let pts = vec![Point3::origin(), Point3::new(1.0, 0.0, 0.0)];
        assert!(estimate_canal_axis(&pts, Vector3::z()).is_none());
    }

    #[test]
    fn linearity_is_low_for_isotropic_cloud() {
        let pts: Vec<Point3<f64>> = (0..50)
            .map(|i| {
                let t = i as f64 * 0.3;
                Point3::new(t.cos(), t.sin(), (t * 1.7).sin() * 0.5)
            })
            .collect();
        let axis = estimate_canal_axis(&pts, Vector3::z()).unwrap();
        assert!(axis.linearity < 0.7);
    }

    // ── AR-V398 — EndoCore insertion + post validation ────────

    #[test]
    fn core_axis_returns_canal_when_curvature_is_negligible() {
        let canal = Vector3::new(0.0, 0.0, -1.0);
        let axis = compute_core_insertion_axis(
            canal,
            PrepTopCurvature {
                gradient: [1.0, 0.0, 0.0],
                magnitude: 0.001,
            },
        );
        assert!((axis - canal).norm() < 1e-9);
    }

    #[test]
    fn core_axis_blends_toward_curvature_gradient() {
        let canal = Vector3::new(0.0, 0.0, -1.0);
        // Strong lateral curvature pulls the axis sideways (but not flips).
        let curv = PrepTopCurvature {
            gradient: [1.0, 0.0, 0.0],
            magnitude: 0.5,
        };
        let axis = compute_core_insertion_axis(canal, curv);
        assert!(axis.norm().abs() - 1.0 < 1e-9);
        // Some lateral component appears, but Z still dominates.
        assert!(axis.x.abs() > 0.05);
        assert!(axis.z.abs() > 0.5);
    }

    #[test]
    fn core_axis_falls_back_to_z_for_zero_canal() {
        let axis = compute_core_insertion_axis(
            Vector3::zeros(),
            PrepTopCurvature::default(),
        );
        assert_eq!(axis, Vector3::z());
    }

    #[test]
    fn core_axis_ignores_gradient_parallel_to_canal() {
        let canal = Vector3::new(0.0, 0.0, -1.0);
        // Gradient parallel to canal → projected tangential = 0 → returns canal.
        let curv = PrepTopCurvature {
            gradient: [0.0, 0.0, 1.0],
            magnitude: 0.3,
        };
        let axis = compute_core_insertion_axis(canal, curv);
        assert!((axis - canal).norm() < 1e-6);
    }

    #[test]
    fn validate_post_ok_when_clearance_above_threshold() {
        let r = validate_post_insertion(Vector3::new(0.0, 0.0, -1.0), 1.0, 0.6);
        assert_eq!(r.verdict, PostInsertionVerdict::Ok);
        assert!((r.clearance_mm - 0.1).abs() < 1e-9);
    }

    #[test]
    fn validate_post_tight_when_clearance_below_50um() {
        let r = validate_post_insertion(Vector3::z(), 1.0, 0.52);
        assert_eq!(r.verdict, PostInsertionVerdict::Tight);
        assert!(r.clearance_mm > 0.0 && r.clearance_mm < 0.05);
    }

    #[test]
    fn validate_post_collision_when_post_too_thick() {
        let r = validate_post_insertion(Vector3::z(), 2.0, 0.5);
        assert_eq!(r.verdict, PostInsertionVerdict::Collision);
        assert!(r.clearance_mm < 0.0);
    }

    #[test]
    fn validate_post_invalid_inputs_yield_invalid_verdict() {
        let r = validate_post_insertion(Vector3::z(), -0.5, 0.5);
        assert_eq!(r.verdict, PostInsertionVerdict::Invalid);

        let r2 = validate_post_insertion(Vector3::z(), 1.0, 0.0);
        assert_eq!(r2.verdict, PostInsertionVerdict::Invalid);

        let r3 = validate_post_insertion(Vector3::new(f64::NAN, 0.0, 0.0), 1.0, 1.0);
        assert_eq!(r3.verdict, PostInsertionVerdict::Invalid);
    }
}

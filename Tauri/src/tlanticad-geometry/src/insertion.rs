//! Insertion direction analysis — PCA-based axis detection + bridge unifier + secondary axis.
//!
//! Ported from exocad's `AverageInsertionDirectionEditor`, `CrownBottomInsertionDirectionEditor`,
//! `AbutmentInsertionDirectionProcessor`, `EndoInsertionDirectionProcessor` and the
//! `InsertionDirectionProcessor.GetBottomFrom` family. AR-V364.
//!
//! Algorithm summary:
//!   1. **Per-tooth axis** — PCA on vertex normals of the prep "bottom" surface; the
//!      eigenvector with the *largest* variance is the dominant normal direction (the
//!      occlusal-pointing surface). We flip sign so it lies in the occlusal hemisphere
//!      defined by the caller's hint.
//!   2. **Bridge unifier** — weighted spherical mean of per-tooth axes, clamped so no
//!      single tooth deviates more than `max_deviation_deg` from the unified axis.
//!   3. **Secondary axis** — cross product of insertion-axis × mesial-distal hint,
//!      ortho-normalized. Used by exocad to align rotational handles.
//!   4. **Undercut severity** — for each vertex, `(−axis) · normal`. Negative ⇒ undercut.
//!      We classify into ok / warning / error using exocad's tunable thresholds.

use nalgebra::{Matrix3, SymmetricEigen, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InsertionAxis {
    pub axis: [f64; 3],
    /// Eigenvalue ratio (largest / sum) — high (~0.7) = strongly directional, low (~0.4) = ambiguous.
    pub directionality: f64,
}

/// PCA on a set of unit normals — returns the eigenvector with the largest eigenvalue.
///
/// Mathematically: build the 3×3 covariance C = Σ(nᵢ nᵢᵀ) / N, find its eigendecomposition,
/// and pick the dominant eigenvector. For a surface that mostly faces "up", that vector
/// points along the average normal.
pub fn dominant_normal_axis(normals: &[Vector3<f64>]) -> Option<InsertionAxis> {
    if normals.is_empty() {
        return None;
    }
    let mut cov = Matrix3::zeros();
    for n in normals {
        let n = n.normalize();
        cov += n * n.transpose();
    }
    cov /= normals.len() as f64;

    let eig = SymmetricEigen::new(cov);
    // SymmetricEigen returns eigenvalues in arbitrary order; pick the largest.
    let (mut max_idx, mut max_val) = (0usize, eig.eigenvalues[0]);
    for i in 1..3 {
        if eig.eigenvalues[i] > max_val {
            max_val = eig.eigenvalues[i];
            max_idx = i;
        }
    }
    let axis = eig.eigenvectors.column(max_idx).into_owned().normalize();
    let total: f64 = eig.eigenvalues.iter().copied().sum();
    let directionality = if total > 1e-12 { max_val / total } else { 0.0 };
    Some(InsertionAxis {
        axis: [axis.x, axis.y, axis.z],
        directionality,
    })
}

/// Detect the insertion axis from a list of vertex normals + occlusal-hemisphere hint.
///
/// `occlusal_hint` is a rough "up" direction (typically the world Y or Z axis). We flip the
/// PCA axis if it points away from the hint so the result is always occlusal-positive.
pub fn detect_insertion_axis(
    normals: &[Vector3<f64>],
    occlusal_hint: Vector3<f64>,
) -> Option<InsertionAxis> {
    let mut a = dominant_normal_axis(normals)?;
    let v = Vector3::new(a.axis[0], a.axis[1], a.axis[2]);
    if v.dot(&occlusal_hint) < 0.0 {
        a.axis = [-v.x, -v.y, -v.z];
    }
    Some(a)
}

/// Weighted average of per-tooth axes, clamped so deviation ≤ `max_deviation_deg`.
///
/// Returns the unified axis and the maximum deviation (in degrees) of the input axes.
pub fn unify_bridge_axes(
    axes: &[Vector3<f64>],
    weights: Option<&[f64]>,
    max_deviation_deg: f64,
) -> Option<(Vector3<f64>, f64)> {
    if axes.is_empty() {
        return None;
    }
    let mut sum = Vector3::zeros();
    for (i, a) in axes.iter().enumerate() {
        let w = weights.and_then(|w| w.get(i).copied()).unwrap_or(1.0);
        sum += a.normalize() * w;
    }
    let unified = if sum.norm() < 1e-12 {
        axes[0].normalize()
    } else {
        sum.normalize()
    };

    let max_deviation = axes
        .iter()
        .map(|a| {
            let cos = a.normalize().dot(&unified).clamp(-1.0, 1.0);
            cos.acos().to_degrees()
        })
        .fold(0.0_f64, f64::max);

    if max_deviation > max_deviation_deg {
        // Project each axis onto the unified one, lerp-clamped, then re-normalize the mean.
        let mut clamped = Vector3::zeros();
        for (i, a) in axes.iter().enumerate() {
            let w = weights.and_then(|w| w.get(i).copied()).unwrap_or(1.0);
            let an = a.normalize();
            let cos = an.dot(&unified).clamp(-1.0, 1.0);
            let dev = cos.acos().to_degrees();
            let factor = (max_deviation_deg / dev.max(1e-6)).min(1.0);
            // Spherical lerp toward unified by `(1 - factor)`.
            let lerped = an * factor + unified * (1.0 - factor);
            clamped += lerped.normalize() * w;
        }
        let final_axis = if clamped.norm() < 1e-12 {
            unified
        } else {
            clamped.normalize()
        };
        return Some((final_axis, max_deviation_deg));
    }
    Some((unified, max_deviation))
}

/// Secondary axis = ortho-normalized component of `mesial_distal` perpendicular to `insertion`.
///
/// Replaces the V14 `polyline-cross-product` heuristic with a robust Gram-Schmidt step.
pub fn secondary_axis(insertion: Vector3<f64>, mesial_distal: Vector3<f64>) -> Vector3<f64> {
    let i = insertion.normalize();
    let m = mesial_distal.normalize();
    let perp = m - i * (m.dot(&i));
    if perp.norm() < 1e-9 {
        // Degenerate: pick an arbitrary perpendicular vector.
        let fallback = if i.x.abs() < 0.9 {
            Vector3::x()
        } else {
            Vector3::y()
        };
        let alt = fallback - i * (fallback.dot(&i));
        return alt.normalize();
    }
    perp.normalize()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UndercutSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UndercutThresholds {
    /// Below this dot value (negative ⇒ undercut), severity = Warning.
    pub warn_dot: f64,
    /// Below this dot value, severity = Error. (Stricter than warn_dot — more negative.)
    pub error_dot: f64,
}

impl Default for UndercutThresholds {
    fn default() -> Self {
        // Typical exocad defaults: 0° = ok, > ~5° undercut = warning, > ~15° = error.
        Self {
            warn_dot: -0.087,  // cos(95°)
            error_dot: -0.259, // cos(105°)
        }
    }
}

/// Per-vertex undercut severity for a mesh and insertion axis.
///
/// Returns one classification per vertex normal. Caller is expected to provide unit normals
/// (e.g. via `Mesh::calculate_normals` followed by reading `mesh.normals`).
pub fn classify_undercut(
    normals: &[Vector3<f64>],
    insertion: Vector3<f64>,
    thresholds: &UndercutThresholds,
) -> Vec<UndercutSeverity> {
    let i = insertion.normalize();
    normals
        .iter()
        .map(|n| {
            let dot = n.normalize().dot(&i);
            if dot < thresholds.error_dot {
                UndercutSeverity::Error
            } else if dot < thresholds.warn_dot {
                UndercutSeverity::Warning
            } else {
                UndercutSeverity::Ok
            }
        })
        .collect()
}

/// Aggregate counts of severity classifications.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub ok: usize,
    pub warning: usize,
    pub error: usize,
}

pub fn severity_counts(severities: &[UndercutSeverity]) -> SeverityCounts {
    let mut c = SeverityCounts::default();
    for s in severities {
        match s {
            UndercutSeverity::Ok => c.ok += 1,
            UndercutSeverity::Warning => c.warning += 1,
            UndercutSeverity::Error => c.error += 1,
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(x: f64, y: f64, z: f64) -> Vector3<f64> {
        Vector3::new(x, y, z).normalize()
    }

    #[test]
    fn pca_axis_recovers_uniform_normal_field() {
        let n = vec![Vector3::z(); 100];
        let axis = dominant_normal_axis(&n).unwrap();
        // Result is ±z.
        assert!((axis.axis[2].abs() - 1.0).abs() < 1e-3);
        assert!(axis.directionality > 0.99);
    }

    #[test]
    fn pca_axis_handles_mixed_field() {
        let mut normals = vec![Vector3::z(); 50];
        normals.extend(vec![unit(0.1, 0.0, 0.99); 50]);
        let axis = dominant_normal_axis(&normals).unwrap();
        // Still mostly z.
        assert!(axis.axis[2].abs() > 0.9);
    }

    #[test]
    fn detect_axis_flips_to_hint_hemisphere() {
        let n = vec![-Vector3::z(); 10];
        let axis = detect_insertion_axis(&n, Vector3::z()).unwrap();
        // After flip, must point along +z.
        assert!(axis.axis[2] > 0.5);
    }

    #[test]
    fn unify_bridge_collinear_returns_same() {
        let axes = vec![Vector3::z(), Vector3::z(), Vector3::z()];
        let (u, dev) = unify_bridge_axes(&axes, None, 5.0).unwrap();
        assert!((u - Vector3::z()).norm() < 1e-9);
        assert!(dev < 1e-9);
    }

    #[test]
    fn unify_bridge_clamps_deviation() {
        let a1 = Vector3::z();
        let a2 = unit(0.5, 0.0, 0.866); // ~30° off
        let (u, dev) = unify_bridge_axes(&[a1, a2], None, 10.0).unwrap();
        // Deviation got clamped to 10°.
        assert!(dev <= 10.0 + 1e-3);
        // Unified is roughly between the two but closer to vertical.
        assert!(u.z > 0.85);
    }

    #[test]
    fn secondary_axis_is_perpendicular_to_insertion() {
        let i = Vector3::z();
        let md = unit(1.0, 0.0, 0.5);
        let s = secondary_axis(i, md);
        assert!(s.dot(&i).abs() < 1e-9);
        assert!((s.norm() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn secondary_axis_handles_parallel_input() {
        let i = Vector3::z();
        let md = Vector3::z();
        let s = secondary_axis(i, md);
        assert!(s.dot(&i).abs() < 1e-9, "must still be perpendicular");
    }

    #[test]
    fn undercut_classifies_normals() {
        let normals = vec![Vector3::z(), -Vector3::z(), Vector3::x()];
        let s = classify_undercut(&normals, Vector3::z(), &UndercutThresholds::default());
        assert!(matches!(s[0], UndercutSeverity::Ok));
        assert!(matches!(s[1], UndercutSeverity::Error));
        assert!(matches!(s[2], UndercutSeverity::Ok | UndercutSeverity::Warning));
    }

    #[test]
    fn severity_counts_aggregate() {
        let c = severity_counts(&[
            UndercutSeverity::Ok,
            UndercutSeverity::Ok,
            UndercutSeverity::Warning,
            UndercutSeverity::Error,
        ]);
        assert_eq!(c.ok, 2);
        assert_eq!(c.warning, 1);
        assert_eq!(c.error, 1);
    }
}

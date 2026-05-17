//! Implant safety checks: nerve clearance, adjacent spacing, root proximity

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use crate::positioning::ImplantPlacement;

/// Aggregated safety check result for a single implant placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    pub nerve_clearance_mm: f64,
    pub adjacent_distance_mm: f64,
    pub root_proximity_mm: f64,
    pub is_safe: bool,
    pub warnings: Vec<String>,
}

/// Minimum clearance from a nerve path (polyline) to the implant body.
///
/// Returns the smallest distance from any nerve path point to the
/// closest point on the implant long axis cylinder.
pub fn check_nerve_clearance(placement: &ImplantPlacement, nerve_path: &[Point3<f64>]) -> f64 {
    if nerve_path.is_empty() {
        return f64::MAX;
    }

    let axis_origin = placement.axis.origin;
    let axis_dir = placement.axis.direction.normalize();
    let half_length = placement.implant.length / 2.0;

    nerve_path
        .iter()
        .map(|nerve_pt| {
            let to_pt = nerve_pt - axis_origin;
            let proj = axis_dir.dot(&to_pt).clamp(-half_length, placement.implant.length);
            let closest_on_axis = axis_origin + axis_dir * proj;
            let dist = (nerve_pt - closest_on_axis).norm();
            // Subtract implant radius to get clearance from implant surface
            (dist - placement.implant.diameter / 2.0).max(0.0)
        })
        .fold(f64::MAX, f64::min)
}

/// Centre-to-centre distance between two implant axes at their origins.
pub fn check_adjacent_implant_distance(p1: &ImplantPlacement, p2: &ImplantPlacement) -> f64 {
    (p1.axis.origin - p2.axis.origin).norm()
}

/// Minimum distance from the implant surface to any tooth root apex.
pub fn check_root_proximity(placement: &ImplantPlacement, roots: &[Point3<f64>]) -> f64 {
    if roots.is_empty() {
        return f64::MAX;
    }

    let origin = placement.axis.origin;
    let radius = placement.implant.diameter / 2.0;

    roots
        .iter()
        .map(|r| {
            let dist = (r - origin).norm();
            (dist - radius).max(0.0)
        })
        .fold(f64::MAX, f64::min)
}

/// Run a full safety assessment for a proposed implant placement.
///
/// Checks nerve clearance (≥2 mm), adjacent implant spacing (≥3 mm
/// centre-to-centre), and root proximity (≥1.5 mm).
pub fn full_safety_check(
    placement: &ImplantPlacement,
    nerve: &[Point3<f64>],
    adjacent: &[ImplantPlacement],
    roots: &[Point3<f64>],
) -> SafetyCheck {
    let nerve_clearance = check_nerve_clearance(placement, nerve);
    let adjacent_distance = adjacent
        .iter()
        .map(|adj| check_adjacent_implant_distance(placement, adj))
        .fold(f64::MAX, f64::min);
    let root_proximity = check_root_proximity(placement, roots);

    let mut warnings = Vec::new();

    if nerve_clearance < 2.0 {
        warnings.push(format!(
            "Nerve clearance {:.2} mm is below the 2 mm safety margin",
            nerve_clearance
        ));
    }

    let adj_display = if adjacent_distance == f64::MAX {
        f64::MAX
    } else {
        adjacent_distance
    };
    if adj_display < 3.0 {
        warnings.push(format!(
            "Adjacent implant distance {:.2} mm is below the 3 mm minimum",
            adj_display
        ));
    }

    if root_proximity < 1.5 {
        warnings.push(format!(
            "Root proximity {:.2} mm is below the 1.5 mm safety margin",
            root_proximity
        ));
    }

    let is_safe = warnings.is_empty();

    SafetyCheck {
        nerve_clearance_mm: if nerve_clearance == f64::MAX { 99.0 } else { nerve_clearance },
        adjacent_distance_mm: if adjacent_distance == f64::MAX { 99.0 } else { adjacent_distance },
        root_proximity_mm: if root_proximity == f64::MAX { 99.0 } else { root_proximity },
        is_safe,
        warnings,
    }
}

//! S139: Antagonist analysis — evaluate opposing arch fit.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Antagonist contact point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntagonistContact {
    pub position: [f64; 3],
    pub clearance: f64,
    pub is_interference: bool,
}

/// Antagonist analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntagonistResult {
    pub contacts: Vec<AntagonistContact>,
    pub min_clearance: f64,
    pub max_interference: f64,
    pub interference_area_pct: f64,
}

/// Analyze antagonist (opposing arch) against restoration.
///
/// For each vertex of `upper`, cast a ray along `axis` and find distance
/// to the nearest triangle of `lower`.
pub fn analyze_antagonist(
    upper_verts: &[Point3<f64>],
    lower_verts: &[Point3<f64>],
    lower_indices: &[[u32; 3]],
    axis: &Vector3<f64>,
) -> AntagonistResult {
    let dir = axis.normalize();
    let mut contacts = Vec::new();
    let mut min_clearance = f64::MAX;
    let mut max_interference = 0.0f64;
    let mut interference_count = 0usize;

    for upper_v in upper_verts {
        let mut best_t = f64::MAX;

        // Simple ray-triangle intersection against lower mesh
        for tri in lower_indices {
            let a = lower_verts[tri[0] as usize];
            let b = lower_verts[tri[1] as usize];
            let c = lower_verts[tri[2] as usize];

            if let Some(t) = ray_triangle_intersect(upper_v, &dir, &a, &b, &c) {
                if t.abs() < best_t.abs() {
                    best_t = t;
                }
            }
        }

        if best_t < f64::MAX {
            let is_interference = best_t < 0.0;
            if is_interference {
                interference_count += 1;
                max_interference = max_interference.max(-best_t);
            }
            min_clearance = min_clearance.min(best_t);

            contacts.push(AntagonistContact {
                position: [upper_v.x, upper_v.y, upper_v.z],
                clearance: best_t,
                is_interference,
            });
        }
    }

    let interference_area_pct = if !upper_verts.is_empty() {
        (interference_count as f64 / upper_verts.len() as f64) * 100.0
    } else {
        0.0
    };

    if min_clearance == f64::MAX {
        min_clearance = 0.0;
    }

    AntagonistResult {
        contacts,
        min_clearance,
        max_interference,
        interference_area_pct,
    }
}

/// Möller–Trumbore ray-triangle intersection.
fn ray_triangle_intersect(
    origin: &Point3<f64>,
    dir: &Vector3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> Option<f64> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = dir.cross(&edge2);
    let a = edge1.dot(&h);
    if a.abs() < 1e-10 {
        return None;
    }
    let f = 1.0 / a;
    let s = origin - v0;
    let u = f * s.dot(&h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(&edge1);
    let v = f * dir.dot(&q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * edge2.dot(&q);
    Some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_lower_mesh() {
        let upper = vec![Point3::new(0.0, 0.0, 1.0)];
        let lower = vec![];
        let lower_idx: Vec<[u32; 3]> = vec![];
        let axis = Vector3::new(0.0, 0.0, -1.0);
        let result = analyze_antagonist(&upper, &lower, &lower_idx, &axis);
        assert_eq!(result.contacts.len(), 0);
    }

    #[test]
    fn basic_antagonist_hit() {
        let upper = vec![Point3::new(0.5, 0.25, 1.0)];
        let lower = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        let lower_idx = vec![[0, 1, 2]];
        let axis = Vector3::new(0.0, 0.0, -1.0);
        let result = analyze_antagonist(&upper, &lower, &lower_idx, &axis);
        assert_eq!(result.contacts.len(), 1);
        assert!(!result.contacts[0].is_interference);
        assert!(result.contacts[0].clearance > 0.0);
    }
}

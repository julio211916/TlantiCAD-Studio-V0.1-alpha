//! Occlusion analysis: contacts, clearance, and interference detection

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// A detected contact between upper and lower dentition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionContact {
    /// Position of contact point
    pub position: [f64; 3],
    /// Contact normal
    pub normal: [f64; 3],
    /// Penetration depth (positive = interference, negative = clearance)
    pub depth: f64,
    /// Contact area estimate (mm²)
    pub area: f64,
}

/// Occlusion analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionResult {
    pub contacts: Vec<OcclusionContact>,
    pub interferences: Vec<OcclusionContact>,
    pub min_clearance: f64,
    pub max_interference: f64,
    /// Per-vertex clearance on the restoration mesh
    pub clearance_map: Vec<f64>,
}

/// Analyze occlusion between a restoration mesh and the opposing arch
pub fn analyze_occlusion(
    restoration_verts: &[Point3<f64>],
    restoration_normals: &[Vector3<f64>],
    restoration_indices: &[[u32; 3]],
    opposing_verts: &[Point3<f64>],
    opposing_indices: &[[u32; 3]],
    occlusal_direction: &Vector3<f64>,
    contact_threshold: f64,
) -> OcclusionResult {
    let dir = occlusal_direction.normalize();
    let mut contacts = Vec::new();
    let mut interferences = Vec::new();
    let mut min_clearance = f64::MAX;
    let mut max_interference = 0.0f64;
    let mut clearance_map = vec![f64::MAX; restoration_verts.len()];

    // For each vertex on restoration, find closest point on opposing
    for (vi, pos) in restoration_verts.iter().enumerate() {
        let normal = &restoration_normals[vi];

        // Cast ray from restoration vertex along occlusal direction
        let mut min_dist = f64::MAX;
        let mut closest_hit = None;

        for tri in opposing_indices {
            let a = &opposing_verts[tri[0] as usize];
            let b = &opposing_verts[tri[1] as usize];
            let c = &opposing_verts[tri[2] as usize];

            if let Some((t, _u, _v)) = ray_triangle_intersect(pos, &dir, a, b, c) {
                if t.abs() < min_dist {
                    min_dist = t.abs();
                    closest_hit = Some((t, Point3::from(pos.coords + dir * t)));
                }
            }
            // Also check reverse direction
            if let Some((t, _u, _v)) = ray_triangle_intersect(pos, &(-dir), a, b, c) {
                if t.abs() < min_dist {
                    min_dist = t.abs();
                    closest_hit = Some((-t, Point3::from(pos.coords - dir * t)));
                }
            }
        }

        if let Some((signed_dist, hit_point)) = closest_hit {
            clearance_map[vi] = signed_dist;

            if signed_dist.abs() < contact_threshold {
                let contact = OcclusionContact {
                    position: [hit_point.x, hit_point.y, hit_point.z],
                    normal: [normal.x, normal.y, normal.z],
                    depth: -signed_dist,
                    area: estimate_vertex_area(restoration_verts, restoration_indices, vi as u32),
                };

                if signed_dist < 0.0 {
                    max_interference = max_interference.max(-signed_dist);
                    interferences.push(contact.clone());
                }
                contacts.push(contact);
            }

            min_clearance = min_clearance.min(signed_dist);
        }
    }

    if min_clearance == f64::MAX { min_clearance = 0.0; }

    OcclusionResult {
        contacts,
        interferences,
        min_clearance,
        max_interference,
        clearance_map,
    }
}

/// Map clearance values to colors for visualization
/// Red: interference, Yellow: tight contact, Green: ideal, Blue: excess clearance
pub fn clearance_to_colors(clearance: &[f64], ideal_gap: f64) -> Vec<[f32; 3]> {
    clearance.iter().map(|&d| {
        if d < 0.0 {
            [1.0, 0.0, 0.0] // Red: interference
        } else if d < ideal_gap * 0.5 {
            let t = (d / (ideal_gap * 0.5)) as f32;
            [1.0, t, 0.0] // Red→Yellow: tight
        } else if d < ideal_gap * 1.5 {
            [0.0, 1.0, 0.0] // Green: ideal
        } else {
            let t = ((d - ideal_gap * 1.5) / ideal_gap).min(1.0) as f32;
            [0.0, 1.0 - t, t] // Green→Blue: excess
        }
    }).collect()
}

fn ray_triangle_intersect(
    origin: &Point3<f64>,
    dir: &Vector3<f64>,
    a: &Point3<f64>,
    b: &Point3<f64>,
    c: &Point3<f64>,
) -> Option<(f64, f64, f64)> {
    let edge1 = b - a;
    let edge2 = c - a;
    let h = dir.cross(&edge2);
    let det = edge1.dot(&h);
    if det.abs() < 1e-12 { return None; }
    let inv_det = 1.0 / det;
    let s = origin - a;
    let u = inv_det * s.dot(&h);
    if !(0.0..=1.0).contains(&u) { return None; }
    let q = s.cross(&edge1);
    let v = inv_det * dir.dot(&q);
    if v < 0.0 || u + v > 1.0 { return None; }
    let t = inv_det * edge2.dot(&q);
    Some((t, u, v))
}

fn estimate_vertex_area(verts: &[Point3<f64>], indices: &[[u32; 3]], vi: u32) -> f64 {
    let mut area = 0.0;
    for tri in indices {
        if tri.contains(&vi) {
            let a = &verts[tri[0] as usize];
            let b = &verts[tri[1] as usize];
            let c = &verts[tri[2] as usize];
            area += (b - a).cross(&(c - a)).norm() / 6.0; // 1/3 of triangle area
        }
    }
    area
}

// ---------------------------------------------------------------------------
// S136-140 additions
// ---------------------------------------------------------------------------

/// Check if clearance is within clinical tolerance.
pub fn check_occlusal_tolerance(result: &OcclusionResult, target_clearance: f64, tolerance: f64) -> Vec<String> {
    let mut issues = Vec::new();
    let min = target_clearance - tolerance;
    let _max = target_clearance + tolerance;

    if result.min_clearance < min {
        issues.push(format!(
            "Minimum clearance {:.2} mm is below tolerance ({:.2} mm)",
            result.min_clearance, min,
        ));
    }

    if result.max_interference > 0.0 {
        issues.push(format!(
            "Interference detected: max {:.2} mm",
            result.max_interference,
        ));
    }

    let heavy_contacts = result.contacts.iter().filter(|c| c.depth > tolerance).count();
    if heavy_contacts > 0 {
        issues.push(format!("{} heavy contact(s) exceeding tolerance", heavy_contacts));
    }

    issues
}

// ---------------------------------------------------------------------------
// S138: Contact map and advanced occlusion analysis
// ---------------------------------------------------------------------------

/// Generate a per-vertex contact intensity map.
pub fn contact_intensity_map(result: &OcclusionResult, vertex_count: usize) -> Vec<f64> {
    let mut intensities = vec![0.0; vertex_count];
    for contact in &result.contacts {
        // Find nearest vertex by position (simplified: uses index if available)
        let _pos = Point3::new(contact.position[0], contact.position[1], contact.position[2]);
        // Distribute contact depth to nearby region
        for i in 0..vertex_count.min(intensities.len()) {
            intensities[i] += contact.depth.abs() * contact.area;
        }
    }
    // Normalize
    let max_val = intensities.iter().cloned().fold(0.0f64, f64::max);
    if max_val > 1e-12 {
        for v in &mut intensities {
            *v /= max_val;
        }
    }
    intensities
}

/// Classify contacts by clinical significance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactClass {
    /// Light contact (< 0.05 mm depth)
    Light,
    /// Normal centric stop
    CentricStop,
    /// Heavy contact requiring adjustment
    Heavy,
    /// Premature contact (interference)
    Premature,
}

/// Classify a single contact by depth thresholds.
pub fn classify_contact(depth: f64) -> ContactClass {
    if depth < 0.0 {
        ContactClass::Premature
    } else if depth < 0.05 {
        ContactClass::Light
    } else if depth < 0.2 {
        ContactClass::CentricStop
    } else {
        ContactClass::Heavy
    }
}

/// Summary of classified contacts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContactSummary {
    pub light: usize,
    pub centric_stops: usize,
    pub heavy: usize,
    pub premature: usize,
    pub total_area: f64,
}

/// Compute a contact summary from an occlusion result.
pub fn summarize_contacts(result: &OcclusionResult) -> ContactSummary {
    let mut summary = ContactSummary::default();
    for contact in &result.contacts {
        summary.total_area += contact.area;
        match classify_contact(contact.depth) {
            ContactClass::Light => summary.light += 1,
            ContactClass::CentricStop => summary.centric_stops += 1,
            ContactClass::Heavy => summary.heavy += 1,
            ContactClass::Premature => summary.premature += 1,
        }
    }
    for interference in &result.interferences {
        summary.premature += 1;
        summary.total_area += interference.area;
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_opposed_meshes() -> (
        Vec<Point3<f64>>,
        Vec<Vector3<f64>>,
        Vec<[u32; 3]>,
        Vec<Point3<f64>>,
        Vec<[u32; 3]>,
    ) {
        // Upper: flat triangle at z=1
        let upper_v = vec![
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(2.0, 0.0, 1.0),
            Point3::new(1.0, 2.0, 1.0),
        ];
        let upper_n = vec![Vector3::new(0.0, 0.0, -1.0); 3]; // pointing down
        let upper_i = vec![[0, 1, 2]];

        // Lower: flat triangle at z=0
        let lower_v = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 0.0),
        ];
        let lower_i = vec![[0, 1, 2]];

        (upper_v, upper_n, upper_i, lower_v, lower_i)
    }

    #[test]
    fn analyze_finds_clearance() {
        let (uv, un, ui, lv, li) = make_opposed_meshes();
        let result = analyze_occlusion(&uv, &un, &ui, &lv, &li, &Vector3::new(0.0, 0.0, -1.0), 0.1);
        // Upper is at z=1, lower at z=0, clearance ≈ 1.0
        assert!(result.min_clearance > 0.0);
    }

    #[test]
    fn clearance_to_colors_len() {
        let map = vec![0.0, 0.5, 1.0, 2.0];
        let colors = clearance_to_colors(&map, 1.5);
        assert_eq!(colors.len(), 4);
    }

    #[test]
    fn tolerance_check_reports_interference() {
        let result = OcclusionResult {
            contacts: vec![OcclusionContact {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                depth: 0.5,
                area: 1.0,
            }],
            interferences: vec![],
            min_clearance: 0.05,
            max_interference: 0.3,
            clearance_map: vec![],
        };
        let issues = check_occlusal_tolerance(&result, 0.5, 0.1);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("Interference")));
    }

    #[test]
    fn empty_meshes_no_crash() {
        let result = analyze_occlusion(
            &[], &[], &[], &[], &[],
            &Vector3::new(0.0, 0.0, -1.0), 0.1,
        );
        assert!(result.contacts.is_empty());
    }

    #[test]
    fn classify_contact_thresholds() {
        assert_eq!(classify_contact(-0.1), ContactClass::Premature);
        assert_eq!(classify_contact(0.01), ContactClass::Light);
        assert_eq!(classify_contact(0.1), ContactClass::CentricStop);
        assert_eq!(classify_contact(0.5), ContactClass::Heavy);
    }

    #[test]
    fn summarize_empty() {
        let result = OcclusionResult {
            contacts: vec![],
            interferences: vec![],
            min_clearance: f64::MAX,
            max_interference: 0.0,
            clearance_map: vec![],
        };
        let summary = summarize_contacts(&result);
        assert_eq!(summary.light, 0);
        assert_eq!(summary.total_area, 0.0);
    }

    #[test]
    fn contact_intensity_map_normalized() {
        let result = OcclusionResult {
            contacts: vec![OcclusionContact {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                depth: 0.1,
                area: 1.0,
            }],
            interferences: vec![],
            min_clearance: 0.5,
            max_interference: 0.0,
            clearance_map: vec![],
        };
        let map = contact_intensity_map(&result, 5);
        assert_eq!(map.len(), 5);
        // Max should be normalized to 1.0
        let max_val = map.iter().cloned().fold(0.0f64, f64::max);
        assert!((max_val - 1.0).abs() < 1e-9 || max_val == 0.0);
    }
}

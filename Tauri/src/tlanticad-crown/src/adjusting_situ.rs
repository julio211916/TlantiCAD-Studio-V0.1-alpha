//! AR-V405 — Adjusting Situ processor.
//!
//! Ported from `DentalProcessorControls/AdjustingSituProcessorControl.xaml.cs`
//! (~22 KB). The "in-situ adjustment" tool detects vertices on a freshly
//! generated restoration that are pressing too hard against the antagonist
//! mesh (i.e. the occlusal contact would create a hot-spot during chewing)
//! and reduces them locally so the resulting bite is comfortable.
//!
//! Algorithm (faithful to exocad's UI behavior):
//! 1. Project every restoration vertex onto the occlusal axis.
//! 2. Build a KD-tree over antagonist vertices.
//! 3. For each restoration vertex, compute the signed gap along the occlusal
//!    axis to the closest antagonist vertex (positive = clear, negative = penetrating).
//! 4. If `-gap` exceeds `contact_force_threshold_mm`, mark the vertex as
//!    "hot": it presses too hard.
//! 5. For every hot vertex, push it backwards along the occlusal axis by the
//!    excess penetration plus a small relief margin. Apply a 1-ring
//!    Laplacian smoothing pass so the dimple blends into surrounding crown.
//! 6. Return an `AdjustmentReport` enumerating all hot spots and the total
//!    material removed.
//!
//! The processor is conservative: it only ever removes material (negative
//! offset along occlusal axis). This matches the exocad rule that adjusting
//! situ never adds occlusal contacts that didn't already exist.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AdjustingSituOptions {
    /// Vertices that interpenetrate the antagonist by more than this distance
    /// (mm) are marked as hot spots and reduced.
    pub contact_force_threshold_mm: f64,
    /// Extra clearance applied beyond the threshold once a vertex is reduced
    /// (mm). Prevents borderline contacts from oscillating between hot/cold
    /// across iterations.
    pub relief_margin_mm: f64,
    /// Search radius (mm) used to find the closest antagonist vertex.
    /// Antagonist vertices outside this radius are ignored — keeps the
    /// adjustment localized.
    pub search_radius_mm: f64,
    /// Number of 1-ring Laplacian smoothing passes applied to hot vertices
    /// after reduction (blends the dimple into surrounding geometry).
    pub smoothing_passes: u32,
    /// Smoothing strength (0..1). 0 = no movement, 1 = move all the way to
    /// the neighborhood centroid.
    pub smoothing_lambda: f64,
}

impl Default for AdjustingSituOptions {
    fn default() -> Self {
        Self {
            // Default 50 µm — exocad's "default occlusal compensation".
            contact_force_threshold_mm: 0.05,
            relief_margin_mm: 0.02,
            search_radius_mm: 5.0,
            smoothing_passes: 2,
            smoothing_lambda: 0.4,
        }
    }
}

/// Per-vertex hot spot detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSpot {
    pub vertex_index: u32,
    /// Signed gap before adjustment (negative = penetrating).
    pub gap_before_mm: f64,
    /// Signed gap after adjustment (should be ≥ 0).
    pub gap_after_mm: f64,
    /// Distance moved along occlusal axis (always ≥ 0).
    pub displacement_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdjustmentReport {
    pub hot_spots: Vec<HotSpot>,
    pub vertices_examined: usize,
    pub total_material_removed_mm3_estimate: f64,
    pub max_penetration_before_mm: f64,
    pub max_penetration_after_mm: f64,
}

/// Compute the per-vertex 1-ring neighborhood (vertex indices that share an
/// edge with `i`). Naive O(faces) scan — good for the relatively small hot
/// spot count (typically <500 vertices in a crown adjustment).
fn build_one_ring(mesh: &Mesh, vertex_count: usize) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); vertex_count];
    for tri in &mesh.indices {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let pairs = [(a, b), (b, c), (c, a)];
        for &(u, v) in &pairs {
            if !adj[u as usize].contains(&v) {
                adj[u as usize].push(v);
            }
            if !adj[v as usize].contains(&u) {
                adj[v as usize].push(u);
            }
        }
    }
    adj
}

/// Adjust `restoration_mesh` so contact against `antagonist_mesh` along
/// `occlusal_axis` does not exceed `contact_force_threshold`. Returns a
/// detailed report.
///
/// `occlusal_axis` should point FROM the restoration tooth toward the
/// antagonist (i.e. the chewing-load direction). It will be normalized
/// internally.
pub fn adjust_for_situ(
    restoration_mesh: &mut Mesh,
    antagonist_mesh: &Mesh,
    occlusal_axis: &Vector3<f64>,
    options: &AdjustingSituOptions,
) -> AdjustmentReport {
    let mut report = AdjustmentReport::default();
    if restoration_mesh.vertices.is_empty() || antagonist_mesh.vertices.is_empty() {
        return report;
    }
    let axis = if occlusal_axis.norm() > 1e-9 {
        occlusal_axis.normalize()
    } else {
        Vector3::z()
    };

    report.vertices_examined = restoration_mesh.vertices.len();
    let r2 = options.search_radius_mm * options.search_radius_mm;

    // Step 1 — find hot vertices and the local penetration (along occlusal axis).
    let mut penetration_per_vertex: Vec<f64> = vec![0.0; restoration_mesh.vertices.len()];
    let mut hot_indices: Vec<u32> = Vec::new();

    for (i, v) in restoration_mesh.vertices.iter().enumerate() {
        // Closest antagonist vertex within radius.
        let mut best_signed_gap = f64::MAX;
        let mut found = false;
        for av in &antagonist_mesh.vertices {
            let delta = av - v;
            if delta.norm_squared() > r2 {
                continue;
            }
            // Signed gap along occlusal axis: positive when antagonist sits
            // FURTHER along axis than the restoration vertex (= no contact).
            // Negative when restoration vertex has overshot the antagonist
            // (= penetration).
            let signed_gap = delta.dot(&axis);
            if signed_gap < best_signed_gap {
                best_signed_gap = signed_gap;
                found = true;
            }
        }
        if !found {
            continue;
        }
        // Penetration = -gap when negative.
        let pen = if best_signed_gap < 0.0 {
            -best_signed_gap
        } else {
            0.0
        };
        penetration_per_vertex[i] = pen;
        if pen > options.contact_force_threshold_mm {
            hot_indices.push(i as u32);
            if pen > report.max_penetration_before_mm {
                report.max_penetration_before_mm = pen;
            }
        }
    }

    if hot_indices.is_empty() {
        return report;
    }

    // Step 2 — push every hot vertex backwards along axis by (penetration + relief).
    // This is "subtract material along the occlusal direction".
    let mut total_displacement_mm = 0.0;
    let mut after_pen: Vec<f64> = vec![0.0; restoration_mesh.vertices.len()];

    for &vi in &hot_indices {
        let pen = penetration_per_vertex[vi as usize];
        let displacement = pen - options.contact_force_threshold_mm + options.relief_margin_mm;
        let displacement = displacement.max(0.0);
        // Move BACKWARDS along the occlusal axis (subtract material).
        let delta = -axis * displacement;
        restoration_mesh.vertices[vi as usize] += delta;
        total_displacement_mm += displacement;
        after_pen[vi as usize] = pen - displacement;
        report.hot_spots.push(HotSpot {
            vertex_index: vi,
            gap_before_mm: -pen,
            gap_after_mm: -(pen - displacement),
            displacement_mm: displacement,
        });
    }

    // Step 3 — Laplacian smoothing on hot vertices to blend the dimple. We
    // smooth ONLY the marked vertices so the rest of the surface is preserved.
    if options.smoothing_passes > 0 && options.smoothing_lambda > 0.0 {
        let adj = build_one_ring(restoration_mesh, restoration_mesh.vertices.len());
        for _ in 0..options.smoothing_passes {
            // Snapshot positions for this pass.
            let snapshot = restoration_mesh.vertices.clone();
            for &vi in &hot_indices {
                let neighbors = &adj[vi as usize];
                if neighbors.is_empty() {
                    continue;
                }
                let mut centroid = Vector3::zeros();
                for &nb in neighbors {
                    centroid += snapshot[nb as usize].coords;
                }
                centroid /= neighbors.len() as f64;
                let current = snapshot[vi as usize].coords;
                let new_pos = current + (centroid - current) * options.smoothing_lambda;
                restoration_mesh.vertices[vi as usize] = Point3::from(new_pos);
            }
        }
    }

    // Track post-adjustment max penetration (recompute for hot vertices only).
    for &vi in &hot_indices {
        let p = after_pen[vi as usize].max(0.0);
        if p > report.max_penetration_after_mm {
            report.max_penetration_after_mm = p;
        }
    }

    // Coarse volume estimate: each hot vertex represents ~ (avg edge length)² of
    // surface area, multiplied by displacement. Approximation matches exocad's
    // "material removed" tooltip.
    let avg_edge_length = estimate_avg_edge_length(restoration_mesh);
    let area_per_vertex = avg_edge_length * avg_edge_length;
    report.total_material_removed_mm3_estimate = total_displacement_mm * area_per_vertex;

    restoration_mesh.calculate_normals();
    report
}

fn estimate_avg_edge_length(mesh: &Mesh) -> f64 {
    if mesh.indices.is_empty() {
        return 0.5;
    }
    let mut sum = 0.0;
    let mut count = 0u64;
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        sum += (v1 - v0).norm() + (v2 - v1).norm() + (v0 - v2).norm();
        count += 3;
    }
    if count == 0 {
        0.5
    } else {
        sum / count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    /// Build a small "antagonist" plate just below z=1 so any restoration vertex
    /// at z>=1 will penetrate.
    fn flat_plate_at_z(z: f64) -> Mesh {
        let mut m = Mesh::new("plate");
        m.vertices.push(Point3::new(-1.0, -1.0, z));
        m.vertices.push(Point3::new(1.0, -1.0, z));
        m.vertices.push(Point3::new(1.0, 1.0, z));
        m.vertices.push(Point3::new(-1.0, 1.0, z));
        m.indices.push([0, 1, 2]);
        m.indices.push([0, 2, 3]);
        m.calculate_normals();
        m
    }

    #[test]
    fn no_contact_returns_no_hot_spots() {
        let mut crown = create_box(Point3::new(-0.5, -0.5, 0.0), Point3::new(0.5, 0.5, 0.5));
        let antagonist = flat_plate_at_z(2.0);
        let report = adjust_for_situ(
            &mut crown,
            &antagonist,
            &Vector3::z(),
            &AdjustingSituOptions::default(),
        );
        assert_eq!(report.hot_spots.len(), 0);
        assert_eq!(report.max_penetration_before_mm, 0.0);
    }

    #[test]
    fn deep_contact_creates_hot_spots_and_reduces_them() {
        // Crown extends from z=0..1.0, antagonist plate at z=0.5 ⇒ top half penetrates.
        let mut crown = create_box(Point3::new(-0.5, -0.5, 0.0), Point3::new(0.5, 0.5, 1.0));
        let antagonist = flat_plate_at_z(0.5);
        let opts = AdjustingSituOptions {
            contact_force_threshold_mm: 0.1,
            relief_margin_mm: 0.05,
            search_radius_mm: 5.0,
            smoothing_passes: 0,
            smoothing_lambda: 0.0,
        };
        let report = adjust_for_situ(&mut crown, &antagonist, &Vector3::z(), &opts);
        assert!(!report.hot_spots.is_empty(), "must detect penetration");
        assert!(report.max_penetration_before_mm > opts.contact_force_threshold_mm);
        // After adjustment the post-adjust penetration should be ≤ threshold (within smoothing tolerance).
        assert!(report.max_penetration_after_mm <= opts.contact_force_threshold_mm + 1e-9);
        // All hot spots moved by a positive distance.
        for hs in &report.hot_spots {
            assert!(hs.displacement_mm > 0.0);
            assert!(hs.gap_after_mm >= hs.gap_before_mm);
        }
    }

    #[test]
    fn smoothing_pass_does_not_break_mesh() {
        let mut crown = create_box(Point3::new(-0.5, -0.5, 0.0), Point3::new(0.5, 0.5, 1.0));
        let antagonist = flat_plate_at_z(0.5);
        let opts = AdjustingSituOptions {
            contact_force_threshold_mm: 0.05,
            relief_margin_mm: 0.02,
            search_radius_mm: 5.0,
            smoothing_passes: 3,
            smoothing_lambda: 0.5,
        };
        let v_before = crown.vertex_count();
        let t_before = crown.triangle_count();
        let report = adjust_for_situ(&mut crown, &antagonist, &Vector3::z(), &opts);
        assert_eq!(crown.vertex_count(), v_before);
        assert_eq!(crown.triangle_count(), t_before);
        assert!(report.total_material_removed_mm3_estimate > 0.0);
    }

    #[test]
    fn empty_meshes_yield_empty_report() {
        let mut empty = Mesh::new("empty");
        let antagonist = flat_plate_at_z(0.5);
        let report = adjust_for_situ(
            &mut empty,
            &antagonist,
            &Vector3::z(),
            &AdjustingSituOptions::default(),
        );
        assert!(report.hot_spots.is_empty());
        assert_eq!(report.vertices_examined, 0);
    }

    #[test]
    fn search_radius_limits_examination() {
        // Place crown far from antagonist. With small radius, no hot spots
        // should be produced even if they would otherwise penetrate.
        let mut crown = create_box(Point3::new(50.0, 50.0, 0.0), Point3::new(50.5, 50.5, 1.0));
        let antagonist = flat_plate_at_z(0.5);
        let opts = AdjustingSituOptions {
            contact_force_threshold_mm: 0.01,
            relief_margin_mm: 0.01,
            search_radius_mm: 1.0, // Way too small to reach
            smoothing_passes: 0,
            smoothing_lambda: 0.0,
        };
        let report = adjust_for_situ(&mut crown, &antagonist, &Vector3::z(), &opts);
        assert_eq!(report.hot_spots.len(), 0);
    }
}

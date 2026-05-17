//! Gingiva segmentation: derive the gingiva face region as the complement
//! of the union of tooth regions, then smooth its boundary.
//!
//! Port: `DentalProcessors/FreeformGingivaSegmentedProcessor` +
//! `FreeformGingivaProcessor`. AR-V401.
//!
//! Algorithm:
//!   * `segment_gingiva_from_teeth` — gingiva = ALL_FACES \ ⋃ tooth_regions.
//!     Builds a `FaceRegion` containing every face index not present in any
//!     of the input tooth regions. Deterministic, O(F + R log R).
//!   * `smooth_gingiva_boundary` — Laplacian relaxation of the vertices that
//!     lie on the gingiva region's boundary edge (vertices shared by an
//!     in-region face AND an out-region face). Each such vertex is moved
//!     toward the average of its in-region neighbours by `λ` per iteration
//!     (default `0.5`). Faces are not reassigned.

use nalgebra::Point3;
use std::collections::{HashMap, HashSet};
use tlanticad_mesh::region::FaceRegion;
use tlanticad_mesh::Mesh;

/// Build the gingiva face region: every face NOT in any tooth region.
pub fn segment_gingiva_from_teeth(arch_mesh: &Mesh, tooth_regions: &[FaceRegion]) -> FaceRegion {
    let n_faces = arch_mesh.indices.len();
    if n_faces == 0 {
        return FaceRegion::default();
    }
    let mut tooth_mask = vec![false; n_faces];
    for r in tooth_regions {
        for &fi in &r.faces {
            if fi < n_faces {
                tooth_mask[fi] = true;
            }
        }
    }
    let faces: Vec<usize> = (0..n_faces).filter(|&i| !tooth_mask[i]).collect();
    FaceRegion { faces }
}

/// Smooth the gingiva region's boundary in-place by Laplacian relaxation.
///
/// `iterations` rounds; `lambda` is the step size in `[0, 1]`.
/// Returns the number of vertices that were modified.
pub fn smooth_gingiva_boundary(
    mesh: &mut Mesh,
    region: &FaceRegion,
    iterations: u32,
) -> usize {
    smooth_gingiva_boundary_with_lambda(mesh, region, iterations, 0.5)
}

/// Same as [`smooth_gingiva_boundary`] but lets callers tune `lambda`.
pub fn smooth_gingiva_boundary_with_lambda(
    mesh: &mut Mesh,
    region: &FaceRegion,
    iterations: u32,
    lambda: f64,
) -> usize {
    if iterations == 0 || region.faces.is_empty() || mesh.vertices.is_empty() {
        return 0;
    }
    let lambda = lambda.clamp(0.0, 1.0);
    let n_faces = mesh.indices.len();
    let mut in_region = vec![false; n_faces];
    for &fi in &region.faces {
        if fi < n_faces {
            in_region[fi] = true;
        }
    }

    // For each vertex, list its in-region and out-region face count.
    // Boundary vertex: appears in BOTH at least once.
    let mut in_count = vec![0u32; mesh.vertices.len()];
    let mut out_count = vec![0u32; mesh.vertices.len()];
    for (fi, tri) in mesh.indices.iter().enumerate() {
        for &vi in tri {
            let vi = vi as usize;
            if in_region[fi] {
                in_count[vi] += 1;
            } else {
                out_count[vi] += 1;
            }
        }
    }
    let boundary_set: HashSet<usize> = (0..mesh.vertices.len())
        .filter(|&vi| in_count[vi] > 0 && out_count[vi] > 0)
        .collect();
    if boundary_set.is_empty() {
        return 0;
    }

    // Build per-vertex neighbour list (ALL faces — neighbours through any edge).
    let mut neighbours: HashMap<usize, HashSet<usize>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            let a = tri[i] as usize;
            let b = tri[(i + 1) % 3] as usize;
            neighbours.entry(a).or_default().insert(b);
            neighbours.entry(b).or_default().insert(a);
        }
    }

    for _ in 0..iterations {
        let snapshot = mesh.vertices.clone();
        for &vi in &boundary_set {
            if let Some(adj) = neighbours.get(&vi) {
                if adj.is_empty() {
                    continue;
                }
                let mut sum = nalgebra::Vector3::zeros();
                for &nv in adj {
                    sum += snapshot[nv].coords;
                }
                let mean = sum / adj.len() as f64;
                let curr = snapshot[vi].coords;
                let new = curr + (mean - curr) * lambda;
                mesh.vertices[vi] = Point3::from(new);
            }
        }
    }
    boundary_set.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    fn flat_strip(length: f64, n_quads: usize) -> Mesh {
        let mut m = Mesh::new("strip");
        for i in 0..=n_quads {
            let x = (i as f64) * length / n_quads as f64;
            m.vertices.push(Point3::new(x, 0.0, 0.0));
            m.vertices.push(Point3::new(x, 1.0, 0.0));
        }
        for i in 0..n_quads {
            let a = (2 * i) as u32;
            let b = (2 * i + 1) as u32;
            let c = (2 * i + 2) as u32;
            let d = (2 * i + 3) as u32;
            m.indices.push([a, b, c]);
            m.indices.push([b, d, c]);
        }
        m.calculate_normals();
        m
    }

    #[test]
    fn gingiva_is_empty_when_all_faces_are_teeth() {
        let m = flat_strip(10.0, 4);
        let teeth = vec![FaceRegion {
            faces: (0..m.indices.len()).collect(),
        }];
        let g = segment_gingiva_from_teeth(&m, &teeth);
        assert!(g.faces.is_empty());
    }

    #[test]
    fn gingiva_is_all_faces_when_no_teeth() {
        let m = flat_strip(10.0, 4);
        let g = segment_gingiva_from_teeth(&m, &[]);
        assert_eq!(g.faces.len(), m.indices.len());
    }

    #[test]
    fn gingiva_is_complement_of_tooth_regions() {
        let m = flat_strip(10.0, 8); // 16 triangles
        let teeth = vec![
            FaceRegion {
                faces: vec![0, 1, 2, 3],
            },
            FaceRegion {
                faces: vec![10, 11],
            },
        ];
        let g = segment_gingiva_from_teeth(&m, &teeth);
        // 16 faces - 6 tooth faces = 10 gingiva faces
        assert_eq!(g.faces.len(), 10);
        for &f in &g.faces {
            assert!(![0, 1, 2, 3, 10, 11].contains(&f));
        }
    }

    #[test]
    fn gingiva_dedupes_overlapping_tooth_regions() {
        let m = flat_strip(10.0, 4); // 8 triangles
        let teeth = vec![
            FaceRegion {
                faces: vec![0, 1, 2, 3],
            },
            FaceRegion {
                faces: vec![2, 3, 4],
            },
        ];
        let g = segment_gingiva_from_teeth(&m, &teeth);
        // 8 - 5 unique tooth faces = 3
        assert_eq!(g.faces.len(), 3);
    }

    #[test]
    fn smooth_returns_zero_when_no_iterations() {
        let mut m = flat_strip(10.0, 4);
        let region = FaceRegion {
            faces: vec![0, 1],
        };
        let n = smooth_gingiva_boundary(&mut m, &region, 0);
        assert_eq!(n, 0);
    }

    #[test]
    fn smooth_returns_zero_for_full_region_with_no_boundary() {
        let mut m = flat_strip(10.0, 4);
        // Entire mesh is in region → no out-faces → no boundary vertices.
        let region = FaceRegion {
            faces: (0..m.indices.len()).collect(),
        };
        let n = smooth_gingiva_boundary(&mut m, &region, 3);
        assert_eq!(n, 0);
    }

    #[test]
    fn smooth_moves_boundary_vertices() {
        let mut m = flat_strip(10.0, 8);
        let region = FaceRegion {
            faces: (0..8).collect(), // first half
        };
        let before = m.vertices.clone();
        let n = smooth_gingiva_boundary(&mut m, &region, 5);
        assert!(n > 0);
        let mut moved = 0;
        for (a, b) in before.iter().zip(m.vertices.iter()) {
            if (a - b).norm() > 1e-9 {
                moved += 1;
            }
        }
        assert!(moved > 0);
    }

    #[test]
    fn smooth_clamps_lambda_to_zero_means_no_motion() {
        let mut m = flat_strip(10.0, 8);
        let region = FaceRegion {
            faces: (0..8).collect(),
        };
        let before = m.vertices.clone();
        let _ = smooth_gingiva_boundary_with_lambda(&mut m, &region, 5, 0.0);
        for (a, b) in before.iter().zip(m.vertices.iter()) {
            assert!((a - b).norm() < 1e-12);
        }
    }
}

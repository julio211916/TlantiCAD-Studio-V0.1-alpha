//! Isotropic remeshing for uniform triangle quality
//!
//! Implements incremental remeshing: split long edges, collapse short edges,
//! flip edges to improve valence, tangential smoothing.

use nalgebra::{Point3, Vector3};
use std::collections::HashMap;

/// Remesh parameters
#[derive(Debug, Clone)]
pub struct RemeshParams {
    /// Target edge length
    pub target_length: f64,
    /// Number of iterations
    pub iterations: u32,
    /// Smoothing factor (0.0 to 1.0)
    pub smooth_factor: f64,
}

impl Default for RemeshParams {
    fn default() -> Self {
        Self {
            target_length: 1.0,
            iterations: 5,
            smooth_factor: 0.3,
        }
    }
}

/// Incremental isotropic remeshing
/// Returns (new_vertices, new_indices)
pub fn remesh(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
    params: &RemeshParams,
) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let mut verts: Vec<Point3<f64>> = vertices.to_vec();
    let mut tris: Vec<[u32; 3]> = indices.to_vec();

    let high = params.target_length * 4.0 / 3.0;
    let low = params.target_length * 4.0 / 5.0;

    for _ in 0..params.iterations {
        // Step 1: Split long edges
        tris = split_long_edges(&mut verts, &tris, high);

        // Step 2: Collapse short edges
        tris = collapse_short_edges(&mut verts, &tris, low);

        // Step 3: Edge flipping to improve valence
        tris = flip_edges_for_valence(&verts, &tris);

        // Step 4: Tangential smoothing
        tangential_smooth(&mut verts, &tris, params.smooth_factor);
    }

    (verts, tris)
}

fn split_long_edges(
    verts: &mut Vec<Point3<f64>>,
    tris: &[[u32; 3]],
    max_len: f64,
) -> Vec<[u32; 3]> {
    let max_len2 = max_len * max_len;
    let mut new_tris: Vec<[u32; 3]> = Vec::new();
    let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();

    for tri in tris {
        let edges = [
            (tri[0], tri[1], tri[2]),
            (tri[1], tri[2], tri[0]),
            (tri[2], tri[0], tri[1]),
        ];

        let mut splits = Vec::new();
        for &(a, b, c) in &edges {
            let len2 = (verts[a as usize] - verts[b as usize]).norm_squared();
            if len2 > max_len2 {
                splits.push((a, b, c));
            }
        }

        if splits.is_empty() {
            new_tris.push(*tri);
        } else {
            // Split the longest edge
            let (a, b, c) = splits.into_iter()
                .max_by(|&(a1, b1, _), &(a2, b2, _)| {
                    let l1 = (verts[a1 as usize] - verts[b1 as usize]).norm_squared();
                    let l2 = (verts[a2 as usize] - verts[b2 as usize]).norm_squared();
                    l1.partial_cmp(&l2).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();

            let key = (a.min(b), a.max(b));
            let mid = *midpoint_cache.entry(key).or_insert_with(|| {
                let mp = Point3::from((verts[a as usize].coords + verts[b as usize].coords) * 0.5);
                verts.push(mp);
                (verts.len() - 1) as u32
            });

            new_tris.push([a, mid, c]);
            new_tris.push([mid, b, c]);
        }
    }
    new_tris
}

fn collapse_short_edges(
    verts: &mut Vec<Point3<f64>>,
    tris: &[[u32; 3]],
    min_len: f64,
) -> Vec<[u32; 3]> {
    let min_len2 = min_len * min_len;
    let mut remap = vec![0u32; verts.len()];
    for (i, r) in remap.iter_mut().enumerate() { *r = i as u32; }

    // Find edges to collapse
    for tri in tris {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let ra = find_root(&remap, a);
            let rb = find_root(&remap, b);
            if ra == rb { continue; }

            let len2 = (verts[ra as usize] - verts[rb as usize]).norm_squared();
            if len2 < min_len2 {
                // Collapse b into a (move a to midpoint)
                let mid = Point3::from((verts[ra as usize].coords + verts[rb as usize].coords) * 0.5);
                verts[ra as usize] = mid;
                remap[rb as usize] = ra;
            }
        }
    }

    // Resolve remap chains
    for i in 0..remap.len() {
        remap[i] = find_root(&remap, i as u32);
    }

    // Filter degenerate triangles
    tris.iter()
        .map(|tri| [remap[tri[0] as usize], remap[tri[1] as usize], remap[tri[2] as usize]])
        .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[2] != tri[0])
        .collect()
}

fn find_root(remap: &[u32], mut v: u32) -> u32 {
    let mut steps = 0;
    while remap[v as usize] != v && steps < 1000 {
        v = remap[v as usize];
        steps += 1;
    }
    v
}

fn flip_edges_for_valence(
    verts: &[Point3<f64>],
    tris: &[[u32; 3]],
) -> Vec<[u32; 3]> {
    // Build adjacency: edge → two face indices
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = (a.min(b), a.max(b));
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Compute vertex valences
    let mut valence: HashMap<u32, i32> = HashMap::new();
    for tri in tris {
        for &v in tri {
            *valence.entry(v).or_insert(0) += 1;
        }
    }

    let mut result = tris.to_vec();

    // Try flipping each interior edge
    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 { continue; }

        let f0 = faces[0];
        let f1 = faces[1];

        // Find opposite vertices
        let c = result[f0].iter().find(|&&v| v != a && v != b).copied();
        let d = result[f1].iter().find(|&&v| v != a && v != b).copied();

        let (Some(c), Some(d)) = (c, d) else { continue };

        // Valence deviation before
        let target_valence = 6i32;
        let before: i32 = [a, b, c, d].iter()
            .map(|v| (valence[v] - target_valence).abs())
            .sum();

        // Valence deviation after flip
        let va_new = valence[&a] - 1;
        let vb_new = valence[&b] - 1;
        let vc_new = valence[&c] + 1;
        let vd_new = valence[&d] + 1;
        let after: i32 = [(va_new - target_valence).abs(),
                          (vb_new - target_valence).abs(),
                          (vc_new - target_valence).abs(),
                          (vd_new - target_valence).abs()].iter().sum();

        if after < before {
            // Check geometric validity (Delaunay criterion)
            let e1 = verts[c as usize] - verts[a as usize];
            let e2 = verts[d as usize] - verts[a as usize];
            if e1.cross(&e2).norm() < 1e-12 { continue; } // degenerate

            result[f0] = [a, c, d];
            result[f1] = [b, d, c];

            // Update valences
            *valence.get_mut(&a).unwrap() -= 1;
            *valence.get_mut(&b).unwrap() -= 1;
            *valence.get_mut(&c).unwrap() += 1;
            *valence.get_mut(&d).unwrap() += 1;
        }
    }
    result
}

fn tangential_smooth(
    verts: &mut Vec<Point3<f64>>,
    tris: &[[u32; 3]],
    factor: f64,
) {
    // Build adjacency
    let mut neighbors: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in tris {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            neighbors.entry(a).or_default().push(b);
            neighbors.entry(b).or_default().push(a);
        }
    }

    // Compute vertex normals (approx)
    let normals = compute_normals(verts, tris);

    let original = verts.clone();
    for (&vi, nbrs) in &neighbors {
        if nbrs.is_empty() { continue; }

        // Laplacian centroid
        let mut centroid = Vector3::zeros();
        for &ni in nbrs {
            centroid += original[ni as usize].coords;
        }
        centroid /= nbrs.len() as f64;
        let p = original[vi as usize];

        // Displacement
        let displacement = Point3::from(centroid) - p;

        // Project onto tangent plane
        let n = &normals[vi as usize];
        let tangential = displacement - n * displacement.dot(n);

        verts[vi as usize] = p + tangential * factor;
    }
}

fn compute_normals(verts: &[Point3<f64>], tris: &[[u32; 3]]) -> Vec<Vector3<f64>> {
    let mut normals = vec![Vector3::zeros(); verts.len()];
    for tri in tris {
        let a = &verts[tri[0] as usize];
        let b = &verts[tri[1] as usize];
        let c = &verts[tri[2] as usize];
        let n = (b - a).cross(&(c - a));
        for &vi in tri {
            normals[vi as usize] += n;
        }
    }
    for n in &mut normals {
        let len = n.norm();
        if len > 1e-15 { *n /= len; }
    }
    normals
}

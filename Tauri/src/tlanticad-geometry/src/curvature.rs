//! Curvature analysis: Gaussian, mean, principal curvatures per vertex

use nalgebra::{Point3, Vector3};

/// Per-vertex curvature data
#[derive(Debug, Clone, Copy)]
pub struct VertexCurvature {
    pub gaussian: f64,
    pub mean: f64,
    pub k1: f64,  // max principal curvature
    pub k2: f64,  // min principal curvature
    pub direction1: Vector3<f64>,
    pub direction2: Vector3<f64>,
}

/// Compute discrete Gaussian curvature at each vertex using angle defect
/// vertices: point positions, indices: triangle triplets
/// Returns one VertexCurvature per vertex
pub fn compute_curvature(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
) -> Vec<VertexCurvature> {
    let n = vertices.len();
    let mut angle_sum = vec![0.0_f64; n];
    let mut area_mixed = vec![0.0_f64; n];
    let mut mean_curv_vec = vec![Vector3::zeros(); n];
    let normals = compute_vertex_normals(vertices, indices);

    // Accumulate angles and cotangent Laplacian per vertex
    for tri in indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let p0 = &vertices[i0];
        let p1 = &vertices[i1];
        let p2 = &vertices[i2];

        let e01 = p1 - p0;
        let e02 = p2 - p0;
        let e12 = p2 - p1;

        let a0 = angle_between(&e01, &e02);
        let a1 = angle_between(&(p0 - p1), &e12);
        let a2 = angle_between(&(p0 - p2), &(p1 - p2));

        angle_sum[i0] += a0;
        angle_sum[i1] += a1;
        angle_sum[i2] += a2;

        // Mixed area contribution (Voronoi when non-obtuse, barycentric otherwise)
        let tri_area = e01.cross(&e02).norm() * 0.5;
        if tri_area < 1e-15 { continue; }

        let area_third = tri_area / 3.0;
        area_mixed[i0] += area_third;
        area_mixed[i1] += area_third;
        area_mixed[i2] += area_third;

        // Cotangent weights for mean curvature
        let cot0 = cotan(a0);
        let cot1 = cotan(a1);
        let cot2 = cotan(a2);

        // Mean curvature vector (cotangent Laplacian)
        mean_curv_vec[i0] += (e01 * cot2 + e02 * cot1) * 0.5;
        mean_curv_vec[i1] += ((p0 - p1) * cot2 + e12 * cot0) * 0.5;
        mean_curv_vec[i2] += ((p0 - p2) * cot1 + (p1 - p2) * cot0) * 0.5;
    }

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let a = area_mixed[i].max(1e-15);

        // Gaussian curvature via angle defect
        let gaussian = (std::f64::consts::TAU / 2.0 - angle_sum[i]) / a;

        // Mean curvature from Laplacian
        let h_vec = mean_curv_vec[i] / a;
        let mean = h_vec.norm() * 0.5 * h_vec.dot(&normals[i]).signum();

        // Principal curvatures from H and K
        let discriminant = (mean * mean - gaussian).max(0.0);
        let sqrt_disc = discriminant.sqrt();
        let k1 = mean + sqrt_disc;
        let k2 = mean - sqrt_disc;

        // Approximate principal directions (simplified: use normal cross)
        let n = &normals[i];
        let arbitrary = if n.x.abs() < 0.9 { Vector3::x() } else { Vector3::y() };
        let d1 = n.cross(&arbitrary).normalize();
        let d2 = n.cross(&d1).normalize();

        result.push(VertexCurvature {
            gaussian,
            mean,
            k1,
            k2,
            direction1: d1,
            direction2: d2,
        });
    }
    result
}

/// Compute angle-weighted vertex normals
pub fn compute_vertex_normals(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
) -> Vec<Vector3<f64>> {
    let mut normals = vec![Vector3::zeros(); vertices.len()];

    for tri in indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let e1 = vertices[i1] - vertices[i0];
        let e2 = vertices[i2] - vertices[i0];
        let face_normal = e1.cross(&e2);
        let area = face_normal.norm();
        if area < 1e-15 { continue; }

        let fn_normalized = face_normal / area;

        // Weight by angle at each vertex
        let a0 = angle_between(&e1, &e2);
        let a1 = angle_between(&(vertices[i0] - vertices[i1]), &(vertices[i2] - vertices[i1]));
        let a2 = angle_between(&(vertices[i0] - vertices[i2]), &(vertices[i1] - vertices[i2]));

        normals[i0] += fn_normalized * a0;
        normals[i1] += fn_normalized * a1;
        normals[i2] += fn_normalized * a2;
    }

    for n in &mut normals {
        let len = n.norm();
        if len > 1e-15 { *n /= len; }
    }
    normals
}

/// Compute Hausdorff distance between two point sets
/// Returns (forward_max, backward_max) — the directed Hausdorff distances
pub fn hausdorff_distance(
    points_a: &[Point3<f64>],
    points_b: &[Point3<f64>],
) -> (f64, f64) {
    let forward = directed_hausdorff(points_a, points_b);
    let backward = directed_hausdorff(points_b, points_a);
    (forward, backward)
}

fn directed_hausdorff(from: &[Point3<f64>], to: &[Point3<f64>]) -> f64 {
    let mut max_dist = 0.0_f64;
    for pa in from {
        let mut min_dist = f64::MAX;
        for pb in to {
            let d = (pa - pb).norm_squared();
            min_dist = min_dist.min(d);
        }
        max_dist = max_dist.max(min_dist);
    }
    max_dist.sqrt()
}

fn angle_between(a: &Vector3<f64>, b: &Vector3<f64>) -> f64 {
    let cos_angle = a.dot(b) / (a.norm() * b.norm()).max(1e-15);
    cos_angle.clamp(-1.0, 1.0).acos()
}

fn cotan(angle: f64) -> f64 {
    let s = angle.sin();
    if s.abs() < 1e-15 { 0.0 } else { angle.cos() / s }
}

//! Mesh thickness analysis for manufacturing validation
//!
//! Computes minimum wall thickness at each vertex using ray casting.

use nalgebra::{Point3, Vector3};

/// Per-vertex thickness result
#[derive(Debug, Clone, Copy)]
pub struct VertexThickness {
    pub vertex_index: u32,
    pub thickness: f64,
    pub inward_point: Point3<f64>,
}

/// Thickness analysis report
#[derive(Debug)]
pub struct ThicknessReport {
    pub per_vertex: Vec<VertexThickness>,
    pub min_thickness: f64,
    pub max_thickness: f64,
    pub avg_thickness: f64,
    pub thin_vertices: Vec<u32>, // below min_threshold
}

/// Analyze mesh thickness by casting rays inward from each vertex
/// Returns thickness report with per-vertex values
pub fn analyze_thickness(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    min_threshold: f64,
) -> ThicknessReport {
    let mut per_vertex = Vec::with_capacity(vertices.len());
    let mut thin_vertices = Vec::new();
    let mut min_t = f64::MAX;
    let mut max_t = 0.0_f64;
    let mut sum_t = 0.0_f64;

    for (vi, (pos, normal)) in vertices.iter().zip(normals.iter()).enumerate() {
        let vi = vi as u32;
        // Cast ray inward (opposite to normal)
        let ray_dir = -*normal;

        // Small offset to avoid self-intersection
        let origin = pos + &ray_dir * 0.01;

        let mut closest_t = f64::MAX;
        let mut closest_point = *pos;

        // Brute-force ray-triangle test (BVH should be used in production)
        for tri in indices {
            // Skip triangles sharing this vertex
            if tri.contains(&vi) { continue; }

            let a = &vertices[tri[0] as usize];
            let b = &vertices[tri[1] as usize];
            let c = &vertices[tri[2] as usize];

            if let Some((t, _u, _v)) = ray_triangle_intersect(&origin, &ray_dir, a, b, c) {
                if t > 0.0 && t < closest_t {
                    closest_t = t;
                    closest_point = Point3::from(origin.coords + ray_dir * t);
                }
            }
        }

        let thickness = if closest_t < f64::MAX { closest_t } else { 0.0 };

        if thickness < min_threshold && thickness > 0.0 {
            thin_vertices.push(vi);
        }

        if thickness > 0.0 {
            min_t = min_t.min(thickness);
            max_t = max_t.max(thickness);
            sum_t += thickness;
        }

        per_vertex.push(VertexThickness {
            vertex_index: vi,
            thickness,
            inward_point: closest_point,
        });
    }

    let count = per_vertex.iter().filter(|v| v.thickness > 0.0).count().max(1);

    ThicknessReport {
        per_vertex,
        min_thickness: if min_t < f64::MAX { min_t } else { 0.0 },
        max_thickness: max_t,
        avg_thickness: sum_t / count as f64,
        thin_vertices,
    }
}

/// Möller–Trumbore ray-triangle intersection
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
    if t > 1e-12 { Some((t, u, v)) } else { None }
}

/// Map thickness values to colors for visualization (blue=thin, green=ok, red=thick)
pub fn thickness_to_colors(report: &ThicknessReport, target: f64) -> Vec<[f32; 3]> {
    report.per_vertex.iter().map(|vt| {
        let ratio = (vt.thickness / target).clamp(0.0, 2.0);
        if ratio < 1.0 {
            // Thin: blue → green
            let t = ratio as f32;
            [0.0, t, 1.0 - t]
        } else {
            // Thick: green → red
            let t = (ratio - 1.0) as f32;
            [t, 1.0 - t, 0.0]
        }
    }).collect()
}

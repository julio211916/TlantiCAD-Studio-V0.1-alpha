//! Hole detection and filling algorithms for watertight mesh repair

use nalgebra::{Point3, Vector3};
use std::collections::HashMap;

/// A boundary loop (hole) described by an ordered ring of vertex indices
#[derive(Debug, Clone)]
pub struct BoundaryHole {
    pub vertices: Vec<u32>,
}

/// Detect all boundary holes in a triangle mesh
pub fn detect_holes(indices: &[[u32; 3]]) -> Vec<BoundaryHole> {
    // Build directed edge map: (from, to) → count
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            *edge_count.entry((a, b)).or_insert(0) += 1;
        }
    }

    // Boundary edges: those without a reverse twin
    let mut boundary_next: HashMap<u32, u32> = HashMap::new();
    for &(a, b) in edge_count.keys() {
        if !edge_count.contains_key(&(b, a)) {
            // Boundary edge goes b→a (reverse direction to form a loop)
            boundary_next.insert(b, a);
        }
    }

    // Walk boundary loops
    let mut visited = HashMap::new();
    let mut holes = Vec::new();

    for &start in boundary_next.keys() {
        if visited.contains_key(&start) { continue; }

        let mut loop_verts = Vec::new();
        let mut current = start;
        loop {
            if visited.contains_key(&current) { break; }
            visited.insert(current, true);
            loop_verts.push(current);
            match boundary_next.get(&current) {
                Some(&next) => current = next,
                None => break,
            }
        }
        if loop_verts.len() >= 3 {
            holes.push(BoundaryHole { vertices: loop_verts });
        }
    }
    holes
}

/// Fill a hole using ear-clipping triangulation
/// Returns new triangles to append to the mesh
pub fn fill_hole_ear_clip(
    vertices: &[Point3<f64>],
    hole: &BoundaryHole,
) -> Vec<[u32; 3]> {
    let n = hole.vertices.len();
    if n < 3 { return Vec::new(); }
    if n == 3 {
        return vec![[hole.vertices[0], hole.vertices[1], hole.vertices[2]]];
    }

    // Compute hole normal for orientation
    let normal = compute_hole_normal(vertices, &hole.vertices);

    let mut remaining: Vec<u32> = hole.vertices.clone();
    let mut triangles = Vec::new();

    while remaining.len() > 3 {
        let m = remaining.len();
        let mut ear_found = false;

        for i in 0..m {
            let prev = remaining[(i + m - 1) % m];
            let curr = remaining[i];
            let next = remaining[(i + 1) % m];

            let a = &vertices[prev as usize];
            let b = &vertices[curr as usize];
            let c = &vertices[next as usize];

            // Check if this is a convex (ear) vertex
            let edge1 = b - a;
            let edge2 = c - b;
            let cross = edge1.cross(&edge2);
            if cross.dot(&normal) <= 0.0 { continue; }

            // Check no other vertex is inside this triangle
            let is_ear = remaining.iter().enumerate().all(|(j, &v)| {
                if j == (i + m - 1) % m || j == i || j == (i + 1) % m { return true; }
                !point_in_triangle_3d(&vertices[v as usize], a, b, c, &normal)
            });

            if is_ear {
                triangles.push([prev, curr, next]);
                remaining.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // Fallback: fan fill from first vertex
            for i in 1..remaining.len() - 1 {
                triangles.push([remaining[0], remaining[i], remaining[i + 1]]);
            }
            break;
        }
    }

    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }

    triangles
}

/// Fill a hole using advancing-front method (better for larger holes)
/// Inserts new vertices at centroid for complex holes
pub fn fill_hole_advancing_front(
    vertices: &mut Vec<Point3<f64>>,
    hole: &BoundaryHole,
) -> Vec<[u32; 3]> {
    let n = hole.vertices.len();
    if n < 3 { return Vec::new(); }
    if n <= 6 {
        return fill_hole_ear_clip(vertices, hole);
    }

    // For larger holes: insert a centroid vertex and fan-fill
    let centroid = compute_centroid(vertices, &hole.vertices);
    let center_idx = vertices.len() as u32;
    vertices.push(centroid);

    let mut triangles = Vec::with_capacity(n);
    for i in 0..n {
        let a = hole.vertices[i];
        let b = hole.vertices[(i + 1) % n];
        triangles.push([a, b, center_idx]);
    }
    triangles
}

fn compute_hole_normal(vertices: &[Point3<f64>], hole_verts: &[u32]) -> Vector3<f64> {
    let mut normal: Vector3<f64> = Vector3::zeros();
    let n = hole_verts.len();
    for i in 0..n {
        let curr = &vertices[hole_verts[i] as usize];
        let next = &vertices[hole_verts[(i + 1) % n] as usize];
        // Newell's method for polygon normal
        normal.x += (curr.y - next.y) * (curr.z + next.z);
        normal.y += (curr.z - next.z) * (curr.x + next.x);
        normal.z += (curr.x - next.x) * (curr.y + next.y);
    }
    let len = normal.norm();
    if len > 1e-15 { normal / len } else { Vector3::z() }
}

fn compute_centroid(vertices: &[Point3<f64>], hole_verts: &[u32]) -> Point3<f64> {
    let mut sum = Vector3::zeros();
    for &vi in hole_verts {
        sum += vertices[vi as usize].coords;
    }
    Point3::from(sum / hole_verts.len() as f64)
}

fn point_in_triangle_3d(
    p: &Point3<f64>,
    a: &Point3<f64>,
    b: &Point3<f64>,
    c: &Point3<f64>,
    normal: &Vector3<f64>,
) -> bool {
    let cross0 = (b - a).cross(&(p - a));
    let cross1 = (c - b).cross(&(p - b));
    let cross2 = (a - c).cross(&(p - c));

    let d0 = cross0.dot(normal);
    let d1 = cross1.dot(normal);
    let d2 = cross2.dot(normal);

    (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
}

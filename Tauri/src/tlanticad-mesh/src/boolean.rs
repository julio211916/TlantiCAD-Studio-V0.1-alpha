//! Mesh Boolean operations: union, intersection, difference
//! Using constructive solid geometry (CSG) via BSP tree

use nalgebra::{Point3, Vector3};

/// Polygon for BSP operations
#[derive(Debug, Clone)]
struct CsgPolygon {
    vertices: Vec<Point3<f64>>,
    normal: Vector3<f64>,
}

/// BSP tree node
#[derive(Debug)]
enum BspNode {
    Leaf,
    Node {
        plane_normal: Vector3<f64>,
        plane_d: f64,
        front: Box<BspNode>,
        back: Box<BspNode>,
        _coplanar: Vec<CsgPolygon>,
    },
}

/// Boolean operation type
#[derive(Debug, Clone, Copy)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
}

/// Perform a Boolean operation between two triangle meshes
/// Returns (new_vertices, new_indices)
pub fn boolean_op(
    verts_a: &[Point3<f64>],
    tris_a: &[[u32; 3]],
    verts_b: &[Point3<f64>],
    tris_b: &[[u32; 3]],
    op: BooleanOp,
) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let polys_a = to_polygons(verts_a, tris_a);
    let polys_b = to_polygons(verts_b, tris_b);

    let bsp_a = build_bsp(&polys_a);
    let bsp_b = build_bsp(&polys_b);

    let result_polys = match op {
        BooleanOp::Union => {
            let a_outside_b = clip_to_bsp(&polys_a, &bsp_b, false);
            let b_outside_a = clip_to_bsp(&polys_b, &bsp_a, false);
            [a_outside_b, b_outside_a].concat()
        }
        BooleanOp::Intersection => {
            let a_inside_b = clip_to_bsp(&polys_a, &bsp_b, true);
            let b_inside_a = clip_to_bsp(&polys_b, &bsp_a, true);
            [a_inside_b, b_inside_a].concat()
        }
        BooleanOp::Difference => {
            let a_outside_b = clip_to_bsp(&polys_a, &bsp_b, false);
            let mut b_inside_a = clip_to_bsp(&polys_b, &bsp_a, true);
            // Flip normals of B's contribution
            for poly in &mut b_inside_a {
                poly.normal = -poly.normal;
                poly.vertices.reverse();
            }
            [a_outside_b, b_inside_a].concat()
        }
    };

    from_polygons(&result_polys)
}

fn to_polygons(verts: &[Point3<f64>], tris: &[[u32; 3]]) -> Vec<CsgPolygon> {
    tris.iter().map(|tri| {
        let a = verts[tri[0] as usize];
        let b = verts[tri[1] as usize];
        let c = verts[tri[2] as usize];
        let normal = (b - a).cross(&(c - a));
        let len = normal.norm();
        let normal = if len > 1e-15 { normal / len } else { Vector3::z() };
        CsgPolygon {
            vertices: vec![a, b, c],
            normal,
        }
    }).collect()
}

fn from_polygons(polys: &[CsgPolygon]) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let mut verts = Vec::new();
    let mut tris = Vec::new();

    for poly in polys {
        if poly.vertices.len() < 3 { continue; }
        let base = verts.len() as u32;

        for v in &poly.vertices {
            verts.push(*v);
        }

        // Fan triangulation for polygons
        for i in 1..poly.vertices.len() - 1 {
            tris.push([base, base + i as u32, base + i as u32 + 1]);
        }
    }
    (verts, tris)
}

fn build_bsp(polys: &[CsgPolygon]) -> BspNode {
    if polys.is_empty() { return BspNode::Leaf; }

    // Use first polygon as splitting plane
    let plane_normal = polys[0].normal;
    let plane_d = plane_normal.dot(&polys[0].vertices[0].coords);

    let mut front_polys = Vec::new();
    let mut back_polys = Vec::new();
    let mut coplanar_front = Vec::new();
    let mut coplanar_back = Vec::new();

    for poly in polys {
        split_polygon(poly, &plane_normal, plane_d, &mut coplanar_front, &mut coplanar_back, &mut front_polys, &mut back_polys);
    }

    let mut coplanar: Vec<CsgPolygon> = coplanar_front;
    coplanar.extend(coplanar_back);

    BspNode::Node {
        plane_normal,
        plane_d,
        front: Box::new(build_bsp(&front_polys)),
        back: Box::new(build_bsp(&back_polys)),
        _coplanar: coplanar,
    }
}

const EPSILON: f64 = 1e-8;

fn classify_point(normal: &Vector3<f64>, d: f64, point: &Point3<f64>) -> i32 {
    let dist = normal.dot(&point.coords) - d;
    if dist > EPSILON { 1 }      // front
    else if dist < -EPSILON { -1 } // back
    else { 0 }                     // coplanar
}

fn split_polygon(
    poly: &CsgPolygon,
    plane_n: &Vector3<f64>,
    plane_d: f64,
    coplanar_front: &mut Vec<CsgPolygon>,
    _coplanar_back: &mut Vec<CsgPolygon>,
    front: &mut Vec<CsgPolygon>,
    back: &mut Vec<CsgPolygon>,
) {
    let sides: Vec<i32> = poly.vertices.iter()
        .map(|v| classify_point(plane_n, plane_d, v))
        .collect();

    let all_front = sides.iter().all(|&s| s >= 0);
    let all_back = sides.iter().all(|&s| s <= 0);

    if all_front && !all_back {
        front.push(poly.clone());
    } else if all_back && !all_front {
        back.push(poly.clone());
    } else if all_front && all_back {
        // Coplanar — check alignment
        if poly.normal.dot(plane_n) > 0.0 {
            coplanar_front.push(poly.clone());
        } else {
            front.push(poly.clone()); // put coplanar on front by default
        }
    } else {
        // Split the polygon
        let mut front_verts = Vec::new();
        let mut back_verts = Vec::new();
        let n = poly.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let vi = &poly.vertices[i];
            let vj = &poly.vertices[j];
            let si = sides[i];
            let sj = sides[j];

            if si >= 0 { front_verts.push(*vi); }
            if si <= 0 { back_verts.push(*vi); }

            if (si > 0 && sj < 0) || (si < 0 && sj > 0) {
                // Compute intersection point
                let t_num = plane_d - plane_n.dot(&vi.coords);
                let t_den = plane_n.dot(&(vj - vi));
                if t_den.abs() > 1e-15 {
                    let t = (t_num / t_den).clamp(0.0, 1.0);
                    let intersection = Point3::from(vi.coords + (vj - vi) * t);
                    front_verts.push(intersection);
                    back_verts.push(intersection);
                }
            }
        }

        if front_verts.len() >= 3 {
            front.push(CsgPolygon { vertices: front_verts, normal: poly.normal });
        }
        if back_verts.len() >= 3 {
            back.push(CsgPolygon { vertices: back_verts, normal: poly.normal });
        }
    }
}

fn clip_to_bsp(polys: &[CsgPolygon], bsp: &BspNode, keep_inside: bool) -> Vec<CsgPolygon> {
    match bsp {
        BspNode::Leaf => {
            if keep_inside { Vec::new() } else { polys.to_vec() }
        }
        BspNode::Node { plane_normal, plane_d, front, back, .. } => {
            let mut front_polys = Vec::new();
            let mut back_polys = Vec::new();
            let mut co_front = Vec::new();
            let mut co_back = Vec::new();

            for poly in polys {
                split_polygon(poly, plane_normal, *plane_d, &mut co_front, &mut co_back, &mut front_polys, &mut back_polys);
            }

            let mut result_front = clip_to_bsp(&front_polys, front, keep_inside);
            let mut result_back = clip_to_bsp(&back_polys, back, keep_inside);

            // Coplanar goes with front or back depending on keep_inside
            if keep_inside {
                result_back.extend(co_front);
                result_back.extend(co_back);
            } else {
                result_front.extend(co_front);
                result_front.extend(co_back);
            }

            result_front.extend(result_back);
            result_front
        }
    }
}

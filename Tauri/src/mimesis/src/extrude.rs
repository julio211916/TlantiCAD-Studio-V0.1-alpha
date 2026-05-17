use crate::contour::Point;
use serde::{Deserialize, Serialize};

/// A 3D vertex.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A 2D texture coordinate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TexCoord {
    pub u: f64,
    pub v: f64,
}

/// A complete 3D mesh with vertices, normals, texture coords, and face indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub normals: Vec<Vertex3D>,
    pub tex_coords: Vec<TexCoord>,
    /// Face indices: each triplet of (vertex_idx, texcoord_idx, normal_idx)
    pub faces: Vec<[usize; 3]>,
    pub triangle_count: usize,
    pub vertex_count: usize,
}

/// Extrude a 2D polygon into a 3D mesh.
///
/// - `polygon`: 2D boundary points
/// - `triangles`: triangle indices from earcut (front face)
/// - `height`: extrusion depth along Z axis
/// - `img_width` / `img_height`: original image dimensions for UV mapping
pub fn extrude_polygon(
    polygon: &[Point],
    triangles: &[usize],
    height: f64,
    img_width: u32,
    img_height: u32,
) -> Mesh3D {
    let n = polygon.len();
    let w = img_width as f64;
    let h = img_height as f64;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();
    let mut faces = Vec::new();

    // ── Front face (z = 0) ──────────────────────────────────────────────
    let front_offset = 0;
    let front_normal_idx = normals.len();
    normals.push(Vertex3D {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    });

    for &(px, py) in polygon {
        vertices.push(Vertex3D {
            x: px,
            y: py,
            z: 0.0,
        });
        tex_coords.push(TexCoord {
            u: px / w,
            v: 1.0 - (py / h),
        });
    }

    // Front face triangles
    for tri in triangles.chunks(3) {
        if tri.len() == 3 {
            faces.push([front_offset + tri[0], front_offset + tri[0], front_normal_idx]);
            faces.push([front_offset + tri[1], front_offset + tri[1], front_normal_idx]);
            faces.push([front_offset + tri[2], front_offset + tri[2], front_normal_idx]);
        }
    }

    // ── Back face (z = height) ──────────────────────────────────────────
    let back_offset = vertices.len();
    let back_normal_idx = normals.len();
    normals.push(Vertex3D {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    });

    for &(px, py) in polygon {
        vertices.push(Vertex3D {
            x: px,
            y: py,
            z: height,
        });
        tex_coords.push(TexCoord {
            u: px / w,
            v: 1.0 - (py / h),
        });
    }

    // Back face triangles (reversed winding)
    for tri in triangles.chunks(3) {
        if tri.len() == 3 {
            faces.push([back_offset + tri[2], back_offset + tri[2], back_normal_idx]);
            faces.push([back_offset + tri[1], back_offset + tri[1], back_normal_idx]);
            faces.push([back_offset + tri[0], back_offset + tri[0], back_normal_idx]);
        }
    }

    // ── Side faces ──────────────────────────────────────────────────────
    let _side_start = vertices.len();
    let side_tc_start = tex_coords.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let (ax, ay) = polygon[i];
        let (bx, by) = polygon[j];

        // Edge normal (outward facing)
        let edge_dx = bx - ax;
        let edge_dy = by - ay;
        let edge_len = (edge_dx * edge_dx + edge_dy * edge_dy).sqrt().max(1e-10);
        let nx = edge_dy / edge_len;
        let ny = -edge_dx / edge_len;

        let normal_idx = normals.len();
        normals.push(Vertex3D {
            x: nx,
            y: ny,
            z: 0.0,
        });

        let vi = vertices.len();

        // 4 vertices per side quad
        vertices.push(Vertex3D { x: ax, y: ay, z: 0.0 });
        vertices.push(Vertex3D { x: bx, y: by, z: 0.0 });
        vertices.push(Vertex3D {
            x: bx,
            y: by,
            z: height,
        });
        vertices.push(Vertex3D {
            x: ax,
            y: ay,
            z: height,
        });

        // Simple UVs for sides
        let u0 = i as f64 / n as f64;
        let u1 = (i + 1) as f64 / n as f64;
        tex_coords.push(TexCoord { u: u0, v: 0.0 });
        tex_coords.push(TexCoord { u: u1, v: 0.0 });
        tex_coords.push(TexCoord { u: u1, v: 1.0 });
        tex_coords.push(TexCoord { u: u0, v: 1.0 });

        let ti = side_tc_start + i * 4;

        // Two triangles per quad
        faces.push([vi, ti, normal_idx]);
        faces.push([vi + 1, ti + 1, normal_idx]);
        faces.push([vi + 2, ti + 2, normal_idx]);

        faces.push([vi, ti, normal_idx]);
        faces.push([vi + 2, ti + 2, normal_idx]);
        faces.push([vi + 3, ti + 3, normal_idx]);
    }

    let triangle_count = faces.len() / 3;
    let vertex_count = vertices.len();

    Mesh3D {
        vertices,
        normals,
        tex_coords,
        faces,
        triangle_count,
        vertex_count,
    }
}

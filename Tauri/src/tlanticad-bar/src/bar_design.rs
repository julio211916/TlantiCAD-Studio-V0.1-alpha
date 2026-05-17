//! Bar framework design for implant-retained overdentures

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::attachment::BarAttachment;

/// Bar framework material
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarMaterial {
    Titanium,
    CobaltChrome,
    PEEK,
    Zirconia,
}

/// Bar cross-section profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarType {
    Dolder,
    Hader,
    Milled,
    Round,
    Oval,
}

/// A segment of bar between two implant abutments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarSegment {
    pub start_implant: Uuid,
    pub end_implant: Uuid,
    pub bar_type: BarType,
    pub material: BarMaterial,
    pub length_mm: f64,
}

/// Complete bar framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarFramework {
    pub segments: Vec<BarSegment>,
    pub attachments: Vec<BarAttachment>,
    pub retentive_clips: u32,
}

impl BarFramework {
    /// Create a new empty framework
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            attachments: Vec::new(),
            retentive_clips: 0,
        }
    }
}

impl Default for BarFramework {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a smooth bar path through implant positions using Catmull-Rom interpolation.
///
/// Returns a polyline with approximately 10 points per segment.
pub fn design_bar_path(implant_positions: &[Point3<f64>]) -> Vec<Point3<f64>> {
    if implant_positions.len() < 2 {
        return implant_positions.to_vec();
    }

    let mut path = Vec::new();
    let steps = 10usize;

    for i in 0..implant_positions.len().saturating_sub(1) {
        let p0 = implant_positions.get(i.saturating_sub(1)).unwrap_or(&implant_positions[i]);
        let p1 = &implant_positions[i];
        let p2 = &implant_positions[(i + 1).min(implant_positions.len() - 1)];
        let p3 = &implant_positions[(i + 2).min(implant_positions.len() - 1)];

        for s in 0..steps {
            let t = s as f64 / steps as f64;
            let pt = catmull_rom(p0, p1, p2, p3, t);
            path.push(pt);
        }
    }
    path.push(*implant_positions.last().unwrap());
    path
}

/// Calculate the total arc length of a bar path
pub fn calculate_bar_length(positions: &[Point3<f64>]) -> f64 {
    positions.windows(2).map(|w| (w[1] - w[0]).norm()).sum()
}

fn catmull_rom(
    p0: &Point3<f64>,
    p1: &Point3<f64>,
    p2: &Point3<f64>,
    p3: &Point3<f64>,
    t: f64,
) -> Point3<f64> {
    let t2 = t * t;
    let t3 = t2 * t;
    let f = |v0: f64, v1: f64, v2: f64, v3: f64| {
        0.5 * ((-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3
            + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
            + (-v0 + v2) * t
            + 2.0 * v1)
    };
    Point3::new(
        f(p0.x, p1.x, p2.x, p3.x),
        f(p0.y, p1.y, p2.y, p3.y),
        f(p0.z, p1.z, p2.z, p3.z),
    )
}

/// Generate a cylindrical tube mesh along a path of points.
/// `radius_mm`: tube radius. `segments`: number of sides per ring (8=octagon, 12=approx circle).
pub fn generate_tube_mesh(path: &[nalgebra::Point3<f64>], radius_mm: f64, segments: u32) -> tlanticad_mesh::Mesh {
    if path.len() < 2 || segments < 3 { return tlanticad_mesh::Mesh::new("tube"); }

    let segs = segments as usize;
    let mut verts: Vec<nalgebra::Point3<f64>> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    for (i, &center) in path.iter().enumerate() {
        let tangent = if i == 0 {
            (path[1] - path[0]).normalize()
        } else if i == path.len() - 1 {
            (path[i] - path[i - 1]).normalize()
        } else {
            ((path[i + 1] - path[i - 1]) * 0.5).normalize()
        };

        let up = if tangent.x.abs() < 0.9 {
            nalgebra::Vector3::new(1.0, 0.0, 0.0)
        } else {
            nalgebra::Vector3::new(0.0, 1.0, 0.0)
        };
        let right = tangent.cross(&up).normalize();
        let up2 = right.cross(&tangent).normalize();

        for s in 0..segs {
            let angle = 2.0 * std::f64::consts::PI * s as f64 / segs as f64;
            let offset = right * (angle.cos() * radius_mm) + up2 * (angle.sin() * radius_mm);
            verts.push(nalgebra::Point3::new(
                center.x + offset.x,
                center.y + offset.y,
                center.z + offset.z,
            ));
        }
    }

    let n_rings = path.len();
    for ring in 0..(n_rings - 1) {
        for s in 0..segs {
            let a = (ring * segs + s) as u32;
            let b = (ring * segs + (s + 1) % segs) as u32;
            let c = ((ring + 1) * segs + (s + 1) % segs) as u32;
            let d = ((ring + 1) * segs + s) as u32;
            indices.push([a, b, d]);
            indices.push([b, c, d]);
        }
    }

    // Start cap
    let cap_start = verts.len() as u32;
    verts.push(path[0]);
    for s in 0..segs {
        let a = s as u32;
        let b = ((s + 1) % segs) as u32;
        indices.push([cap_start, b, a]);
    }

    // End cap
    let cap_end = verts.len() as u32;
    verts.push(*path.last().unwrap());
    let last_ring = ((n_rings - 1) * segs) as u32;
    for s in 0..segs {
        let a = last_ring + s as u32;
        let b = last_ring + ((s + 1) % segs) as u32;
        indices.push([cap_end, a, b]);
    }

    let mut mesh = tlanticad_mesh::Mesh::new("tube");
    mesh.vertices = verts;
    mesh.indices = indices;
    mesh
}

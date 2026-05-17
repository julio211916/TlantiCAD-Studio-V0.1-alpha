//! Individual tooth die management

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// An individual removable tooth die
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothDie {
    pub fdi_number: u8,
    pub mesh: Option<Mesh>,
    pub margin_line: Vec<Point3<f64>>,
    pub insertion_axis: Vector3<f64>,
}

impl ToothDie {
    /// Create a new tooth die for the given FDI number
    pub fn new(fdi_number: u8) -> Self {
        Self {
            fdi_number,
            mesh: None,
            margin_line: Vec::new(),
            insertion_axis: Vector3::new(0.0, 0.0, 1.0),
        }
    }
}

/// Calculate the best insertion axis for a die by averaging vertex normals
/// in the gingival third of the die mesh.
pub fn calculate_insertion_axis(die: &ToothDie) -> Vector3<f64> {
    let mesh = match &die.mesh {
        Some(m) if !m.vertices.is_empty() => m,
        _ => return Vector3::new(0.0, 0.0, 1.0),
    };

    // Find bounding box and select lower-third vertices
    let (min, max) = mesh.calculate_bounds();
    let lower_threshold = min.z + (max.z - min.z) * 0.33;

    let mut avg_normal = Vector3::zeros();
    let mut count = 0usize;

    for (i, v) in mesh.vertices.iter().enumerate() {
        if v.z < lower_threshold && i < mesh.normals.len() {
            avg_normal += mesh.normals[i];
            count += 1;
        }
    }

    if count > 0 {
        let n = avg_normal / count as f64;
        if n.norm() > 1e-6 {
            return n.normalize();
        }
    }

    Vector3::new(0.0, 0.0, 1.0)
}

/// Find undercut areas on a die: vertices whose normals oppose the insertion axis.
///
/// Returns the positions of vertices that would lock the die during removal.
pub fn check_undercuts(die: &ToothDie) -> Vec<Point3<f64>> {
    let mesh = match &die.mesh {
        Some(m) => m,
        None => return Vec::new(),
    };

    let axis = die.insertion_axis.normalize();
    let mut undercut_points = Vec::new();

    for (i, v) in mesh.vertices.iter().enumerate() {
        if i < mesh.normals.len() {
            let n = mesh.normals[i];
            if n.dot(&axis) < -0.15 {
                undercut_points.push(*v);
            }
        }
    }

    undercut_points
}

/// Automatically segment a full arch scan into individual tooth dies
/// by finding inter-proximal valleys (low-z regions between teeth).
///
/// Algorithm:
/// 1. Compute height map projected onto X axis (32 bins)
/// 2. Find valley bins (local z minima below 40% height threshold)
/// 3. Cut arch along valley planes into X slices
/// 4. Split each slice by connected component → individual dies
pub fn segment_arch_into_dies(arch: &Mesh) -> Vec<ToothDie> {
    use tlanticad_mesh::{connected_components, extract_submesh};

    let z_min = arch.vertices.iter().map(|v| v.z).fold(f64::INFINITY, f64::min);
    let z_max = arch.vertices.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
    let z_range = z_max - z_min;
    if z_range < 0.1 { return Vec::new(); }

    let x_min = arch.vertices.iter().map(|v| v.x).fold(f64::INFINITY, f64::min);
    let x_max = arch.vertices.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max);
    let x_range = x_max - x_min;
    if x_range < 1.0 { return Vec::new(); }

    let bins = 32usize;
    let mut max_z_per_bin = vec![z_min; bins];
    for v in &arch.vertices {
        let bin = ((v.x - x_min) / x_range * (bins - 1) as f64) as usize;
        let bin = bin.min(bins - 1);
        if v.z > max_z_per_bin[bin] {
            max_z_per_bin[bin] = v.z;
        }
    }

    let threshold = z_min + z_range * 0.4;
    let mut valley_x: Vec<f64> = Vec::new();
    for i in 1..(bins - 1) {
        if max_z_per_bin[i] < threshold
            && max_z_per_bin[i] < max_z_per_bin[i - 1]
            && max_z_per_bin[i] < max_z_per_bin[i + 1]
        {
            valley_x.push(x_min + (i as f64 / (bins - 1) as f64) * x_range);
        }
    }

    if valley_x.is_empty() {
        return vec![ToothDie {
            fdi_number: 0,
            mesh: Some(arch.clone()),
            margin_line: Vec::new(),
            insertion_axis: Vector3::new(0.0, 0.0, 1.0),
        }];
    }

    let mut boundaries = vec![x_min];
    boundaries.extend_from_slice(&valley_x);
    boundaries.push(x_max + 0.001);

    let mut dies: Vec<ToothDie> = Vec::new();

    for seg in 0..(boundaries.len() - 1) {
        let x_lo = boundaries[seg];
        let x_hi = boundaries[seg + 1];

        let face_indices: Vec<usize> = (0..arch.indices.len()).filter(|&fi| {
            let tri = arch.indices[fi];
            let a = tri[0] as usize;
            let b = tri[1] as usize;
            let c = tri[2] as usize;
            if a >= arch.vertices.len() || b >= arch.vertices.len() || c >= arch.vertices.len() { return false; }
            let cx = (arch.vertices[a].x + arch.vertices[b].x + arch.vertices[c].x) / 3.0;
            cx >= x_lo && cx < x_hi
        }).collect();

        if face_indices.is_empty() { continue; }

        let sub = extract_submesh(arch, &face_indices);
        let components = connected_components(&sub);

        for comp in &components {
            if comp.len() < 4 { continue; }
            let die_mesh = extract_submesh(&sub, comp);
            dies.push(ToothDie {
                fdi_number: 0,
                insertion_axis: calculate_insertion_axis_from_mesh(&die_mesh),
                mesh: Some(die_mesh),
                margin_line: Vec::new(),
            });
        }
    }

    dies
}

fn calculate_insertion_axis_from_mesh(mesh: &Mesh) -> Vector3<f64> {
    if mesh.vertices.is_empty() { return Vector3::new(0.0, 0.0, 1.0); }
    Vector3::new(0.0, 0.0, 1.0)
}

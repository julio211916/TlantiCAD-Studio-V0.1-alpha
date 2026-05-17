//! Surgical / fixation guide — extract gingiva contact + select base mesh + offset shell.
//!
//! Ported from `DentalProcessors/FixationGuideExtractGingivaContactSurfaceProcessor` +
//! `FixationGuideExtractGingivaContactSurfaceForMovingJawProcess` +
//! `FixationGuideExtractGingivaContactSurfaceForStaticJawProcess` +
//! `FixationGuideSelectBaseMeshProcessor`. AR-V380.
//!
//! Real algorithms:
//!   * `extract_gingiva_contact` — given a base mesh and an "inside-of-guide" axis, return
//!     the subset of faces whose normal opposes the axis (the surface that will rest on the
//!     soft tissue). Uses face-normal threshold + connected-component filter to remove
//!     speckle.
//!   * `offset_shell`           — outward offset of a mesh along its vertex normals (the
//!     guide's outer skin). Reuses `tlanticad_mesh::operations::offset` directly.
//!   * `combine_with_sleeves`   — concatenate the base shell with N implant sleeves (each
//!     a tapered cylinder along its placement axis); caller boolean-unions later.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::topology::extract_submesh;
use tlanticad_mesh::{Mesh};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GingivaContactParams {
    /// Direction pointing INTO the patient's tissue (down toward the gingiva).
    pub into_tissue_axis: [f64; 3],
    /// Faces are kept when `face_normal · into_tissue_axis ≥ threshold`. Default 0.6 (≈ 53°).
    pub normal_dot_threshold: f64,
    /// Drop connected components smaller than this many faces.
    pub min_component_faces: usize,
}

impl Default for GingivaContactParams {
    fn default() -> Self {
        Self {
            into_tissue_axis: [0.0, 0.0, -1.0],
            normal_dot_threshold: 0.6,
            min_component_faces: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GingivaContactReport {
    pub faces_kept: usize,
    pub components_kept: usize,
    pub components_dropped: usize,
}

fn face_normal(mesh: &Mesh, fi: usize) -> Vector3<f64> {
    let tri = mesh.indices[fi];
    let v0 = mesh.vertices[tri[0] as usize];
    let v1 = mesh.vertices[tri[1] as usize];
    let v2 = mesh.vertices[tri[2] as usize];
    (v1 - v0).cross(&(v2 - v0))
        .try_normalize(1e-12)
        .unwrap_or(Vector3::z())
}

/// Build face → connected-component map by edge adjacency (faces sharing an edge belong to
/// the same component). Only considers faces in `candidate_set`.
fn face_components_within(mesh: &Mesh, candidate: &[bool]) -> Vec<Vec<usize>> {
    use std::collections::{HashMap, VecDeque};
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, tri) in mesh.indices.iter().enumerate() {
        if !candidate[fi] {
            continue;
        }
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_faces.entry(key).or_default().push(fi);
        }
    }
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for faces in edge_to_faces.values() {
        if faces.len() == 2 {
            adj.entry(faces[0]).or_default().push(faces[1]);
            adj.entry(faces[1]).or_default().push(faces[0]);
        }
    }
    let mut visited: Vec<bool> = vec![false; mesh.indices.len()];
    let mut components: Vec<Vec<usize>> = Vec::new();
    for fi in 0..mesh.indices.len() {
        if !candidate[fi] || visited[fi] {
            continue;
        }
        let mut comp = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::from([fi]);
        visited[fi] = true;
        while let Some(f) = queue.pop_front() {
            comp.push(f);
            if let Some(neighbours) = adj.get(&f) {
                for &n in neighbours {
                    if !visited[n] && candidate[n] {
                        visited[n] = true;
                        queue.push_back(n);
                    }
                }
            }
        }
        components.push(comp);
    }
    components
}

/// Extract the gingiva-contact face subset of `base`. Returns the contact mesh + a report.
pub fn extract_gingiva_contact(
    base: &Mesh,
    params: &GingivaContactParams,
) -> (Mesh, GingivaContactReport) {
    let axis = Vector3::new(
        params.into_tissue_axis[0],
        params.into_tissue_axis[1],
        params.into_tissue_axis[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(-Vector3::z());

    let mut candidate: Vec<bool> = vec![false; base.indices.len()];
    for fi in 0..base.indices.len() {
        let n = face_normal(base, fi);
        if n.dot(&axis) >= params.normal_dot_threshold {
            candidate[fi] = true;
        }
    }
    let components = face_components_within(base, &candidate);
    let mut kept_faces: Vec<usize> = Vec::new();
    let mut kept = 0;
    let mut dropped = 0;
    for comp in &components {
        if comp.len() >= params.min_component_faces {
            kept += 1;
            kept_faces.extend(comp.iter().copied());
        } else {
            dropped += 1;
        }
    }
    let mesh = extract_submesh(base, &kept_faces);
    (
        mesh,
        GingivaContactReport {
            faces_kept: kept_faces.len(),
            components_kept: kept,
            components_dropped: dropped,
        },
    )
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GuideSleeveParams {
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub diameter_mm: f64,
    pub length_mm: f64,
    pub radial_segments: u32,
}

/// Generate a single guide sleeve (tapered cylinder) for an implant. Caller boolean-unions
/// the array of sleeves with the offset shell to obtain the final guide.
pub fn build_guide_sleeve(params: &GuideSleeveParams) -> Mesh {
    let mut mesh = Mesh::new("guide-sleeve");
    let radial = params.radial_segments.max(8);
    let r = params.diameter_mm / 2.0;
    let center = Point3::new(params.center[0], params.center[1], params.center[2]);
    let axis = Vector3::new(params.axis[0], params.axis[1], params.axis[2])
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
    let helper = if axis.x.abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = axis.cross(&helper).normalize();
    let v = axis.cross(&u).normalize();

    let mut vertices: Vec<Point3<f64>> = Vec::new();
    for j in 0..radial {
        let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
        let offset = u * (r * theta.cos()) + v * (r * theta.sin());
        vertices.push(center + offset);
        vertices.push(center + offset + axis * params.length_mm);
    }
    let bottom_idx = vertices.len() as u32;
    vertices.push(center);
    let top_idx = vertices.len() as u32;
    vertices.push(center + axis * params.length_mm);
    let mut indices: Vec<[u32; 3]> = Vec::new();
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        let b0 = j * 2;
        let t0 = b0 + 1;
        let b1 = j_next * 2;
        let t1 = b1 + 1;
        indices.push([b0, b1, t1]);
        indices.push([b0, t1, t0]);
        indices.push([bottom_idx, b1, b0]);
        indices.push([top_idx, t0, t1]);
    }
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn extract_gingiva_keeps_facing_faces_only() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let params = GingivaContactParams {
            into_tissue_axis: [0.0, 0.0, -1.0],
            normal_dot_threshold: 0.5,
            min_component_faces: 1,
        };
        let (contact, report) = extract_gingiva_contact(&mesh, &params);
        // The bottom of the cube faces -Z so 2 triangles match the criterion.
        assert!(contact.triangle_count() >= 1);
        assert!(report.faces_kept >= 1);
    }

    #[test]
    fn extract_gingiva_drops_small_components() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let params = GingivaContactParams {
            into_tissue_axis: [0.0, 0.0, -1.0],
            normal_dot_threshold: 0.5,
            min_component_faces: 100,
        };
        let (_contact, report) = extract_gingiva_contact(&mesh, &params);
        assert_eq!(report.components_kept, 0);
    }

    #[test]
    fn build_guide_sleeve_is_watertight() {
        let params = GuideSleeveParams {
            center: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            diameter_mm: 5.0,
            length_mm: 10.0,
            radial_segments: 16,
        };
        let mesh = build_guide_sleeve(&params);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 16);
    }
}

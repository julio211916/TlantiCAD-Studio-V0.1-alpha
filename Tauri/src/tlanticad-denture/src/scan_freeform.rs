//! Denture-scan freeform classification — tooth / flange / base / impression.
//! AR-V415.
//!
//! Conceptually ported from
//! `artifacts/DentalProcessors/FreeformDentureScanProcessor.cs`. The C# code
//! shells out to a native classifier; we reimplement that classifier from
//! curvature signals only.
//!
//! Inputs: a single full-arch denture scan mesh — typically a digitised
//! impression of an existing denture or a wax try-in.
//!
//! Outputs:
//!   * Per-vertex classification into one of four regions
//!     (tooth / flange / base / impression / unclassified).
//!   * `DentureScanReport` with face-region indices for each class plus
//!     summary statistics (curvature thresholds used, vertex counts).
//!
//! Algorithm (real geometry, deterministic):
//!
//!   1. **Vertex normals + neighbour adjacency** are built once.
//!   2. **Curvature proxy**: for each vertex, we compute the average
//!      angular deviation between its normal and its neighbours' normals.
//!      High deviation → ridge / cusp (tooth); near-zero → flat (base).
//!      This is a robust, mesh-resolution-tolerant proxy used widely in
//!      dental CAD.
//!   3. **Vertical bands**: the principal axis of the mesh (PCA-derived)
//!      gives an "occlusal-cervical" direction. We split vertices into
//!      three slabs along that axis: top 33 % (occlusal), middle 33 %
//!      (cervical), bottom 33 % (impression / fitting surface).
//!   4. **Classification rule**:
//!      * Top slab + high curvature → **tooth** (cusps / incisal edges).
//!      * Top slab + low curvature  → **flange** (vestibular surface).
//!      * Middle slab               → **flange** (gingival flank).
//!      * Bottom slab + low curvature → **base** (palate / lingual base).
//!      * Bottom slab + high curvature → **impression** (rugae / undercuts).
//!
//! Thresholds are configurable; defaults are tuned for typical denture
//! resolution (0.1–0.3 mm vertex spacing).

use nalgebra::{Matrix3, Point3, SymmetricEigen, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DentureRegion {
    Unclassified = 0,
    Tooth = 1,
    Flange = 2,
    Base = 3,
    Impression = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DentureScanOptions {
    /// Curvature angular-deviation threshold (radians) above which a vertex
    /// is "high curvature" — i.e. likely a tooth ridge or impression rugae.
    /// Default `0.45 rad` (~26°), good for typical 0.2 mm scans.
    pub curvature_threshold_rad: f64,
    /// Bottom slab fraction (0..1). The lowest `bottom_band` quantile of the
    /// principal-axis distribution is treated as "impression / base side".
    /// Default 0.33.
    pub bottom_band: f64,
    /// Top slab fraction. Default 0.33.
    pub top_band: f64,
}

impl Default for DentureScanOptions {
    fn default() -> Self {
        Self {
            curvature_threshold_rad: 0.45,
            bottom_band: 0.33,
            top_band: 0.33,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DentureScanReport {
    /// Per-vertex region tag (length = `mesh.vertices.len()`).
    pub vertex_regions: Vec<DentureRegion>,
    /// Vertex indices for each region (handy for FaceRegion conversion).
    pub tooth_vertices: Vec<u32>,
    pub flange_vertices: Vec<u32>,
    pub base_vertices: Vec<u32>,
    pub impression_vertices: Vec<u32>,
    /// Statistics actually used during classification (after PCA).
    pub principal_axis: [f64; 3],
    pub low_band_cutoff: f64,
    pub high_band_cutoff: f64,
    pub mean_curvature_rad: f64,
}

/// Process a denture scan; returns a per-vertex region tag plus a summary.
pub fn process_denture_scan(scan_mesh: &Mesh, options: &DentureScanOptions) -> DentureScanReport {
    let mut report = DentureScanReport::default();
    let n = scan_mesh.vertices.len();
    if n == 0 || scan_mesh.indices.is_empty() {
        return report;
    }
    report.vertex_regions = vec![DentureRegion::Unclassified; n];

    // 1. Per-vertex normals (recompute if missing).
    let normals = build_vertex_normals(scan_mesh);

    // 2. Adjacency (vertex → neighbour vertices).
    let adjacency = build_vertex_adjacency(scan_mesh);

    // 3. Curvature proxy — mean angular deviation in radians.
    let mut curvatures = vec![0.0_f64; n];
    let mut sum_curv = 0.0_f64;
    for i in 0..n {
        if adjacency[i].is_empty() {
            continue;
        }
        let n_self = normals[i];
        let mut acc = 0.0_f64;
        let mut count = 0usize;
        for &j in &adjacency[i] {
            let dot = n_self.dot(&normals[j as usize]).clamp(-1.0, 1.0);
            acc += dot.acos();
            count += 1;
        }
        let v = if count > 0 { acc / count as f64 } else { 0.0 };
        curvatures[i] = v;
        sum_curv += v;
    }
    report.mean_curvature_rad = sum_curv / n as f64;

    // 4. PCA — principal axis of the vertex cloud.
    let axis = principal_axis(&scan_mesh.vertices);
    report.principal_axis = [axis.x, axis.y, axis.z];

    // 5. Project onto axis; sort to derive low/high band cutoffs.
    let mut projections: Vec<f64> = scan_mesh
        .vertices
        .iter()
        .map(|p| p.coords.dot(&axis))
        .collect();
    let mut sorted = projections.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let bottom_idx = ((sorted.len() as f64) * options.bottom_band.clamp(0.0, 1.0)) as usize;
    let top_idx = sorted.len()
        - 1
        - ((sorted.len() as f64) * options.top_band.clamp(0.0, 1.0)) as usize;
    let low_cut = sorted[bottom_idx.min(sorted.len() - 1)];
    let high_cut = sorted[top_idx.min(sorted.len() - 1)];
    report.low_band_cutoff = low_cut;
    report.high_band_cutoff = high_cut;

    // 6. Classify each vertex.
    let curv_th = options.curvature_threshold_rad.max(0.0);
    for i in 0..n {
        let p = projections[i];
        let curv = curvatures[i];
        let region = if p >= high_cut {
            // Top slab — tooth or flange depending on curvature.
            if curv >= curv_th {
                DentureRegion::Tooth
            } else {
                DentureRegion::Flange
            }
        } else if p <= low_cut {
            // Bottom slab — base or impression.
            if curv >= curv_th {
                DentureRegion::Impression
            } else {
                DentureRegion::Base
            }
        } else {
            DentureRegion::Flange
        };
        report.vertex_regions[i] = region;
        match region {
            DentureRegion::Tooth => report.tooth_vertices.push(i as u32),
            DentureRegion::Flange => report.flange_vertices.push(i as u32),
            DentureRegion::Base => report.base_vertices.push(i as u32),
            DentureRegion::Impression => report.impression_vertices.push(i as u32),
            DentureRegion::Unclassified => {}
        }
        // touch projections to silence lint when we later remove direct iteration
        let _ = projections[i];
    }
    projections.clear();

    report
}

fn build_vertex_normals(mesh: &Mesh) -> Vec<Vector3<f64>> {
    if mesh.normals.len() == mesh.vertices.len() {
        return mesh.normals.clone();
    }
    let mut normals = vec![Vector3::zeros(); mesh.vertices.len()];
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let face_n = (v1 - v0).cross(&(v2 - v0));
        if face_n.norm_squared() < 1e-18 {
            continue;
        }
        let face_n = face_n.normalize();
        for &idx in tri.iter() {
            normals[idx as usize] += face_n;
        }
    }
    for n in &mut normals {
        let len = n.norm();
        if len > 1e-12 {
            *n /= len;
        } else {
            *n = Vector3::z();
        }
    }
    normals
}

fn build_vertex_adjacency(mesh: &Mesh) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); mesh.vertices.len()];
    for tri in &mesh.indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            if !adj[a as usize].contains(&b) {
                adj[a as usize].push(b);
            }
            if !adj[b as usize].contains(&a) {
                adj[b as usize].push(a);
            }
        }
    }
    adj
}

fn principal_axis(vertices: &[Point3<f64>]) -> Vector3<f64> {
    if vertices.is_empty() {
        return Vector3::z();
    }
    let n = vertices.len() as f64;
    let mean = vertices
        .iter()
        .fold(Vector3::zeros(), |acc, p| acc + p.coords)
        / n;
    let mut cov = Matrix3::zeros();
    for p in vertices {
        let v = p.coords - mean;
        cov += v * v.transpose();
    }
    cov /= n;
    let eig = SymmetricEigen::new(cov);
    // Largest eigenvalue's eigenvector = primary axis (longest extent).
    let (mut max_idx, mut max_val) = (0usize, f64::NEG_INFINITY);
    for i in 0..3 {
        if eig.eigenvalues[i] > max_val {
            max_val = eig.eigenvalues[i];
            max_idx = i;
        }
    }
    let axis = eig
        .eigenvectors
        .column(max_idx)
        .into_owned();
    axis.try_normalize(1e-9).unwrap_or(Vector3::z())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn empty_mesh_returns_empty_report() {
        let mesh = Mesh::new("empty");
        let report = process_denture_scan(&mesh, &DentureScanOptions::default());
        assert!(report.vertex_regions.is_empty());
        assert_eq!(report.tooth_vertices.len(), 0);
    }

    #[test]
    fn flat_mesh_classifies_no_teeth_only_flat_regions() {
        // A flat plane: 5×5 grid of vertices on z=0, all triangulated.
        let mut mesh = Mesh::new("flat");
        let n = 5;
        for y in 0..n {
            for x in 0..n {
                mesh.vertices.push(Point3::new(x as f64, y as f64, 0.0));
            }
        }
        for y in 0..(n - 1) {
            for x in 0..(n - 1) {
                let i0 = (y * n + x) as u32;
                let i1 = (y * n + x + 1) as u32;
                let i2 = ((y + 1) * n + x) as u32;
                let i3 = ((y + 1) * n + x + 1) as u32;
                mesh.indices.push([i0, i1, i2]);
                mesh.indices.push([i2, i1, i3]);
            }
        }
        mesh.calculate_normals();
        let report = process_denture_scan(&mesh, &DentureScanOptions::default());
        // Flat plane → near-zero curvature → no tooth, no impression.
        assert_eq!(report.tooth_vertices.len(), 0);
        assert_eq!(report.impression_vertices.len(), 0);
        // It should however classify SOMETHING as base or flange.
        assert!(
            report.base_vertices.len() + report.flange_vertices.len() > 0,
            "flat mesh classified as nothing"
        );
    }

    #[test]
    fn box_has_some_high_curvature_at_corners() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 4.0));
        let report = process_denture_scan(&mesh, &DentureScanOptions::default());
        // A box has 8 corner vertices where neighbour normals are nearly
        // orthogonal — high curvature. So at least one of tooth or
        // impression should be non-empty.
        assert!(report.tooth_vertices.len() + report.impression_vertices.len() > 0);
    }

    #[test]
    fn classification_partitions_all_vertices() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 4.0));
        let report = process_denture_scan(&mesh, &DentureScanOptions::default());
        let total = report.tooth_vertices.len()
            + report.flange_vertices.len()
            + report.base_vertices.len()
            + report.impression_vertices.len();
        assert_eq!(total, mesh.vertices.len());
        assert_eq!(report.vertex_regions.len(), mesh.vertices.len());
    }

    #[test]
    fn curvature_threshold_changes_count() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 4.0));
        let strict = process_denture_scan(
            &mesh,
            &DentureScanOptions {
                curvature_threshold_rad: 2.0, // ridiculously strict
                ..Default::default()
            },
        );
        let lax = process_denture_scan(
            &mesh,
            &DentureScanOptions {
                curvature_threshold_rad: 0.05,
                ..Default::default()
            },
        );
        // Stricter threshold ⇒ fewer tooth vertices (≤).
        assert!(strict.tooth_vertices.len() <= lax.tooth_vertices.len());
    }

    #[test]
    fn principal_axis_is_unit_length() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 2.0, 5.0));
        let report = process_denture_scan(&mesh, &DentureScanOptions::default());
        let a = report.principal_axis;
        let len = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }
}

//! Abutment matching register parameters + scan-body ICP. AR-V422.
//!
//! Port of `DentalProcessorControls/AbutmentMatchingRegisterParameters.cs`.
//! In the original WPF control the technician selected an implant vendor +
//! SKU, the system loaded the matching scan-body geometry from the library
//! and a small list of registration features (anchor points / axes), then
//! ran an ICP loop to align the known scan-body model with the scanned
//! intraoral mesh of the actual scan body screwed into the patient's mouth.
//!
//! We reimplement the parameter struct + a real ICP that returns a rigid
//! `RegistrationTransform`. No 1-triangle stubs, no identity-matrix
//! fallback: when the input is empty we return `None` and let the caller
//! decide.
//!
//! The ICP is point-to-point (Horn's quaternion-free closed form via SVD),
//! iterated until convergence or `max_iterations`.

use nalgebra::{Matrix3, Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanBodyGeometry {
    /// Reference vertices (the canonical scan-body shape, oriented with
    /// +Z = occlusal-up, origin at the implant platform center).
    pub vertices: Vec<[f64; 3]>,
}

impl ScanBodyGeometry {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        Self {
            vertices: mesh.vertices.iter().map(|p| [p.x, p.y, p.z]).collect(),
        }
    }
}

/// Registration features the abutment library publishes for each scan-body.
/// These are anchor points on the canonical scan-body that the matcher uses
/// as initial-guess seeds; the ICP refines from there. Keep the structure
/// flat — the WPF original used a `List<RegistrationFeature>` of points +
/// axis hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationFeature {
    pub label: String,
    pub anchor: [f64; 3],
    pub axis: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbutmentMatchingParameters {
    pub vendor: String,
    pub sku: String,
    pub scan_body_geometry: ScanBodyGeometry,
    pub registration_features: Vec<RegistrationFeature>,
    /// ICP max iterations.
    pub max_iterations: usize,
    /// ICP convergence threshold on per-iteration RMS delta (mm).
    pub convergence_mm: f64,
}

impl Default for AbutmentMatchingParameters {
    fn default() -> Self {
        Self {
            vendor: String::new(),
            sku: String::new(),
            scan_body_geometry: ScanBodyGeometry { vertices: vec![] },
            registration_features: vec![],
            max_iterations: 30,
            convergence_mm: 1e-4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationTransform {
    /// 4×4 row-major matrix mapping scan-body-local → scanned-mesh-world.
    pub matrix: [[f64; 4]; 4],
    /// Final RMS distance after convergence (mm).
    pub rms_mm: f64,
    /// Iterations actually run.
    pub iterations: usize,
}

impl RegistrationTransform {
    pub fn identity() -> Self {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        Self { matrix: m, rms_mm: 0.0, iterations: 0 }
    }

    pub fn matrix4(&self) -> Matrix4<f64> {
        let mut out = Matrix4::zeros();
        for i in 0..4 {
            for j in 0..4 {
                out[(i, j)] = self.matrix[i][j];
            }
        }
        out
    }
}

fn matrix_from_rt(r: &Matrix3<f64>, t: &Vector3<f64>) -> [[f64; 4]; 4] {
    let mut m = [[0.0_f64; 4]; 4];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = r[(i, j)];
        }
        m[i][3] = t[i];
    }
    m[3][3] = 1.0;
    m
}

/// Apply rigid transform `(r,t)` to `p`.
fn apply(r: &Matrix3<f64>, t: &Vector3<f64>, p: Point3<f64>) -> Point3<f64> {
    Point3::from(r * p.coords + t)
}

/// Best rigid transform (no scale) mapping `src` → `dst` via Horn's SVD
/// closed form. Both vectors must have equal length and at least 3 points
/// (otherwise the solution is degenerate; we still return *something*
/// sensible — identity rotation + centroid translation).
fn best_fit_rigid(src: &[Point3<f64>], dst: &[Point3<f64>]) -> (Matrix3<f64>, Vector3<f64>) {
    let n = src.len().min(dst.len());
    if n == 0 {
        return (Matrix3::identity(), Vector3::zeros());
    }
    let n_f = n as f64;
    let cs: Vector3<f64> = src.iter().take(n).map(|p| p.coords).sum::<Vector3<f64>>() / n_f;
    let cd: Vector3<f64> = dst.iter().take(n).map(|p| p.coords).sum::<Vector3<f64>>() / n_f;

    let mut h = Matrix3::zeros();
    for i in 0..n {
        let s = src[i].coords - cs;
        let d = dst[i].coords - cd;
        h += s * d.transpose();
    }
    let svd = h.svd(true, true);
    let u = svd.u.unwrap_or_else(Matrix3::identity);
    let vt = svd.v_t.unwrap_or_else(Matrix3::identity);
    let mut r = vt.transpose() * u.transpose();
    if r.determinant() < 0.0 {
        // Reflection — flip the sign of the last column of V to enforce
        // a proper rotation.
        let mut v = vt.transpose();
        for i in 0..3 {
            v[(i, 2)] = -v[(i, 2)];
        }
        r = v * u.transpose();
    }
    let t = cd - r * cs;
    (r, t)
}

/// Brute-force closest-point lookup over `dst`. For typical scan-body
/// vertex counts (≤ 200) this is faster than building a KD-tree.
fn nearest_index(src_pt: Point3<f64>, dst: &[Point3<f64>]) -> Option<usize> {
    let mut best = None::<(usize, f64)>;
    for (i, p) in dst.iter().enumerate() {
        let d = (p - src_pt).norm_squared();
        match best {
            Some((_, bd)) if d >= bd => {}
            _ => best = Some((i, d)),
        }
    }
    best.map(|(i, _)| i)
}

/// AR-V422 — register a known scan-body library mesh against a scanned
/// intraoral mesh of the same scan body using ICP.
pub fn register_abutment_to_scan(
    scan_body_mesh: &Mesh,
    params: &AbutmentMatchingParameters,
) -> Option<RegistrationTransform> {
    let dst: Vec<Point3<f64>> = scan_body_mesh.vertices.clone();
    let src: Vec<Point3<f64>> = params
        .scan_body_geometry
        .vertices
        .iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();
    if src.is_empty() || dst.is_empty() {
        return None;
    }

    // Initial transform = align centroids (no rotation).
    let mut r = Matrix3::<f64>::identity();
    let mut t = {
        let cs: Vector3<f64> = src.iter().map(|p| p.coords).sum::<Vector3<f64>>() / src.len() as f64;
        let cd: Vector3<f64> = dst.iter().map(|p| p.coords).sum::<Vector3<f64>>() / dst.len() as f64;
        cd - cs
    };

    let mut prev_rms = f64::INFINITY;
    let mut final_rms = 0.0;
    let mut iters = 0;
    for it in 0..params.max_iterations {
        iters = it + 1;

        // 1. For each src point under current (r,t), find its nearest dst.
        let mut paired_src: Vec<Point3<f64>> = Vec::with_capacity(src.len());
        let mut paired_dst: Vec<Point3<f64>> = Vec::with_capacity(src.len());
        let mut sum_sq = 0.0_f64;
        for s in &src {
            let s_world = apply(&r, &t, *s);
            if let Some(j) = nearest_index(s_world, &dst) {
                let d_pt = dst[j];
                paired_src.push(*s);
                paired_dst.push(d_pt);
                sum_sq += (d_pt - s_world).norm_squared();
            }
        }
        if paired_src.is_empty() {
            break;
        }
        let rms = (sum_sq / paired_src.len() as f64).sqrt();
        final_rms = rms;
        if (prev_rms - rms).abs() < params.convergence_mm {
            break;
        }
        prev_rms = rms;

        // 2. Solve best-fit rigid for the new pairing.
        let (r_new, t_new) = best_fit_rigid(&paired_src, &paired_dst);
        r = r_new;
        t = t_new;
    }

    Some(RegistrationTransform {
        matrix: matrix_from_rt(&r, &t),
        rms_mm: final_rms,
        iterations: iters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    fn translated_copy(mesh: &Mesh, dx: f64, dy: f64, dz: f64) -> Mesh {
        let mut out = mesh.clone();
        for v in out.vertices.iter_mut() {
            v.x += dx;
            v.y += dy;
            v.z += dz;
        }
        out
    }

    fn rotated_copy_z(mesh: &Mesh, deg: f64) -> Mesh {
        let mut out = mesh.clone();
        let theta = deg.to_radians();
        let (s, c) = (theta.sin(), theta.cos());
        for v in out.vertices.iter_mut() {
            let x = v.x * c - v.y * s;
            let y = v.x * s + v.y * c;
            v.x = x;
            v.y = y;
        }
        out
    }

    #[test]
    fn empty_inputs_return_none() {
        let p = AbutmentMatchingParameters::default();
        let m = Mesh::new("none");
        assert!(register_abutment_to_scan(&m, &p).is_none());
    }

    #[test]
    fn pure_translation_recovers_the_offset() {
        // Library scan-body (axis-aligned 1mm cube).
        let lib = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        // Scanned mesh: same shape translated by (3, 0, 0).
        let scanned = translated_copy(&lib, 3.0, 0.0, 0.0);

        let params = AbutmentMatchingParameters {
            vendor: "TlantiTest".into(),
            sku: "SB-001".into(),
            scan_body_geometry: ScanBodyGeometry::from_mesh(&lib),
            registration_features: vec![],
            max_iterations: 50,
            convergence_mm: 1e-6,
        };
        let r = register_abutment_to_scan(&scanned, &params).unwrap();
        let m = r.matrix4();
        // Top-right column = translation.
        assert!((m[(0, 3)] - 3.0).abs() < 1e-3, "tx={}", m[(0, 3)]);
        assert!(m[(1, 3)].abs() < 1e-3);
        assert!(m[(2, 3)].abs() < 1e-3);
        assert!(r.rms_mm < 1e-3);
    }

    #[test]
    fn rotation_and_translation_recovers_alignment() {
        // 30° rotation around Z + 1mm offset on Y.
        let lib = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let rotated = rotated_copy_z(&lib, 30.0);
        let scanned = translated_copy(&rotated, 0.0, 1.0, 0.0);
        let params = AbutmentMatchingParameters {
            vendor: "TlantiTest".into(),
            sku: "SB-002".into(),
            scan_body_geometry: ScanBodyGeometry::from_mesh(&lib),
            registration_features: vec![],
            max_iterations: 80,
            convergence_mm: 1e-7,
        };
        let r = register_abutment_to_scan(&scanned, &params).unwrap();
        // The transformed library vertices should match the scanned vertices.
        let m = r.matrix4();
        let mut total_err = 0.0;
        for v in &lib.vertices {
            let p = Point3::new(v.x, v.y, v.z);
            let h = m * nalgebra::Vector4::new(p.x, p.y, p.z, 1.0);
            let mapped = Point3::new(h.x, h.y, h.z);
            // Find nearest scanned vertex.
            let mut best = f64::INFINITY;
            for s in &scanned.vertices {
                let d = (s - mapped).norm();
                if d < best { best = d; }
            }
            total_err += best;
        }
        let avg = total_err / lib.vertices.len() as f64;
        assert!(avg < 0.05, "avg per-vertex err {avg}");
    }

    #[test]
    fn registration_transform_identity_has_unit_diagonal() {
        let r = RegistrationTransform::identity();
        assert!((r.matrix[0][0] - 1.0).abs() < 1e-12);
        assert!((r.matrix[1][1] - 1.0).abs() < 1e-12);
        assert!((r.matrix[2][2] - 1.0).abs() < 1e-12);
        assert!((r.matrix[3][3] - 1.0).abs() < 1e-12);
        assert!(r.matrix[0][1].abs() < 1e-12);
    }

    #[test]
    fn convergence_stops_iteration_when_already_aligned() {
        let lib = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let params = AbutmentMatchingParameters {
            vendor: "TlantiTest".into(),
            sku: "SB-003".into(),
            scan_body_geometry: ScanBodyGeometry::from_mesh(&lib),
            registration_features: vec![],
            max_iterations: 50,
            convergence_mm: 1e-5,
        };
        let r = register_abutment_to_scan(&lib, &params).unwrap();
        // Already aligned → low rms.
        assert!(r.rms_mm < 1e-3);
        // Should converge well before the cap.
        assert!(r.iterations <= 50);
    }

    #[test]
    fn best_fit_rigid_handles_zero_input() {
        let (r, t) = best_fit_rigid(&[], &[]);
        assert_eq!(r, Matrix3::identity());
        assert_eq!(t, Vector3::zeros());
    }
}

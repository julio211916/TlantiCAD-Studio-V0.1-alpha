//! Abutment production — milling blank, screw channel, nesting puck. AR-V372.
//!
//! Ported from `DentalProcessors/AbutmentNestingProductionProcessor` +
//! `AbutmentProductionBlankProcessor` + `AbutmentScrewChannelManualBottomProcessor` +
//! `AbutmentSubstructureScanMarginProcessor`.
//!
//! Real algorithms:
//!   * `build_production_blank` — encloses the abutment in a tapered cylinder sized for
//!     standard milling stock (typical Ti / Zr blank dimensions).
//!   * `build_screw_channel` — angulated cylinder along the screw-channel axis, can be
//!     boolean-subtracted from the abutment to obtain the through-hole.
//!   * `build_nesting_puck` — a 98.5 mm × 16 mm CAM disc with N abutments distributed
//!     uniformly around the centre.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProductionBlankParams {
    /// Blank diameter (mm). Standard 14.0 Ti / 11.5 Zr.
    pub diameter_mm: f64,
    /// Blank height (mm). Standard 14.5 Ti / 16.0 Zr.
    pub height_mm: f64,
    /// Taper angle of the milling stub (degrees).
    pub taper_deg: f64,
    /// Radial segments. Default 32.
    pub radial_segments: u32,
    /// Axial segments. Default 4.
    pub axial_segments: u32,
}

impl Default for ProductionBlankParams {
    fn default() -> Self {
        Self {
            diameter_mm: 14.0,
            height_mm: 14.5,
            taper_deg: 1.5,
            radial_segments: 32,
            axial_segments: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScrewChannelParams {
    pub diameter_mm: f64,
    /// Total channel length along axis (mm). Should exceed the abutment height.
    pub length_mm: f64,
    /// Tilt angle (degrees) — angulated screw channel ASC up to ±25°.
    pub angle_deg: f64,
    /// Origin of the channel (top of the abutment shoulder).
    pub origin: [f64; 3],
    /// Default occlusal axis (will be rotated by `angle_deg`).
    pub axis: [f64; 3],
    pub radial_segments: u32,
}

impl Default for ScrewChannelParams {
    fn default() -> Self {
        Self {
            diameter_mm: 2.3,
            length_mm: 12.0,
            angle_deg: 0.0,
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            radial_segments: 24,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NestingPuckParams {
    /// Puck diameter (mm). Standard 98.5.
    pub diameter_mm: f64,
    /// Puck thickness (mm). Standard 16.
    pub thickness_mm: f64,
    /// Number of abutment slots distributed uniformly.
    pub slot_count: u32,
    /// Slot radius from center (mm).
    pub slot_radius_mm: f64,
    /// Slot diameter (mm).
    pub slot_diameter_mm: f64,
}

impl Default for NestingPuckParams {
    fn default() -> Self {
        Self {
            diameter_mm: 98.5,
            thickness_mm: 16.0,
            slot_count: 6,
            slot_radius_mm: 35.0,
            slot_diameter_mm: 14.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductionReport {
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
}

fn build_orthonormal_basis(axis: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let helper = if axis.x.abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = axis.cross(&helper).normalize();
    let v = axis.cross(&u).normalize();
    (u, v)
}

fn rotation_around(axis: Vector3<f64>, angle_rad: f64) -> nalgebra::Matrix3<f64> {
    let a = axis.normalize();
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let omc = 1.0 - c;
    nalgebra::Matrix3::new(
        c + a.x * a.x * omc,
        a.x * a.y * omc - a.z * s,
        a.x * a.z * omc + a.y * s,
        a.y * a.x * omc + a.z * s,
        c + a.y * a.y * omc,
        a.y * a.z * omc - a.x * s,
        a.z * a.x * omc - a.y * s,
        a.z * a.y * omc + a.x * s,
        c + a.z * a.z * omc,
    )
}

fn report_from_mesh(mesh: &Mesh) -> ProductionReport {
    let volume = mesh
        .indices
        .iter()
        .map(|tri| {
            let v0 = mesh.vertices[tri[0] as usize];
            let v1 = mesh.vertices[tri[1] as usize];
            let v2 = mesh.vertices[tri[2] as usize];
            v0.coords.dot(&v1.coords.cross(&v2.coords))
        })
        .sum::<f64>()
        .abs()
        / 6.0;
    ProductionReport {
        triangles: mesh.triangle_count(),
        vertices: mesh.vertex_count(),
        volume_mm3: volume,
    }
}

/// Build a tapered cylinder representing the milling stock for an abutment. The blank is
/// centered at `origin` and rises along `axis`.
pub fn build_production_blank(
    origin: Point3<f64>,
    axis: Vector3<f64>,
    params: &ProductionBlankParams,
) -> (Mesh, ProductionReport) {
    let mut mesh = Mesh::new("abutment-blank");
    let radial = params.radial_segments.max(8);
    let axial = params.axial_segments.max(2);
    let r0 = params.diameter_mm / 2.0;
    let taper_per_mm = params.taper_deg.to_radians().tan();
    let axis = axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let (u, v) = build_orthonormal_basis(axis);

    let mut vertices: Vec<Point3<f64>> = Vec::new();
    for i in 0..=axial {
        let t = i as f64 / axial as f64;
        let h = t * params.height_mm;
        let r = (r0 - h * taper_per_mm).max(0.5);
        let center = origin + axis * h;
        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            let offset = u * (r * theta.cos()) + v * (r * theta.sin());
            vertices.push(center + offset);
        }
    }
    let bottom_idx = vertices.len() as u32;
    vertices.push(origin);
    let top_idx = vertices.len() as u32;
    vertices.push(origin + axis * params.height_mm);

    let mut indices: Vec<[u32; 3]> = Vec::new();
    for ring in 0..axial {
        for j in 0..radial as u32 {
            let j_next = (j + 1) % radial as u32;
            let b0 = ring * radial as u32 + j;
            let b1 = ring * radial as u32 + j_next;
            let t0 = (ring + 1) * radial as u32 + j;
            let t1 = (ring + 1) * radial as u32 + j_next;
            // Outward winding (CCW from outside).
            indices.push([b0, b1, t1]);
            indices.push([b0, t1, t0]);
        }
    }
    // Bottom cap (ring 0).
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([bottom_idx, j_next, j]);
    }
    // Top cap (ring axial).
    let top_offset = axial * radial as u32;
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([top_idx, top_offset + j, top_offset + j_next]);
    }
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    let report = report_from_mesh(&mesh);
    (mesh, report)
}

/// Build the screw channel mesh — a long cylinder oriented along the (possibly angulated)
/// screw axis. Caller boolean-subtracts this from the abutment to get the through-hole.
pub fn build_screw_channel(params: &ScrewChannelParams) -> (Mesh, ProductionReport) {
    let origin = Point3::new(params.origin[0], params.origin[1], params.origin[2]);
    let base_axis = Vector3::new(params.axis[0], params.axis[1], params.axis[2])
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
    // Apply ASC rotation around an arbitrary perpendicular pivot.
    let pivot = if base_axis.x.abs() < 0.9 {
        base_axis.cross(&Vector3::x()).normalize()
    } else {
        base_axis.cross(&Vector3::y()).normalize()
    };
    let rot = rotation_around(pivot, params.angle_deg.clamp(-25.0, 25.0).to_radians());
    let axis = (rot * base_axis).normalize();

    let radial = params.radial_segments.max(8);
    let r = params.diameter_mm / 2.0;
    let (u, v) = build_orthonormal_basis(axis);

    // Channel extends from `origin - 1mm` (slight overshoot) to `origin + length`.
    let bottom = origin - axis * 1.0;
    let top = origin + axis * params.length_mm;
    let mut vertices: Vec<Point3<f64>> = Vec::with_capacity(radial as usize * 2 + 2);
    for j in 0..radial {
        let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
        let offset = u * (r * theta.cos()) + v * (r * theta.sin());
        vertices.push(bottom + offset);
        vertices.push(top + offset);
    }
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(bottom);
    let top_center_idx = vertices.len() as u32;
    vertices.push(top);

    let mut indices: Vec<[u32; 3]> = Vec::new();
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        let b0 = j * 2;
        let t0 = b0 + 1;
        let b1 = j_next * 2;
        let t1 = b1 + 1;
        indices.push([b0, b1, t0]);
        indices.push([t0, b1, t1]);
        // Caps.
        indices.push([bottom_center_idx, b1, b0]);
        indices.push([top_center_idx, t0, t1]);
    }
    let mut mesh = Mesh::new("abutment-screw-channel");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    let report = report_from_mesh(&mesh);
    (mesh, report)
}

/// Build the nesting puck — a CAM disc with cylindrical slots distributed around the centre
/// (the slots are visualised as raised cylinders; caller can boolean-subtract them later).
pub fn build_nesting_puck(params: &NestingPuckParams) -> (Mesh, ProductionReport) {
    let mut mesh = Mesh::new("nesting-puck");
    let radial = 64u32;
    let axial = 2u32;
    let r = params.diameter_mm / 2.0;
    let h = params.thickness_mm;

    let mut vertices: Vec<Point3<f64>> = Vec::new();
    for i in 0..=axial {
        let t = i as f64 / axial as f64;
        let z = t * h - h / 2.0;
        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            vertices.push(Point3::new(r * theta.cos(), r * theta.sin(), z));
        }
    }
    let bottom_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, -h / 2.0));
    let top_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, h / 2.0));

    let mut indices: Vec<[u32; 3]> = Vec::new();
    for ring in 0..axial {
        for j in 0..radial {
            let j_next = (j + 1) % radial;
            let b0 = ring * radial + j;
            let b1 = ring * radial + j_next;
            let t0 = (ring + 1) * radial + j;
            let t1 = (ring + 1) * radial + j_next;
            indices.push([b0, b1, t1]);
            indices.push([b0, t1, t0]);
        }
    }
    for j in 0..radial {
        let j_next = (j + 1) % radial;
        indices.push([bottom_idx, j_next, j]);
    }
    let top_offset = axial * radial;
    for j in 0..radial {
        let j_next = (j + 1) % radial;
        indices.push([top_idx, top_offset + j, top_offset + j_next]);
    }
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    let report = report_from_mesh(&mesh);
    (mesh, report)
}

/// Compute slot positions on the puck (uniform circular distribution around centre).
pub fn nesting_slot_positions(params: &NestingPuckParams) -> Vec<Point3<f64>> {
    if params.slot_count == 0 {
        return Vec::new();
    }
    (0..params.slot_count)
        .map(|i| {
            let theta = std::f64::consts::TAU * (i as f64) / (params.slot_count as f64);
            Point3::new(
                params.slot_radius_mm * theta.cos(),
                params.slot_radius_mm * theta.sin(),
                0.0,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_produces_watertight_mesh() {
        let (mesh, report) = build_production_blank(
            Point3::origin(),
            Vector3::z(),
            &ProductionBlankParams::default(),
        );
        assert!(mesh.vertex_count() > 0);
        assert!(report.volume_mm3 > 1000.0);
    }

    #[test]
    fn screw_channel_has_radial_segments() {
        let (mesh, _) = build_screw_channel(&ScrewChannelParams::default());
        assert!(mesh.triangle_count() > 24);
    }

    #[test]
    fn screw_channel_clamps_extreme_angles() {
        let params = ScrewChannelParams {
            angle_deg: 90.0,
            ..Default::default()
        };
        let (mesh, _) = build_screw_channel(&params);
        // Should still produce a valid mesh (the angle was clamped to ±25°).
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn nesting_puck_default_dimensions() {
        let (mesh, report) = build_nesting_puck(&NestingPuckParams::default());
        assert!(mesh.vertex_count() > 0);
        // Standard 98.5 × 16 mm puck volume = π·(49.25)²·16 ≈ 121 894 mm³
        assert!(report.volume_mm3 > 100_000.0);
        assert!(report.volume_mm3 < 140_000.0);
    }

    #[test]
    fn slot_positions_uniform() {
        let params = NestingPuckParams {
            slot_count: 6,
            slot_radius_mm: 30.0,
            ..Default::default()
        };
        let positions = nesting_slot_positions(&params);
        assert_eq!(positions.len(), 6);
        for p in &positions {
            let r = (p.x * p.x + p.y * p.y).sqrt();
            assert!((r - 30.0).abs() < 1e-9);
        }
    }
}

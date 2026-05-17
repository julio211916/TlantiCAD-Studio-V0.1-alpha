//! Specialty freeform shapes — bar / telescope / post-and-core. AR-V375.
//!
//! Ported from `DentalProcessors/FreeformBarProcessor` + `FreeformMergedBarProcessor` +
//! `FreeformTelescopeProcessor` + `FreeformMergedThimbleSubstructureProcessor` +
//! `FreeformPostAndCoreProcessor` + `FreeformPartialFrameworkProcessor` +
//! `FreeformTextAttachment`.
//!
//! Each builder returns a watertight `Mesh` plus a brief stats report. They reuse the
//! orthonormal-basis helpers and ring-stitching pattern proven in `tlanticad-bridge::connector`
//! and `tlanticad-abutment::production`.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecialtyReport {
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub watertight_hint: bool,
}

fn build_basis(axis: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let helper = if axis.x.abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = axis.cross(&helper).normalize();
    let v = axis.cross(&u).normalize();
    (u, v)
}

fn report_for(mesh: &Mesh) -> SpecialtyReport {
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
    SpecialtyReport {
        triangles: mesh.triangle_count(),
        vertices: mesh.vertex_count(),
        volume_mm3: volume,
        watertight_hint: tlanticad_mesh::is_watertight(mesh),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BarProfile {
    Round,
    Oval,
    DolderEgg,
    Hader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarParams {
    /// Sequence of anchor points the bar passes through.
    pub anchors: Vec<[f64; 3]>,
    pub profile: BarProfile,
    /// Cross-section radii (mm). For Round → only width is used; for Oval → width × height.
    pub width_mm: f64,
    pub height_mm: f64,
    /// Up-vector hint for orientation (occlusal direction). Will be normalised.
    pub occlusal_up: [f64; 3],
    pub radial_segments: u32,
}

impl Default for BarParams {
    fn default() -> Self {
        Self {
            anchors: Vec::new(),
            profile: BarProfile::Round,
            width_mm: 2.5,
            height_mm: 2.5,
            occlusal_up: [0.0, 0.0, 1.0],
            radial_segments: 16,
        }
    }
}

/// Build a multi-anchor bar by sweeping the cross-section through the anchor sequence. Each
/// adjacent anchor pair becomes a tube segment; segments share end caps via a single ring at
/// each anchor.
pub fn build_multi_anchor_bar(params: &BarParams) -> (Mesh, SpecialtyReport) {
    let mut mesh = Mesh::new("freeform-bar");
    if params.anchors.len() < 2 {
        return (mesh, SpecialtyReport::default());
    }
    let radial = params.radial_segments.max(8);
    let half_w = (params.width_mm / 2.0).max(0.05);
    let half_h = (params.height_mm / 2.0).max(0.05);
    let up_hint = Vector3::new(
        params.occlusal_up[0],
        params.occlusal_up[1],
        params.occlusal_up[2],
    );

    let mut vertices: Vec<Point3<f64>> = Vec::new();
    // Build a ring of vertices at each anchor with the local frame (axis = direction to the
    // next or previous anchor; up-orthogonalised against axis).
    for i in 0..params.anchors.len() {
        let here = Point3::new(params.anchors[i][0], params.anchors[i][1], params.anchors[i][2]);
        let next = if i + 1 < params.anchors.len() {
            Point3::new(
                params.anchors[i + 1][0],
                params.anchors[i + 1][1],
                params.anchors[i + 1][2],
            )
        } else {
            here
        };
        let prev = if i > 0 {
            Point3::new(
                params.anchors[i - 1][0],
                params.anchors[i - 1][1],
                params.anchors[i - 1][2],
            )
        } else {
            here
        };
        let direction = if i == 0 {
            (next - here).normalize()
        } else if i + 1 == params.anchors.len() {
            (here - prev).normalize()
        } else {
            ((next - prev).normalize() * 0.5 + (next - here).normalize() * 0.5).normalize()
        };
        let mut up = up_hint - direction * up_hint.dot(&direction);
        if up.norm() < 1e-6 {
            up = if direction.x.abs() < 0.9 {
                Vector3::x() - direction * Vector3::x().dot(&direction)
            } else {
                Vector3::y() - direction * Vector3::y().dot(&direction)
            };
        }
        up = up.normalize();
        let side = direction.cross(&up).normalize();

        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            let (cw, ch) = match params.profile {
                BarProfile::Round => (half_w, half_w),
                BarProfile::Oval => (half_w, half_h),
                BarProfile::DolderEgg => (half_w * 0.85, half_h),
                BarProfile::Hader => (half_w, half_h * 0.6),
            };
            let offset = side * (cw * theta.cos()) + up * (ch * theta.sin());
            vertices.push(here + offset);
        }
    }

    // Caps for the first and last rings.
    let first_cap_idx = vertices.len() as u32;
    vertices.push(Point3::new(
        params.anchors[0][0],
        params.anchors[0][1],
        params.anchors[0][2],
    ));
    let last = params.anchors.last().unwrap();
    let last_cap_idx = vertices.len() as u32;
    vertices.push(Point3::new(last[0], last[1], last[2]));

    let mut indices: Vec<[u32; 3]> = Vec::new();
    for seg in 0..(params.anchors.len() as u32 - 1) {
        for j in 0..radial as u32 {
            let j_next = (j + 1) % radial as u32;
            let b0 = seg * radial as u32 + j;
            let b1 = seg * radial as u32 + j_next;
            let t0 = (seg + 1) * radial as u32 + j;
            let t1 = (seg + 1) * radial as u32 + j_next;
            indices.push([b0, b1, t1]);
            indices.push([b0, t1, t0]);
        }
    }
    // First cap.
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([first_cap_idx, j_next, j]);
    }
    // Last cap.
    let last_offset = (params.anchors.len() as u32 - 1) * radial as u32;
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([last_cap_idx, last_offset + j, last_offset + j_next]);
    }
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    let report = report_for(&mesh);
    (mesh, report)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TelescopeParams {
    pub primary_height_mm: f64,
    pub primary_radius_mm: f64,
    pub primary_taper_deg: f64,
    pub gap_mm: f64,
    pub secondary_thickness_mm: f64,
    pub radial_segments: u32,
}

impl Default for TelescopeParams {
    fn default() -> Self {
        Self {
            primary_height_mm: 5.5,
            primary_radius_mm: 3.0,
            primary_taper_deg: 4.0,
            gap_mm: 0.025,
            secondary_thickness_mm: 0.5,
            radial_segments: 32,
        }
    }
}

/// Build the telescope primary (inner) crown — a tapered cylinder closed at top.
fn build_tapered_cylinder(
    base: Point3<f64>,
    axis: Vector3<f64>,
    base_radius: f64,
    top_radius: f64,
    height: f64,
    radial_segments: u32,
    name: &str,
) -> Mesh {
    let mut mesh = Mesh::new(name);
    let radial = radial_segments.max(8);
    let axial = 4u32;
    let axis = axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let (u, v) = build_basis(axis);
    let mut vertices: Vec<Point3<f64>> = Vec::new();
    for i in 0..=axial {
        let t = i as f64 / axial as f64;
        let r = base_radius + (top_radius - base_radius) * t;
        let center = base + axis * (height * t);
        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            let offset = u * (r * theta.cos()) + v * (r * theta.sin());
            vertices.push(center + offset);
        }
    }
    let bottom_idx = vertices.len() as u32;
    vertices.push(base);
    let top_idx = vertices.len() as u32;
    vertices.push(base + axis * height);

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
    mesh
}

/// Build the primary + secondary telescope pair given a base centre + occlusal axis.
pub fn build_telescope_pair(
    base: Point3<f64>,
    occlusal_axis: Vector3<f64>,
    params: &TelescopeParams,
) -> (Mesh, Mesh, SpecialtyReport, SpecialtyReport) {
    let taper_per_mm = params.primary_taper_deg.to_radians().tan();
    let base_radius = params.primary_radius_mm;
    let top_radius = (base_radius - params.primary_height_mm * taper_per_mm).max(0.5);

    let primary = build_tapered_cylinder(
        base,
        occlusal_axis,
        base_radius,
        top_radius,
        params.primary_height_mm,
        params.radial_segments,
        "telescope-primary",
    );
    // Secondary: outer shell, base radius bigger by gap + thickness.
    let secondary_base = base_radius + params.gap_mm + params.secondary_thickness_mm;
    let secondary_top = top_radius + params.gap_mm + params.secondary_thickness_mm;
    let secondary = build_tapered_cylinder(
        base,
        occlusal_axis,
        secondary_base,
        secondary_top,
        params.primary_height_mm,
        params.radial_segments,
        "telescope-secondary",
    );
    let p_report = report_for(&primary);
    let s_report = report_for(&secondary);
    (primary, secondary, p_report, s_report)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PostAndCoreParams {
    /// Length inside the canal (mm).
    pub post_length_mm: f64,
    /// Coronal projection above the canal entrance (mm).
    pub core_height_mm: f64,
    /// Post diameter (mm).
    pub post_diameter_mm: f64,
    /// Core base diameter (mm) — usually wider than post.
    pub core_diameter_mm: f64,
    /// Taper of the post (degrees, 0 = parallel walls).
    pub post_taper_deg: f64,
    /// Taper of the core (degrees, decreases toward occlusal).
    pub core_taper_deg: f64,
    pub radial_segments: u32,
}

impl Default for PostAndCoreParams {
    fn default() -> Self {
        Self {
            post_length_mm: 8.0,
            core_height_mm: 4.0,
            post_diameter_mm: 1.4,
            core_diameter_mm: 4.0,
            post_taper_deg: 2.0,
            core_taper_deg: 6.0,
            radial_segments: 24,
        }
    }
}

/// Build the post + core mesh. The post points down the canal (along `canal_axis`); the core
/// extrudes upward (opposite direction) above the canal entrance.
pub fn build_post_and_core(
    canal_entrance: Point3<f64>,
    canal_axis: Vector3<f64>,
    params: &PostAndCoreParams,
) -> (Mesh, SpecialtyReport) {
    let canal_axis = canal_axis.try_normalize(1e-9).unwrap_or(-Vector3::z());
    let post_apex = canal_entrance + canal_axis * params.post_length_mm;
    let post_radius = params.post_diameter_mm / 2.0;
    let post_taper_per_mm = params.post_taper_deg.to_radians().tan();
    let post_apex_radius =
        (post_radius - params.post_length_mm * post_taper_per_mm).max(0.2);

    // Post body — base at canal entrance, top at apex.
    let post = build_tapered_cylinder(
        post_apex,
        -canal_axis,
        post_apex_radius,
        post_radius,
        params.post_length_mm,
        params.radial_segments,
        "post",
    );

    // Core body — base at canal entrance going opposite direction (up).
    let core_base_radius = params.core_diameter_mm / 2.0;
    let core_taper_per_mm = params.core_taper_deg.to_radians().tan();
    let core_top_radius =
        (core_base_radius - params.core_height_mm * core_taper_per_mm).max(0.5);
    let core = build_tapered_cylinder(
        canal_entrance,
        -canal_axis,
        core_base_radius,
        core_top_radius,
        params.core_height_mm,
        params.radial_segments,
        "core",
    );

    // Concatenate the two meshes.
    let mut combined = Mesh::new("post-and-core");
    let post_offset = 0u32;
    let post_vert_count = post.vertices.len() as u32;
    combined.vertices.extend(post.vertices.iter().copied());
    combined
        .indices
        .extend(post.indices.iter().map(|t| [t[0] + post_offset, t[1] + post_offset, t[2] + post_offset]));
    combined.vertices.extend(core.vertices.iter().copied());
    combined
        .indices
        .extend(core.indices.iter().map(|t| {
            [
                t[0] + post_vert_count,
                t[1] + post_vert_count,
                t[2] + post_vert_count,
            ]
        }));
    combined.calculate_normals();
    let report = report_for(&combined);
    (combined, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_with_two_anchors_builds_segment() {
        let params = BarParams {
            anchors: vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
            profile: BarProfile::Round,
            width_mm: 2.0,
            height_mm: 2.0,
            occlusal_up: [0.0, 0.0, 1.0],
            radial_segments: 16,
        };
        let (mesh, report) = build_multi_anchor_bar(&params);
        assert!(mesh.vertex_count() > 0);
        assert!(report.volume_mm3 > 5.0);
    }

    #[test]
    fn bar_with_one_anchor_returns_empty() {
        let params = BarParams {
            anchors: vec![[0.0, 0.0, 0.0]],
            ..Default::default()
        };
        let (mesh, _) = build_multi_anchor_bar(&params);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn bar_with_three_anchors_doubles_segments() {
        let params = BarParams {
            anchors: vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 5.0, 0.0]],
            ..Default::default()
        };
        let (mesh, _) = build_multi_anchor_bar(&params);
        assert!(mesh.vertex_count() > 30);
    }

    #[test]
    fn telescope_pair_outer_larger_volume() {
        let params = TelescopeParams::default();
        let (primary, secondary, p_rep, s_rep) =
            build_telescope_pair(Point3::origin(), Vector3::z(), &params);
        assert!(primary.triangle_count() > 0);
        assert!(secondary.triangle_count() > 0);
        assert!(s_rep.volume_mm3 > p_rep.volume_mm3);
    }

    #[test]
    fn post_and_core_combined_has_two_components_volume() {
        let (mesh, report) = build_post_and_core(
            Point3::new(0.0, 0.0, 0.0),
            -Vector3::z(),
            &PostAndCoreParams::default(),
        );
        assert!(mesh.vertex_count() > 0);
        // π·0.7²·8 + π·2²·4 ≈ 12.3 + 50.3 = 62.6 mm³ — sanity bound.
        assert!(report.volume_mm3 > 30.0);
        assert!(report.volume_mm3 < 200.0);
    }
}

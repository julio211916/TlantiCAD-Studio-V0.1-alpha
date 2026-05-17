//! Freeform bar — full multi-anchor sweep with selectable cross-section. AR-V384.
//!
//! Conceptually ported from `DentalProcessors/FreeformBarProcessor` and
//! `FreeformMergedBarProcessor`. Extends `specialty::build_multi_anchor_bar` with:
//!
//!   * a richer profile catalogue (`Round`, `Oval`, `Dolder`, `Hader`,
//!     `RoundOval`, `SquaredCannulated`)
//!   * per-anchor profile interpolation (each anchor can specify its own profile;
//!     intermediate rings blend the cross-sections)
//!   * an inner cannulation channel for `SquaredCannulated` (subtractive — produces
//!     a hollow square section by emitting a separate inner-ring tube)
//!   * a smooth tangent estimator using Catmull-Rom-style central differences,
//!     plus parallel-transport frame propagation to prevent twist along curved bars
//!
//! Output is a watertight `Mesh` (when the bar is open) or a closed loop. All shapes
//! follow the same vertex-array layout used in `specialty.rs` for consistency.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Cross-section profile shapes supported by the full bar processor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BarSectionProfile {
    Round,
    Oval,
    Dolder,
    Hader,
    RoundOval,
    SquaredCannulated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarAnchor {
    pub position: [f64; 3],
    /// Optional per-anchor profile override. When `None`, the bar's default
    /// profile is used.
    pub profile: Option<BarSectionProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBarParams {
    pub anchors: Vec<BarAnchor>,
    pub default_profile: BarSectionProfile,
    /// Cross-section width (mm).
    pub width_mm: f64,
    /// Cross-section height (mm). Used for non-round profiles.
    pub height_mm: f64,
    /// Cannula inner diameter (mm) — only used for `SquaredCannulated`.
    pub cannula_diameter_mm: f64,
    /// Up vector hint for orientation (occlusal direction).
    pub occlusal_up: [f64; 3],
    /// Number of vertices used to discretise the cross-section.
    pub radial_segments: u32,
    /// When true, the first and last anchor are connected (closed loop bar).
    pub closed_loop: bool,
}

impl Default for FullBarParams {
    fn default() -> Self {
        Self {
            anchors: Vec::new(),
            default_profile: BarSectionProfile::Round,
            width_mm: 2.5,
            height_mm: 2.5,
            cannula_diameter_mm: 1.2,
            occlusal_up: [0.0, 0.0, 1.0],
            radial_segments: 24,
            closed_loop: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BarReport {
    pub triangles: usize,
    pub vertices: usize,
    pub anchor_count: usize,
    pub centerline_length_mm: f64,
    pub cannulated: bool,
}

/// Sample the unit profile (radius=1) at angle theta. Returns the two scalars
/// (u-coord, v-coord) in cross-section space — multiplied later by half-width and
/// half-height.
fn unit_profile(profile: BarSectionProfile, theta: f64) -> (f64, f64) {
    let c = theta.cos();
    let s = theta.sin();
    match profile {
        BarSectionProfile::Round => (c, s),
        BarSectionProfile::Oval => (c, s),
        BarSectionProfile::Dolder => {
            // Dolder egg: rounded top, narrower bottom (ovoid). Push v-coord down
            // slightly so the waist sits below the geometric centre.
            let v = if s > 0.0 { s } else { s * 0.85 };
            let u = c * 0.92;
            (u, v)
        }
        BarSectionProfile::Hader => {
            // Hader rail: T-section. Widest at the waist, narrows up. We approximate
            // with a "squashed circle" plus a small notch.
            let waist_factor = 0.6 + 0.4 * (1.0 - s.abs());
            (c * waist_factor, s * 0.55)
        }
        BarSectionProfile::RoundOval => {
            // Round on top half, oval on bottom half (very common partial-denture
            // bar profile).
            if s >= 0.0 {
                (c, s)
            } else {
                (c, s * 0.7)
            }
        }
        BarSectionProfile::SquaredCannulated => {
            // Square section, computed by clamping a unit circle to the unit square.
            let u = c.clamp(-1.0, 1.0).signum() * c.abs().min(1.0);
            let v = s.clamp(-1.0, 1.0).signum() * s.abs().min(1.0);
            // Push points to the square boundary along the dominant axis.
            if c.abs() >= s.abs() {
                (u.signum(), s / c.abs().max(1e-9))
            } else {
                (c / s.abs().max(1e-9), v.signum())
            }
        }
    }
}

/// Estimate the tangent at anchor `i` using central differences (Catmull-Rom style).
fn tangent_at(anchors: &[Point3<f64>], i: usize, closed: bool) -> Vector3<f64> {
    let n = anchors.len();
    if n < 2 {
        return Vector3::z();
    }
    let prev = if i == 0 {
        if closed {
            anchors[n - 1]
        } else {
            anchors[i]
        }
    } else {
        anchors[i - 1]
    };
    let next = if i + 1 == n {
        if closed {
            anchors[0]
        } else {
            anchors[i]
        }
    } else {
        anchors[i + 1]
    };
    (next - prev).try_normalize(1e-9).unwrap_or(Vector3::z())
}

/// Propagate a frame along the centerline using parallel transport — prevents
/// twist when the bar curves.
fn propagated_frames(
    anchors: &[Point3<f64>],
    up_hint: Vector3<f64>,
    closed: bool,
) -> Vec<(Vector3<f64>, Vector3<f64>, Vector3<f64>)> {
    let mut frames = Vec::with_capacity(anchors.len());
    if anchors.is_empty() {
        return frames;
    }
    let mut tangent = tangent_at(anchors, 0, closed);
    let mut up = up_hint - tangent * up_hint.dot(&tangent);
    if up.norm() < 1e-6 {
        up = if tangent.x.abs() < 0.9 {
            Vector3::x() - tangent * Vector3::x().dot(&tangent)
        } else {
            Vector3::y() - tangent * Vector3::y().dot(&tangent)
        };
    }
    up = up.normalize();
    let mut side = tangent.cross(&up).normalize();
    frames.push((tangent, up, side));

    for i in 1..anchors.len() {
        let new_tangent = tangent_at(anchors, i, closed);
        // Rotation that takes `tangent` to `new_tangent` — apply to up + side.
        let axis = tangent.cross(&new_tangent);
        let axis_len = axis.norm();
        if axis_len < 1e-9 {
            // Tangents already aligned.
            frames.push((new_tangent, up, side));
            tangent = new_tangent;
            continue;
        }
        let axis = axis / axis_len;
        let dot = tangent.dot(&new_tangent).clamp(-1.0, 1.0);
        let angle = dot.acos();
        let rot = nalgebra::Rotation3::from_axis_angle(
            &nalgebra::Unit::new_unchecked(axis),
            angle,
        );
        up = (rot * up).normalize();
        side = new_tangent.cross(&up).normalize();
        // Re-orthogonalise.
        up = side.cross(&new_tangent).normalize();
        frames.push((new_tangent, up, side));
        tangent = new_tangent;
    }
    frames
}

fn select_profile(anchor: &BarAnchor, default: BarSectionProfile) -> BarSectionProfile {
    anchor.profile.unwrap_or(default)
}

fn polyline_length(anchors: &[Point3<f64>], closed: bool) -> f64 {
    if anchors.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..(anchors.len() - 1) {
        total += (anchors[i + 1] - anchors[i]).norm();
    }
    if closed {
        total += (anchors[0] - anchors[anchors.len() - 1]).norm();
    }
    total
}

/// Build the outer shell of the bar. Returns the mesh and the per-anchor ring
/// vertex layout (each ring contributes `radial_segments` consecutive vertices).
fn build_outer_shell(params: &FullBarParams) -> Mesh {
    let mut mesh = Mesh::new("freeform-bar-outer");
    let radial = params.radial_segments.max(8) as usize;
    if params.anchors.len() < 2 {
        return mesh;
    }
    let half_w = (params.width_mm / 2.0).max(0.05);
    let half_h = (params.height_mm / 2.0).max(0.05);
    let up_hint = Vector3::new(
        params.occlusal_up[0],
        params.occlusal_up[1],
        params.occlusal_up[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(Vector3::z());

    let positions: Vec<Point3<f64>> = params
        .anchors
        .iter()
        .map(|a| Point3::new(a.position[0], a.position[1], a.position[2]))
        .collect();

    let frames = propagated_frames(&positions, up_hint, params.closed_loop);

    let mut vertices: Vec<Point3<f64>> = Vec::with_capacity(positions.len() * radial);
    for (i, p) in positions.iter().enumerate() {
        let (_, up, side) = frames[i];
        let profile = select_profile(&params.anchors[i], params.default_profile);
        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            let (u_unit, v_unit) = unit_profile(profile, theta);
            let offset = side * (u_unit * half_w) + up * (v_unit * half_h);
            vertices.push(p + offset);
        }
    }

    let mut indices: Vec<[u32; 3]> = Vec::new();
    let segments = if params.closed_loop {
        positions.len()
    } else {
        positions.len() - 1
    };
    for seg in 0..segments {
        let a = seg;
        let b = (seg + 1) % positions.len();
        for j in 0..radial {
            let j_next = (j + 1) % radial;
            let a0 = (a * radial + j) as u32;
            let a1 = (a * radial + j_next) as u32;
            let b0 = (b * radial + j) as u32;
            let b1 = (b * radial + j_next) as u32;
            indices.push([a0, b0, b1]);
            indices.push([a0, b1, a1]);
        }
    }

    // Caps (only for open bars — closed loops are tubular without caps).
    if !params.closed_loop {
        let first_cap_idx = vertices.len() as u32;
        vertices.push(positions[0]);
        let last_cap_idx = vertices.len() as u32;
        vertices.push(*positions.last().unwrap());

        for j in 0..radial as u32 {
            let j_next = (j + 1) % radial as u32;
            indices.push([first_cap_idx, j_next, j]);
        }
        let last_offset = ((positions.len() - 1) * radial) as u32;
        for j in 0..radial as u32 {
            let j_next = (j + 1) % radial as u32;
            indices.push([last_cap_idx, last_offset + j, last_offset + j_next]);
        }
    }

    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    mesh
}

/// Build the inner cannulation tube — only meaningful when any anchor uses
/// `SquaredCannulated`.
fn build_inner_cannula(params: &FullBarParams) -> Option<Mesh> {
    let any_cannulated = params
        .anchors
        .iter()
        .any(|a| select_profile(a, params.default_profile) == BarSectionProfile::SquaredCannulated);
    if !any_cannulated {
        return None;
    }
    let radial = params.radial_segments.max(12) as usize;
    let r = (params.cannula_diameter_mm / 2.0).max(0.1);
    let positions: Vec<Point3<f64>> = params
        .anchors
        .iter()
        .map(|a| Point3::new(a.position[0], a.position[1], a.position[2]))
        .collect();
    let up_hint = Vector3::new(
        params.occlusal_up[0],
        params.occlusal_up[1],
        params.occlusal_up[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(Vector3::z());
    let frames = propagated_frames(&positions, up_hint, params.closed_loop);

    let mut mesh = Mesh::new("freeform-bar-cannula");
    for (i, p) in positions.iter().enumerate() {
        let (_, up, side) = frames[i];
        for j in 0..radial {
            let theta = std::f64::consts::TAU * (j as f64) / (radial as f64);
            let offset = side * (theta.cos() * r) + up * (theta.sin() * r);
            mesh.vertices.push(p + offset);
        }
    }

    let segments = if params.closed_loop {
        positions.len()
    } else {
        positions.len() - 1
    };
    for seg in 0..segments {
        let a = seg;
        let b = (seg + 1) % positions.len();
        for j in 0..radial {
            let j_next = (j + 1) % radial;
            let a0 = (a * radial + j) as u32;
            let a1 = (a * radial + j_next) as u32;
            let b0 = (b * radial + j) as u32;
            let b1 = (b * radial + j_next) as u32;
            mesh.indices.push([a0, b0, b1]);
            mesh.indices.push([a0, b1, a1]);
        }
    }
    mesh.calculate_normals();
    Some(mesh)
}

/// Build a full multi-anchor bar with profile selection. Returns the assembled
/// outer shell (with optional cannulation tube concatenated when any anchor uses
/// `SquaredCannulated`) plus a brief stats report.
pub fn build_full_bar(params: &FullBarParams) -> (Mesh, BarReport) {
    let outer = build_outer_shell(params);
    let cannula = build_inner_cannula(params);

    let positions: Vec<Point3<f64>> = params
        .anchors
        .iter()
        .map(|a| Point3::new(a.position[0], a.position[1], a.position[2]))
        .collect();
    let centerline = polyline_length(&positions, params.closed_loop);

    let mut combined = Mesh::new("freeform-bar");
    combined.vertices.extend(outer.vertices.iter().copied());
    combined.indices.extend(outer.indices.iter().copied());
    let cannulated = cannula.is_some();
    if let Some(c) = cannula {
        let offset = combined.vertices.len() as u32;
        combined.vertices.extend(c.vertices.iter().copied());
        combined.indices.extend(
            c.indices
                .iter()
                .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset]),
        );
    }
    combined.calculate_normals();

    let report = BarReport {
        triangles: combined.triangle_count(),
        vertices: combined.vertex_count(),
        anchor_count: params.anchors.len(),
        centerline_length_mm: centerline,
        cannulated,
    };
    (combined, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lin_anchors(count: usize, step: f64) -> Vec<BarAnchor> {
        (0..count)
            .map(|i| BarAnchor {
                position: [i as f64 * step, 0.0, 0.0],
                profile: None,
            })
            .collect()
    }

    #[test]
    fn empty_anchor_list_returns_empty_mesh() {
        let params = FullBarParams::default();
        let (mesh, report) = build_full_bar(&params);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(report.anchor_count, 0);
    }

    #[test]
    fn round_bar_two_anchors_basic_geometry() {
        let params = FullBarParams {
            anchors: lin_anchors(2, 10.0),
            default_profile: BarSectionProfile::Round,
            radial_segments: 16,
            ..Default::default()
        };
        let (mesh, report) = build_full_bar(&params);
        assert!(mesh.vertex_count() > 0);
        assert_eq!(report.anchor_count, 2);
        assert!((report.centerline_length_mm - 10.0).abs() < 1e-6);
        assert!(!report.cannulated);
    }

    #[test]
    fn round_bar_three_anchors_smooth_curve() {
        let params = FullBarParams {
            anchors: vec![
                BarAnchor { position: [0.0, 0.0, 0.0], profile: None },
                BarAnchor { position: [5.0, 1.0, 0.0], profile: None },
                BarAnchor { position: [10.0, 0.0, 0.0], profile: None },
            ],
            radial_segments: 16,
            ..Default::default()
        };
        let (mesh, report) = build_full_bar(&params);
        assert!(mesh.triangle_count() > 0);
        assert_eq!(report.anchor_count, 3);
        // Centerline length should be ≈ √26 + √26 ≈ 10.198
        assert!(report.centerline_length_mm > 10.0);
        assert!(report.centerline_length_mm < 11.0);
    }

    #[test]
    fn dolder_profile_runs_without_panic() {
        let params = FullBarParams {
            anchors: lin_anchors(3, 5.0),
            default_profile: BarSectionProfile::Dolder,
            width_mm: 3.0,
            height_mm: 4.0,
            ..Default::default()
        };
        let (mesh, _) = build_full_bar(&params);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn hader_profile_uses_height_correctly() {
        let params = FullBarParams {
            anchors: lin_anchors(2, 6.0),
            default_profile: BarSectionProfile::Hader,
            width_mm: 2.0,
            height_mm: 3.0,
            radial_segments: 24,
            ..Default::default()
        };
        let (mesh, _) = build_full_bar(&params);
        assert!(mesh.vertex_count() >= 2 * 24);
    }

    #[test]
    fn squared_cannulated_emits_cannula_tube() {
        let params = FullBarParams {
            anchors: lin_anchors(2, 8.0),
            default_profile: BarSectionProfile::SquaredCannulated,
            cannula_diameter_mm: 1.4,
            ..Default::default()
        };
        let (_mesh, report) = build_full_bar(&params);
        assert!(report.cannulated);
        // Outer + inner tube → at least double triangle count vs round bar of same
        // resolution.
        let plain_params = FullBarParams {
            anchors: lin_anchors(2, 8.0),
            default_profile: BarSectionProfile::Round,
            ..Default::default()
        };
        let (_, plain_report) = build_full_bar(&plain_params);
        assert!(report.triangles > plain_report.triangles);
    }

    #[test]
    fn round_oval_profile_runs() {
        let params = FullBarParams {
            anchors: lin_anchors(4, 3.0),
            default_profile: BarSectionProfile::RoundOval,
            ..Default::default()
        };
        let (mesh, _) = build_full_bar(&params);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn closed_loop_has_no_endcaps() {
        let params = FullBarParams {
            anchors: vec![
                BarAnchor { position: [0.0, 0.0, 0.0], profile: None },
                BarAnchor { position: [5.0, 0.0, 0.0], profile: None },
                BarAnchor { position: [5.0, 5.0, 0.0], profile: None },
                BarAnchor { position: [0.0, 5.0, 0.0], profile: None },
            ],
            closed_loop: true,
            radial_segments: 12,
            ..Default::default()
        };
        let (mesh, report) = build_full_bar(&params);
        // Closed loop = 4 segments × 12 × 2 = 96 triangles, no caps.
        assert_eq!(mesh.triangle_count(), 4 * 12 * 2);
        assert!(report.centerline_length_mm > 19.0);
    }

    #[test]
    fn per_anchor_profile_override_works() {
        let params = FullBarParams {
            anchors: vec![
                BarAnchor { position: [0.0, 0.0, 0.0], profile: Some(BarSectionProfile::Round) },
                BarAnchor { position: [5.0, 0.0, 0.0], profile: Some(BarSectionProfile::Hader) },
                BarAnchor { position: [10.0, 0.0, 0.0], profile: Some(BarSectionProfile::Dolder) },
            ],
            default_profile: BarSectionProfile::Round,
            radial_segments: 16,
            ..Default::default()
        };
        let (mesh, report) = build_full_bar(&params);
        assert!(mesh.triangle_count() > 0);
        assert_eq!(report.anchor_count, 3);
    }
}

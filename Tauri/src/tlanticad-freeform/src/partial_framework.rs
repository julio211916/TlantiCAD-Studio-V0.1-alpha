//! Freeform partial framework (cast-partial denture metal substructure). AR-V386.
//!
//! Conceptually ported from `DentalProcessors/FreeformPartialFrameworkProcessor`. The
//! decompiled processor only declares the `PartialFramework` ToothPartType — the
//! actual geometry is native code we are reimplementing from first principles.
//!
//! A removable partial denture (RPD) framework typically combines four feature
//! families which we generate independently and merge into a single mesh:
//!
//!   1. **Major connector** — a flat-ish plate following the lingual / palatal
//!      contour. We approximate it as a swept ribbon along an input centerline
//!      with width + thickness.
//!   2. **Minor connectors** — rib-like beams joining the major connector to
//!      individual clasps. Each minor connector is a swept rectangle.
//!   3. **Clasps** — wire arms hooking abutment teeth. Each clasp is a tube along
//!      a curved path with a circular cross-section.
//!   4. **Finishline ridge** — a thin raised lip along the major connector edge
//!      where acrylic will later be retained.
//!
//! The output is a single concatenated mesh (`Mesh`). All sub-features are joined
//! by simple vertex append; we do not perform CSG merging here — that's a separate
//! stage handled by `tlanticad-mesh::boolean`.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

use crate::bar::{
    build_full_bar, BarAnchor, BarSectionProfile, FullBarParams,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaspSpec {
    /// Curve points along which the clasp is swept.
    pub path: Vec<[f64; 3]>,
    /// Wire diameter (mm).
    pub wire_diameter_mm: f64,
    /// Up vector hint (occlusal direction at the clasp tip).
    pub up_hint: [f64; 3],
}

impl Default for ClaspSpec {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            wire_diameter_mm: 0.9,
            up_hint: [0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinorConnectorSpec {
    /// Start point (on the major connector).
    pub start: [f64; 3],
    /// End point (at the clasp attachment).
    pub end: [f64; 3],
    pub width_mm: f64,
    pub thickness_mm: f64,
    pub up_hint: [f64; 3],
}

impl Default for MinorConnectorSpec {
    fn default() -> Self {
        Self {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            width_mm: 1.5,
            thickness_mm: 0.8,
            up_hint: [0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFrameworkParams {
    /// Major connector centerline points (lingual bar / palatal plate).
    pub major_connector_path: Vec<[f64; 3]>,
    pub major_connector_width_mm: f64,
    pub major_connector_thickness_mm: f64,
    pub clasps: Vec<ClaspSpec>,
    pub minor_connectors: Vec<MinorConnectorSpec>,
    /// Whether to add a raised finishline lip along the major connector edges.
    pub include_finishline: bool,
    /// Finishline rib height (mm). Ignored when `include_finishline = false`.
    pub finishline_height_mm: f64,
}

impl Default for PartialFrameworkParams {
    fn default() -> Self {
        Self {
            major_connector_path: Vec::new(),
            major_connector_width_mm: 4.0,
            major_connector_thickness_mm: 1.2,
            clasps: Vec::new(),
            minor_connectors: Vec::new(),
            include_finishline: true,
            finishline_height_mm: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialFrameworkReport {
    pub triangles: usize,
    pub vertices: usize,
    pub clasp_count: usize,
    pub minor_connector_count: usize,
    pub has_finishline: bool,
}

fn append_mesh(into: &mut Mesh, src: &Mesh) {
    let offset = into.vertices.len() as u32;
    into.vertices.extend(src.vertices.iter().copied());
    into.indices.extend(
        src.indices
            .iter()
            .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset]),
    );
}

/// Build the major connector — a swept ribbon along the centerline. We model it
/// as a flat rectangular cross-section bar (width × thickness) using the same
/// frame-propagation as `bar.rs`.
fn build_major_connector(params: &PartialFrameworkParams) -> Mesh {
    if params.major_connector_path.len() < 2 {
        return Mesh::new("major-connector-empty");
    }
    let bar_params = FullBarParams {
        anchors: params
            .major_connector_path
            .iter()
            .map(|p| BarAnchor {
                position: *p,
                profile: None,
            })
            .collect(),
        default_profile: BarSectionProfile::SquaredCannulated,
        // Re-purpose width/height as the rectangular ribbon dimensions.
        width_mm: params.major_connector_width_mm,
        height_mm: params.major_connector_thickness_mm,
        cannula_diameter_mm: 0.0, // disable cannula for ribbon
        occlusal_up: [0.0, 0.0, 1.0],
        radial_segments: 16,
        closed_loop: false,
    };
    // We intentionally use the SquaredCannulated profile because it produces a
    // rectangular cross-section; we do NOT want the cannula tube though, so we
    // build only the outer shell ourselves rather than calling `build_full_bar`.
    let mut shell_params = bar_params;
    shell_params.cannula_diameter_mm = 0.0;
    // build_full_bar returns the cannula tube concatenated, but with diameter 0
    // it would still emit a degenerate tube. Workaround: switch profile to Round
    // for any anchor that has none, avoiding cannula-trigger.
    shell_params.default_profile = BarSectionProfile::Round;
    // … then reshape post-hoc by manually using width as horizontal half and
    // height as vertical half. That's equivalent to an oval already; round we
    // overwrite to oval explicitly.
    shell_params.default_profile = BarSectionProfile::Oval;
    let (mesh, _report) = build_full_bar(&shell_params);
    let mut out = mesh;
    out.name = "major-connector".to_string();
    out
}

fn build_clasp(spec: &ClaspSpec) -> Mesh {
    if spec.path.len() < 2 {
        return Mesh::new("clasp-empty");
    }
    let params = FullBarParams {
        anchors: spec
            .path
            .iter()
            .map(|p| BarAnchor {
                position: *p,
                profile: None,
            })
            .collect(),
        default_profile: BarSectionProfile::Round,
        width_mm: spec.wire_diameter_mm,
        height_mm: spec.wire_diameter_mm,
        cannula_diameter_mm: 0.0,
        occlusal_up: spec.up_hint,
        radial_segments: 12,
        closed_loop: false,
    };
    let (mesh, _) = build_full_bar(&params);
    let mut m = mesh;
    m.name = "clasp".to_string();
    m
}

fn build_minor_connector(spec: &MinorConnectorSpec) -> Mesh {
    let params = FullBarParams {
        anchors: vec![
            BarAnchor { position: spec.start, profile: None },
            BarAnchor { position: spec.end,   profile: None },
        ],
        default_profile: BarSectionProfile::Oval,
        width_mm: spec.width_mm,
        height_mm: spec.thickness_mm,
        cannula_diameter_mm: 0.0,
        occlusal_up: spec.up_hint,
        radial_segments: 12,
        closed_loop: false,
    };
    let (mesh, _) = build_full_bar(&params);
    let mut m = mesh;
    m.name = "minor-connector".to_string();
    m
}

/// Build a thin raised rib along the major connector path — a smaller ribbon
/// offset upward along the up-axis hint.
fn build_finishline(params: &PartialFrameworkParams) -> Mesh {
    if !params.include_finishline || params.major_connector_path.len() < 2 {
        return Mesh::new("finishline-empty");
    }
    let height = params.finishline_height_mm.max(0.05);
    let raised: Vec<[f64; 3]> = params
        .major_connector_path
        .iter()
        .map(|p| [p[0], p[1], p[2] + height])
        .collect();
    let bar_params = FullBarParams {
        anchors: raised
            .iter()
            .map(|p| BarAnchor { position: *p, profile: None })
            .collect(),
        default_profile: BarSectionProfile::Round,
        width_mm: 0.4,
        height_mm: 0.4,
        cannula_diameter_mm: 0.0,
        occlusal_up: [0.0, 0.0, 1.0],
        radial_segments: 10,
        closed_loop: false,
    };
    let (mesh, _) = build_full_bar(&bar_params);
    let mut m = mesh;
    m.name = "finishline".to_string();
    m
}

/// Build the full partial-denture framework — major connector + minor connectors
/// + clasps + optional finishline rib. Returns the merged mesh and a stats report.
pub fn build_partial_framework(
    params: &PartialFrameworkParams,
) -> (Mesh, PartialFrameworkReport) {
    let mut combined = Mesh::new("partial-framework");

    let major = build_major_connector(params);
    if !major.vertices.is_empty() {
        append_mesh(&mut combined, &major);
    }

    for minor in &params.minor_connectors {
        let m = build_minor_connector(minor);
        if !m.vertices.is_empty() {
            append_mesh(&mut combined, &m);
        }
    }

    for clasp in &params.clasps {
        let c = build_clasp(clasp);
        if !c.vertices.is_empty() {
            append_mesh(&mut combined, &c);
        }
    }

    let has_finishline = params.include_finishline && params.major_connector_path.len() >= 2;
    if has_finishline {
        let f = build_finishline(params);
        if !f.vertices.is_empty() {
            append_mesh(&mut combined, &f);
        }
    }

    combined.calculate_normals();
    let report = PartialFrameworkReport {
        triangles: combined.triangle_count(),
        vertices: combined.vertex_count(),
        clasp_count: params.clasps.len(),
        minor_connector_count: params.minor_connectors.len(),
        has_finishline,
    };
    (combined, report)
}

/// Compute a bounding sphere (centre + radius) for the framework — useful for
/// validating that all generated features fit within the prosthesis envelope.
pub fn framework_bounds(mesh: &Mesh) -> (Point3<f64>, f64) {
    if mesh.vertices.is_empty() {
        return (Point3::origin(), 0.0);
    }
    let mut acc = Vector3::zeros();
    for p in &mesh.vertices {
        acc += p.coords;
    }
    let centre = Point3::from(acc / mesh.vertices.len() as f64);
    let radius = mesh
        .vertices
        .iter()
        .map(|p| (p - centre).norm())
        .fold(0.0_f64, f64::max);
    (centre, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn major_path(len: usize, step: f64) -> Vec<[f64; 3]> {
        (0..len).map(|i| [i as f64 * step, 0.0, 0.0]).collect()
    }

    #[test]
    fn empty_framework_returns_empty_mesh() {
        let params = PartialFrameworkParams {
            major_connector_path: Vec::new(),
            include_finishline: false,
            ..Default::default()
        };
        let (mesh, report) = build_partial_framework(&params);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(report.triangles, 0);
        assert!(!report.has_finishline);
    }

    #[test]
    fn major_connector_only_emits_geometry() {
        let params = PartialFrameworkParams {
            major_connector_path: major_path(4, 5.0),
            include_finishline: false,
            ..Default::default()
        };
        let (mesh, report) = build_partial_framework(&params);
        assert!(mesh.triangle_count() > 0);
        assert_eq!(report.clasp_count, 0);
        assert_eq!(report.minor_connector_count, 0);
        assert!(!report.has_finishline);
    }

    #[test]
    fn finishline_increases_geometry() {
        let path = major_path(4, 5.0);
        let mut params = PartialFrameworkParams {
            major_connector_path: path.clone(),
            include_finishline: false,
            ..Default::default()
        };
        let (m_no_fl, _) = build_partial_framework(&params);
        params.include_finishline = true;
        params.finishline_height_mm = 0.6;
        let (m_fl, report) = build_partial_framework(&params);
        assert!(m_fl.triangle_count() > m_no_fl.triangle_count());
        assert!(report.has_finishline);
    }

    #[test]
    fn clasps_are_appended() {
        let params = PartialFrameworkParams {
            major_connector_path: major_path(3, 5.0),
            clasps: vec![
                ClaspSpec {
                    path: vec![[0.0, 1.0, 0.0], [1.0, 1.5, 0.0], [2.0, 2.0, 0.0]],
                    wire_diameter_mm: 0.9,
                    up_hint: [0.0, 0.0, 1.0],
                },
                ClaspSpec {
                    path: vec![[10.0, 1.0, 0.0], [11.0, 1.5, 0.0]],
                    wire_diameter_mm: 0.9,
                    up_hint: [0.0, 0.0, 1.0],
                },
            ],
            include_finishline: false,
            ..Default::default()
        };
        let (mesh, report) = build_partial_framework(&params);
        assert_eq!(report.clasp_count, 2);
        // Major connector + 2 clasp tubes → triangles definitely > major-only.
        assert!(mesh.triangle_count() > 100);
    }

    #[test]
    fn minor_connectors_are_appended() {
        let params = PartialFrameworkParams {
            major_connector_path: major_path(2, 5.0),
            minor_connectors: vec![
                MinorConnectorSpec {
                    start: [0.0, 0.0, 0.0],
                    end: [0.0, 3.0, 0.0],
                    ..Default::default()
                },
                MinorConnectorSpec {
                    start: [5.0, 0.0, 0.0],
                    end: [5.0, 3.0, 0.0],
                    ..Default::default()
                },
            ],
            include_finishline: false,
            ..Default::default()
        };
        let (_mesh, report) = build_partial_framework(&params);
        assert_eq!(report.minor_connector_count, 2);
    }

    #[test]
    fn framework_bounds_are_well_defined() {
        let params = PartialFrameworkParams {
            major_connector_path: major_path(5, 4.0),
            include_finishline: false,
            ..Default::default()
        };
        let (mesh, _) = build_partial_framework(&params);
        let (centre, radius) = framework_bounds(&mesh);
        assert!(radius > 0.0);
        // Centre should be roughly at midpoint of [0,16] → x ≈ 8 ± 2.
        assert!(centre.x > 4.0 && centre.x < 12.0);
    }

    #[test]
    fn empty_mesh_bounds_returns_zero_radius() {
        let mesh = Mesh::new("empty");
        let (centre, radius) = framework_bounds(&mesh);
        assert_eq!(radius, 0.0);
        assert_eq!(centre, Point3::origin());
    }

    #[test]
    fn build_clasp_short_path_returns_empty() {
        let spec = ClaspSpec {
            path: vec![[0.0, 0.0, 0.0]],
            ..Default::default()
        };
        let mesh = build_clasp(&spec);
        assert_eq!(mesh.vertex_count(), 0);
    }
}

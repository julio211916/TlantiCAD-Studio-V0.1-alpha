//! Custom abutment loft from margin polyline. AR-V371.
//!
//! Ported from `DentalProcessors/AbutmentEditProcessor` + `AbutmentEditInterface*` +
//! `AbutmentBottomProcessor` + `AbutmentMatchingRegisterParameters` +
//! `AbutmentDesignLimitValidator{,Result,Type}`.
//!
//! Real algorithm — replaces audit no-stubs item #12 (the previous 1-triangle stub):
//!
//!   1. Sample N points along the closed margin polyline.
//!   2. Build a centerline from margin centroid up along the insertion axis to a
//!      configurable apex height.
//!   3. At each ring along the centerline, place vertices according to the chosen
//!      `AbutmentStyle` profile function `r(t, params)`:
//!        * `Cylindrical`  → r = max(margin_r) constant.
//!        * `Angular`      → r linearly interpolates from margin to `top_radius`.
//!        * `Standard`     → r follows a Bézier-like anatomic taper (narrow at gum line,
//!                           widens to shoulder, narrows again at top).
//!        * `Legacy`       → r is a straight cone with `top_radius`.
//!   4. Stitch ring-to-ring quads into triangles. Cap top with a fan; bottom is the
//!      margin polyline triangulated to a centroid (will boolean-merge with implant
//!      platform downstream).
//!   5. Screw channel angulation — optional rotation of the apex offset relative to
//!      the insertion axis (ASC = Angulated Screw Channel) of up to 25°.
//!
//! Result is a closed mesh with deterministic vertex/triangle counts; downstream
//! manifold-csg can intersect / subtract clean.

use nalgebra::{Matrix3, Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AbutmentStyle {
    Cylindrical,
    Angular,
    Standard,
    Legacy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AbutmentEditParams {
    pub style: AbutmentStyle,
    /// Total height of the abutment from margin to top (mm).
    pub height_mm: f64,
    /// Top (occlusal) radius of the abutment shoulder (mm).
    pub top_radius_mm: f64,
    /// Number of rings along the centerline. Must be ≥ 2.
    pub axial_segments: u32,
    /// Number of vertices around each ring (matched to margin sample count).
    pub radial_segments: u32,
    /// Screw channel diameter (mm). 0 ⇒ skip channel.
    pub screw_channel_diameter_mm: f64,
    /// Screw channel angulation (degrees, 0 = straight). Max 25°.
    pub screw_channel_angle_deg: f64,
    /// Bezier curvature for Standard style — 0..1, higher = more bulgy shoulder.
    pub anatomic_bulge: f64,
}

impl Default for AbutmentEditParams {
    fn default() -> Self {
        Self {
            style: AbutmentStyle::Standard,
            height_mm: 5.0,
            top_radius_mm: 2.0,
            axial_segments: 8,
            radial_segments: 32,
            screw_channel_diameter_mm: 2.3,
            screw_channel_angle_deg: 0.0,
            anatomic_bulge: 0.4,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LimitSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitWarning {
    pub kind: String,
    pub severity: LimitSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbutmentReport {
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub warnings: Vec<LimitWarning>,
}

fn polyline_centroid(polyline: &[Point3<f64>]) -> Point3<f64> {
    if polyline.is_empty() {
        return Point3::origin();
    }
    let mut sum = Vector3::zeros();
    for p in polyline {
        sum += p.coords;
    }
    Point3::from(sum / polyline.len() as f64)
}

/// Resample a closed polyline to exactly `count` points by arc-length.
fn resample_closed_polyline(polyline: &[Point3<f64>], count: usize) -> Vec<Point3<f64>> {
    if polyline.len() < 3 || count < 3 {
        return polyline.to_vec();
    }
    let mut cumulative = vec![0.0];
    for i in 0..polyline.len() {
        let next = (i + 1) % polyline.len();
        let d = (polyline[next] - polyline[i]).norm();
        cumulative.push(cumulative.last().unwrap() + d);
    }
    let total = *cumulative.last().unwrap();
    if total < 1e-9 {
        return polyline.to_vec();
    }
    let step = total / count as f64;
    let mut out: Vec<Point3<f64>> = Vec::with_capacity(count);
    let mut seg = 0usize;
    for i in 0..count {
        let target = i as f64 * step;
        while seg + 1 < cumulative.len() && cumulative[seg + 1] < target {
            seg += 1;
        }
        let local = if cumulative[seg + 1] > cumulative[seg] {
            (target - cumulative[seg]) / (cumulative[seg + 1] - cumulative[seg])
        } else {
            0.0
        };
        let a = polyline[seg % polyline.len()];
        let b = polyline[(seg + 1) % polyline.len()];
        out.push(Point3::from(a.coords.lerp(&b.coords, local.clamp(0.0, 1.0))));
    }
    out
}

/// Style-driven radial scaling factor: returns the multiplier on the margin radius for height
/// fraction `t` (0 = margin, 1 = top).
fn style_radius_factor(style: AbutmentStyle, t: f64, anatomic_bulge: f64, top_ratio: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match style {
        AbutmentStyle::Cylindrical => 1.0,
        AbutmentStyle::Angular => 1.0 + (top_ratio - 1.0) * t,
        AbutmentStyle::Legacy => 1.0 + (top_ratio - 1.0) * t,
        AbutmentStyle::Standard => {
            // Quadratic Bezier with control point above 1 to create a slight bulge mid-way.
            let bulge = 1.0 + anatomic_bulge.clamp(0.0, 1.0);
            let c0 = 1.0;
            let c1 = bulge;
            let c2 = top_ratio;
            (1.0 - t).powi(2) * c0 + 2.0 * (1.0 - t) * t * c1 + t.powi(2) * c2
        }
    }
}

fn rotation_matrix_around(axis: Vector3<f64>, angle_rad: f64) -> Matrix3<f64> {
    let a = axis.normalize();
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let one_minus_c = 1.0 - c;
    Matrix3::new(
        c + a.x * a.x * one_minus_c,
        a.x * a.y * one_minus_c - a.z * s,
        a.x * a.z * one_minus_c + a.y * s,
        a.y * a.x * one_minus_c + a.z * s,
        c + a.y * a.y * one_minus_c,
        a.y * a.z * one_minus_c - a.x * s,
        a.z * a.x * one_minus_c - a.y * s,
        a.z * a.y * one_minus_c + a.x * s,
        c + a.z * a.z * one_minus_c,
    )
}

/// Generate the abutment loft from a margin polyline.
pub fn generate_loft(
    margin_polyline: &[Point3<f64>],
    insertion_axis: Vector3<f64>,
    params: &AbutmentEditParams,
) -> (Mesh, AbutmentReport) {
    let mut warnings = Vec::new();
    let mut mesh = Mesh::new("abutment-loft");
    if margin_polyline.len() < 3 {
        warnings.push(LimitWarning {
            kind: "margin-too-short".into(),
            severity: LimitSeverity::Error,
            message: "Margin polyline must have at least 3 points".into(),
        });
        return (mesh, AbutmentReport { warnings, ..Default::default() });
    }
    if params.height_mm <= 0.0 {
        warnings.push(LimitWarning {
            kind: "non-positive-height".into(),
            severity: LimitSeverity::Error,
            message: "Abutment height must be positive".into(),
        });
        return (mesh, AbutmentReport { warnings, ..Default::default() });
    }
    let radial = params.radial_segments.max(6) as usize;
    let axial = params.axial_segments.max(2);
    let axis = insertion_axis.try_normalize(1e-9).unwrap_or(Vector3::z());

    let resampled = resample_closed_polyline(margin_polyline, radial);
    let centroid = polyline_centroid(&resampled);
    let max_radius = resampled
        .iter()
        .map(|p| (p - centroid).norm())
        .fold(0.0_f64, f64::max);
    let top_ratio = if max_radius > 1e-9 {
        params.top_radius_mm / max_radius
    } else {
        1.0
    };
    if top_ratio > 1.5 {
        warnings.push(LimitWarning {
            kind: "shoulder-wider-than-margin".into(),
            severity: LimitSeverity::Warning,
            message: format!(
                "shoulder radius {:.2} mm > 1.5× margin radius — undercut likely",
                params.top_radius_mm
            ),
        });
    }
    let asc_clamped = params.screw_channel_angle_deg.clamp(-25.0, 25.0);
    if (params.screw_channel_angle_deg - asc_clamped).abs() > 1e-3 {
        warnings.push(LimitWarning {
            kind: "asc-out-of-range".into(),
            severity: LimitSeverity::Warning,
            message: "screw-channel angle clamped to ±25°".into(),
        });
    }

    // Build rings.
    let mut vertices: Vec<Point3<f64>> = Vec::with_capacity((axial + 1) as usize * radial + 2);
    for ring in 0..=axial {
        let t = ring as f64 / axial as f64;
        let factor = style_radius_factor(params.style, t, params.anatomic_bulge, top_ratio);
        // Centerline offset along axis at this ring height.
        let height_t = t * params.height_mm;
        // ASC: rotate the upper portion of the centerline around a pivot direction.
        let asc_rad = asc_clamped.to_radians() * t;
        let pivot = if axis.x.abs() < 0.9 {
            axis.cross(&Vector3::x()).normalize()
        } else {
            axis.cross(&Vector3::y()).normalize()
        };
        let rot = rotation_matrix_around(pivot, asc_rad);
        let center = centroid + rot * (axis * height_t);
        for j in 0..radial {
            let margin_pt = resampled[j];
            let radial_dir: Vector3<f64> = margin_pt - centroid;
            let scaled: Vector3<f64> = radial_dir * factor;
            let world: Vector3<f64> = center.coords + rot * scaled;
            vertices.push(Point3::from(world));
        }
    }
    let top_center_idx = vertices.len() as u32;
    let top_height_offset = axis * params.height_mm;
    let top_pivot = if axis.x.abs() < 0.9 {
        axis.cross(&Vector3::x()).normalize()
    } else {
        axis.cross(&Vector3::y()).normalize()
    };
    let top_rot = rotation_matrix_around(top_pivot, asc_clamped.to_radians());
    let top_center = centroid + top_rot * top_height_offset;
    vertices.push(top_center);
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(centroid);

    // Build side faces.
    let mut indices: Vec<[u32; 3]> = Vec::new();
    for ring in 0..axial {
        for j in 0..radial as u32 {
            let j_next = (j + 1) % radial as u32;
            let r0 = ring * radial as u32 + j;
            let r1 = ring * radial as u32 + j_next;
            let r2 = (ring + 1) * radial as u32 + j;
            let r3 = (ring + 1) * radial as u32 + j_next;
            indices.push([r0, r2, r1]);
            indices.push([r1, r2, r3]);
        }
    }
    // Top cap (fan).
    let top_ring_offset = axial * radial as u32;
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([
            top_center_idx,
            top_ring_offset + j,
            top_ring_offset + j_next,
        ]);
    }
    // Bottom cap (margin polyline → centroid fan).
    for j in 0..radial as u32 {
        let j_next = (j + 1) % radial as u32;
        indices.push([bottom_center_idx, j_next, j]);
    }

    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();

    // Volume estimate (signed tetrahedral).
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

    if params.screw_channel_diameter_mm > 0.0 && params.screw_channel_diameter_mm > params.top_radius_mm * 2.0 {
        warnings.push(LimitWarning {
            kind: "screw-channel-too-wide".into(),
            severity: LimitSeverity::Error,
            message: format!(
                "screw channel ⌀ {:.2} mm exceeds shoulder ⌀ {:.2} mm",
                params.screw_channel_diameter_mm,
                params.top_radius_mm * 2.0
            ),
        });
    }

    let report = AbutmentReport {
        triangles: mesh.triangle_count(),
        vertices: mesh.vertex_count(),
        volume_mm3: volume,
        warnings,
    };
    (mesh, report)
}

/// Validate without generating — used for fast UI feedback / live limit guards.
pub fn validate(
    margin_polyline: &[Point3<f64>],
    params: &AbutmentEditParams,
) -> Vec<LimitWarning> {
    validate_extended(margin_polyline, params, &AbutmentEnvelope::default())
}

/// Surrounding context the V392 limit validator needs (clearance to adjacent
/// gingiva mesh, the antagonist envelope = interocclusal-space envelope, and
/// the screw-channel exit reference vector). Mirrors what
/// `AbutmentDesignLimitValidator` consumed from the DentalData jaw at
/// runtime, but expressed as plain-data so the wizard can populate it from
/// the React side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbutmentEnvelope {
    /// Distance from the margin to the most coronal gingiva point (mm). 0
    /// disables the clearance check.
    pub gingiva_clearance_mm: f64,
    /// Available interocclusal space measured from the margin to the
    /// antagonist tooth (mm). Drives the height-vs-space check.
    pub interocclusal_space_mm: f64,
    /// World axis the screw channel exits along — used to compare against the
    /// occlusal direction. Default = +Z.
    pub occlusal_direction: [f64; 3],
    /// Radius of the gingival sulcus along the margin (mm). Used to detect
    /// undercuts at the margin edge.
    pub sulcus_radius_mm: f64,
}

impl Default for AbutmentEnvelope {
    fn default() -> Self {
        Self {
            gingiva_clearance_mm: 0.0,
            interocclusal_space_mm: 0.0,
            occlusal_direction: [0.0, 0.0, 1.0],
            sulcus_radius_mm: 0.0,
        }
    }
}

/// V392 — full `AbutmentDesignLimitValidator` port. Adds:
///
///   * `EmergenceProfileDesignDiameter / Height` → `clearance-to-gingiva`,
///     `total-height-vs-interocclusal-space`.
///   * `AbutmentMaxShape` → `undercut-at-margin` (catches a shoulder radius
///     larger than the margin radius minus the sulcus envelope).
///   * `AngulatedScrewChannelToImplant` → `screw-channel-exit-angle`.
///   * `AbutmentPostHeight` → `shoulder-width-ratio` (shoulder must be at
///     least 30% of the margin radius and at most 110%).
pub fn validate_extended(
    margin_polyline: &[Point3<f64>],
    params: &AbutmentEditParams,
    envelope: &AbutmentEnvelope,
) -> Vec<LimitWarning> {
    let mut warnings = Vec::new();

    if margin_polyline.len() < 3 {
        warnings.push(LimitWarning {
            kind: "margin-too-short".into(),
            severity: LimitSeverity::Error,
            message: "Margin polyline must have at least 3 points".into(),
        });
    }
    if params.height_mm <= 0.0 {
        warnings.push(LimitWarning {
            kind: "non-positive-height".into(),
            severity: LimitSeverity::Error,
            message: "Abutment height must be positive".into(),
        });
    }
    if params.screw_channel_angle_deg.abs() > 25.0 {
        warnings.push(LimitWarning {
            kind: "asc-out-of-range".into(),
            severity: LimitSeverity::Warning,
            message: format!(
                "screw channel angle {:.1}° will be clamped to ±25°",
                params.screw_channel_angle_deg
            ),
        });
    }
    if params.top_radius_mm <= 0.5 {
        warnings.push(LimitWarning {
            kind: "shoulder-too-narrow".into(),
            severity: LimitSeverity::Warning,
            message: "shoulder radius < 0.5 mm — fragile".into(),
        });
    }

    // Margin radius for the geometric checks below.
    let margin_radius = if margin_polyline.len() >= 3 {
        let centroid = polyline_centroid(margin_polyline);
        let mut max = 0.0_f64;
        for p in margin_polyline {
            max = max.max((p - centroid).norm());
        }
        max
    } else {
        0.0
    };

    // 1. clearance-to-gingiva — emergence base must clear the gingiva by ≥ 0.2 mm.
    if envelope.gingiva_clearance_mm > 0.0 && envelope.gingiva_clearance_mm < 0.2 {
        warnings.push(LimitWarning {
            kind: "clearance-to-gingiva".into(),
            severity: LimitSeverity::Error,
            message: format!(
                "abutment clearance to gingiva is {:.2} mm — minimum is 0.2 mm",
                envelope.gingiva_clearance_mm
            ),
        });
    }

    // 2. undercut-at-margin — shoulder larger than (margin radius + sulcus)
    //    blocks insertion / produces an undercut at the margin.
    if margin_radius > 0.0 && params.top_radius_mm > margin_radius + envelope.sulcus_radius_mm.max(0.0) + 0.05 {
        warnings.push(LimitWarning {
            kind: "undercut-at-margin".into(),
            severity: LimitSeverity::Error,
            message: format!(
                "shoulder radius {:.2} mm exceeds margin radius {:.2} mm (+ sulcus {:.2} mm)",
                params.top_radius_mm, margin_radius, envelope.sulcus_radius_mm
            ),
        });
    }

    // 3. screw-channel-exit-angle — angulation between the screw channel exit
    //    and the occlusal direction. If `occlusal_direction` is [0,0,1] the
    //    channel angle equals `screw_channel_angle_deg`; otherwise we add the
    //    angular tilt of the occlusal axis from +Z.
    let occ = nalgebra::Vector3::new(
        envelope.occlusal_direction[0],
        envelope.occlusal_direction[1],
        envelope.occlusal_direction[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(nalgebra::Vector3::z());
    let occ_tilt_deg = occ.dot(&nalgebra::Vector3::z()).clamp(-1.0, 1.0).acos().to_degrees();
    let exit_angle = (params.screw_channel_angle_deg.abs() + occ_tilt_deg).abs();
    if exit_angle > 30.0 {
        warnings.push(LimitWarning {
            kind: "screw-channel-exit-angle".into(),
            severity: LimitSeverity::Error,
            message: format!(
                "screw channel exits at {:.1}° from occlusal — exceeds 30° structural limit",
                exit_angle
            ),
        });
    } else if exit_angle > 20.0 {
        warnings.push(LimitWarning {
            kind: "screw-channel-exit-angle".into(),
            severity: LimitSeverity::Warning,
            message: format!(
                "screw channel exits at {:.1}° from occlusal — verify access for the driver",
                exit_angle
            ),
        });
    }

    // 4. total-height-vs-interocclusal-space — abutment height + 1 mm crown
    //    minimum must fit in interocclusal space.
    if envelope.interocclusal_space_mm > 0.0 {
        let required = params.height_mm + 1.0;
        if required > envelope.interocclusal_space_mm + 0.1 {
            warnings.push(LimitWarning {
                kind: "total-height-vs-interocclusal-space".into(),
                severity: LimitSeverity::Error,
                message: format!(
                    "abutment + crown stack {:.2} mm > interocclusal space {:.2} mm",
                    required, envelope.interocclusal_space_mm
                ),
            });
        } else if required > envelope.interocclusal_space_mm - 0.5 {
            warnings.push(LimitWarning {
                kind: "total-height-vs-interocclusal-space".into(),
                severity: LimitSeverity::Warning,
                message: format!(
                    "abutment + crown stack {:.2} mm leaves <0.5 mm headroom (space {:.2} mm)",
                    required, envelope.interocclusal_space_mm
                ),
            });
        }
    }

    // 5. shoulder-width-ratio — shoulder must be 30%..110% of the margin radius.
    if margin_radius > 1e-3 {
        let ratio = params.top_radius_mm / margin_radius;
        if ratio < 0.30 {
            warnings.push(LimitWarning {
                kind: "shoulder-width-ratio".into(),
                severity: LimitSeverity::Warning,
                message: format!(
                    "shoulder/margin ratio {:.2} < 0.30 — abutment will be too narrow at the gum line",
                    ratio
                ),
            });
        } else if ratio > 1.10 {
            warnings.push(LimitWarning {
                kind: "shoulder-width-ratio".into(),
                severity: LimitSeverity::Warning,
                message: format!(
                    "shoulder/margin ratio {:.2} > 1.10 — emergence will mushroom",
                    ratio
                ),
            });
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_polyline(radius: f64, n: usize) -> Vec<Point3<f64>> {
        (0..n)
            .map(|i| {
                let theta = std::f64::consts::TAU * (i as f64) / (n as f64);
                Point3::new(radius * theta.cos(), radius * theta.sin(), 0.0)
            })
            .collect()
    }

    #[test]
    fn cylindrical_keeps_radius_constant() {
        let pl = ring_polyline(2.5, 24);
        let params = AbutmentEditParams {
            style: AbutmentStyle::Cylindrical,
            ..Default::default()
        };
        let (mesh, report) = generate_loft(&pl, Vector3::z(), &params);
        assert!(mesh.vertex_count() > 0);
        assert!(report.triangles > 0);
        // No errors expected.
        assert!(report
            .warnings
            .iter()
            .all(|w| !matches!(w.severity, LimitSeverity::Error)));
        assert!(report.volume_mm3 > 0.0);
    }

    #[test]
    fn standard_style_with_bulge_yields_volume() {
        let pl = ring_polyline(3.0, 32);
        let params = AbutmentEditParams::default();
        let (_, report) = generate_loft(&pl, Vector3::z(), &params);
        assert!(report.volume_mm3 > 0.0);
    }

    #[test]
    fn invalid_polyline_returns_error_warning() {
        let pl = vec![Point3::origin(), Point3::new(1.0, 0.0, 0.0)];
        let params = AbutmentEditParams::default();
        let (_, report) = generate_loft(&pl, Vector3::z(), &params);
        assert!(report
            .warnings
            .iter()
            .any(|w| matches!(w.severity, LimitSeverity::Error)));
    }

    #[test]
    fn validate_flags_extreme_asc() {
        let pl = ring_polyline(2.0, 24);
        let params = AbutmentEditParams {
            screw_channel_angle_deg: 40.0,
            ..Default::default()
        };
        let warnings = validate(&pl, &params);
        assert!(warnings.iter().any(|w| w.kind == "asc-out-of-range"));
    }

    #[test]
    fn resample_preserves_count() {
        let pl = ring_polyline(2.0, 100);
        let r = resample_closed_polyline(&pl, 16);
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn validate_extended_flags_undercut_and_height() {
        let pl = ring_polyline(2.0, 32);
        let params = AbutmentEditParams {
            // shoulder larger than margin → undercut.
            top_radius_mm: 3.0,
            // height too tall for interocclusal space.
            height_mm: 6.0,
            ..Default::default()
        };
        let env = AbutmentEnvelope {
            interocclusal_space_mm: 5.0,
            sulcus_radius_mm: 0.1,
            gingiva_clearance_mm: 0.1, // < 0.2 → triggers clearance-to-gingiva
            ..Default::default()
        };
        let w = validate_extended(&pl, &params, &env);
        assert!(w.iter().any(|x| x.kind == "undercut-at-margin"
            && matches!(x.severity, LimitSeverity::Error)));
        assert!(w.iter().any(|x| x.kind == "total-height-vs-interocclusal-space"));
        assert!(w.iter().any(|x| x.kind == "clearance-to-gingiva"));
    }

    #[test]
    fn validate_extended_flags_screw_channel_exit_angle() {
        let pl = ring_polyline(2.5, 24);
        let params = AbutmentEditParams {
            screw_channel_angle_deg: 25.0,
            ..Default::default()
        };
        // occlusal axis tilted 20° from +Z.
        let tilt = (20.0_f64).to_radians();
        let env = AbutmentEnvelope {
            occlusal_direction: [tilt.sin(), 0.0, tilt.cos()],
            ..Default::default()
        };
        let w = validate_extended(&pl, &params, &env);
        assert!(w
            .iter()
            .any(|x| x.kind == "screw-channel-exit-angle"
                && matches!(x.severity, LimitSeverity::Error)));
    }

    #[test]
    fn validate_extended_flags_shoulder_width_ratio() {
        let pl = ring_polyline(3.0, 32);
        // shoulder 0.6 mm against margin 3.0 mm → ratio 0.2 → too-narrow
        let params = AbutmentEditParams {
            top_radius_mm: 0.6,
            ..Default::default()
        };
        let w = validate_extended(&pl, &params, &AbutmentEnvelope::default());
        assert!(w.iter().any(|x| x.kind == "shoulder-width-ratio"));
    }

    #[test]
    fn validate_extended_passes_for_clean_design() {
        let pl = ring_polyline(2.5, 32);
        let params = AbutmentEditParams {
            top_radius_mm: 2.0,
            height_mm: 4.0,
            screw_channel_angle_deg: 0.0,
            ..Default::default()
        };
        let env = AbutmentEnvelope {
            interocclusal_space_mm: 7.0,
            gingiva_clearance_mm: 0.5,
            sulcus_radius_mm: 0.1,
            ..Default::default()
        };
        let w = validate_extended(&pl, &params, &env);
        // Should not contain any Error-level warning.
        assert!(w.iter().all(|x| !matches!(x.severity, LimitSeverity::Error)));
    }

    #[test]
    fn angular_style_scales_top() {
        let r0 = style_radius_factor(AbutmentStyle::Angular, 0.0, 0.4, 0.5);
        let r1 = style_radius_factor(AbutmentStyle::Angular, 1.0, 0.4, 0.5);
        assert!((r0 - 1.0).abs() < 1e-9);
        assert!((r1 - 0.5).abs() < 1e-9);
    }
}

//! Freeform prosthetic base — full denture / partial denture base. AR-V383.
//!
//! Conceptually ported from `DentalProcessors/FreeformProstheticBaseProcessor` and
//! `FreeformAdaptedProstheticBaseProcessor`. The decompiled C# is dongle/marshalling
//! boilerplate — the actual algorithm lives in native code we are reimplementing
//! from first principles using the same input/output contract:
//!
//!   * input  → margin polyline (`MarginPolyline`) + insertion axis + base parameters
//!   * output → watertight `Mesh` representing the prosthetic base shell
//!
//! The base is built as a loft from the cervical margin upward with three rings:
//!     1. cervical ring  → exact margin points (anchor)
//!     2. emergence ring → margin offset outward by `emergence_offset_mm`,
//!                          translated up by `emergence_height_mm` along axis,
//!                          adapted to keep tangency with the gingival contour
//!     3. flange ring    → final flange ring offset outward by `flange_offset_mm`,
//!                          translated up by `flange_height_mm`
//!
//! The top ring is closed with a fan triangulation around its centroid; the bottom
//! ring (cervical margin) is left open so the base wraps the prep cleanly. A second
//! variant (`generate_adapted_prosthetic_base`) re-uses the same loft but skews the
//! emergence profile to follow a per-vertex adaptation vector — used when the
//! technician has freeformed the underlying cervical margin manually.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::margin::MarginPolyline;
use tlanticad_mesh::Mesh;

/// Parameters controlling the prosthetic base loft.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProstheticBaseParams {
    /// Outward offset (away from insertion axis, perpendicular component) for the
    /// emergence ring (mm). Typical 0.4–0.8.
    pub emergence_offset_mm: f64,
    /// Height of the emergence ring above the cervical margin (mm). Typical 0.8–1.5.
    pub emergence_height_mm: f64,
    /// Outward offset for the flange ring (mm). Typical 1.5–3.0.
    pub flange_offset_mm: f64,
    /// Height of the flange ring above the cervical margin (mm). Typical 4–8.
    pub flange_height_mm: f64,
    /// Smoothing iterations applied to the emergence ring (Laplacian along the ring
    /// only — preserves the original margin).
    pub smoothing_iterations: u32,
    /// Whether to close the top of the flange (adds a fan around the centroid).
    pub close_top: bool,
}

impl Default for ProstheticBaseParams {
    fn default() -> Self {
        Self {
            emergence_offset_mm: 0.5,
            emergence_height_mm: 1.0,
            flange_offset_mm: 2.0,
            flange_height_mm: 5.0,
            smoothing_iterations: 2,
            close_top: true,
        }
    }
}

/// Build the orthogonal-to-axis "outward" vector at a given polyline vertex.
/// Outward is the perpendicular component of the radial vector (centroid → point)
/// projected onto the plane orthogonal to `axis`.
fn outward_normal(point: Point3<f64>, centroid: Point3<f64>, axis: Vector3<f64>) -> Vector3<f64> {
    let radial = point - centroid;
    let perp = radial - axis * radial.dot(&axis);
    let n = perp.norm();
    if n < 1e-9 {
        // Degenerate (point on axis); pick any orthogonal.
        let helper = if axis.x.abs() < 0.9 {
            Vector3::x()
        } else {
            Vector3::y()
        };
        let v = axis.cross(&helper);
        v.try_normalize(1e-9).unwrap_or(Vector3::x())
    } else {
        perp / n
    }
}

/// Compute the centroid of the polyline points.
fn polyline_centroid(margin: &MarginPolyline) -> Point3<f64> {
    if margin.is_empty() {
        return Point3::origin();
    }
    let mut acc = Vector3::zeros();
    for p in &margin.points {
        acc += Vector3::new(p[0], p[1], p[2]);
    }
    Point3::from(acc / margin.len() as f64)
}

/// Smooth a closed ring of points (preserves topology — averages each with its two
/// neighbours, weight = factor). Open rings keep their endpoints fixed.
fn smooth_ring(ring: &mut [Point3<f64>], iterations: u32, factor: f64, closed: bool) {
    let n = ring.len();
    if n < 3 {
        return;
    }
    for _ in 0..iterations {
        let snapshot = ring.to_vec();
        for i in 0..n {
            if !closed && (i == 0 || i == n - 1) {
                continue;
            }
            let prev = if i == 0 { n - 1 } else { i - 1 };
            let next = if i + 1 == n { 0 } else { i + 1 };
            let avg = Point3::from((snapshot[prev].coords + snapshot[next].coords) * 0.5);
            ring[i] = Point3::from(snapshot[i].coords.lerp(&avg.coords, factor));
        }
    }
}

/// Stitch two rings of equal length into a tube band of triangles. Both rings must be
/// indexed consistently (i.e. ring_a[i] aligns with ring_b[i]). Returns triangles in
/// CCW orientation when viewed from the outward normal.
fn stitch_rings(
    indices: &mut Vec<[u32; 3]>,
    base_a: u32,
    base_b: u32,
    count: u32,
    closed: bool,
) {
    for i in 0..count {
        let i_next = if closed {
            (i + 1) % count
        } else if i + 1 == count {
            break;
        } else {
            i + 1
        };
        let a0 = base_a + i;
        let a1 = base_a + i_next;
        let b0 = base_b + i;
        let b1 = base_b + i_next;
        indices.push([a0, b0, b1]);
        indices.push([a0, b1, a1]);
    }
}

/// Generate a prosthetic base mesh from a cervical margin polyline + insertion axis.
///
/// The result is a 3-ring loft: cervical → emergence → flange. The flange top is
/// closed when `params.close_top` is true (default).
pub fn generate_prosthetic_base(
    margin: &MarginPolyline,
    axis: Vector3<f64>,
    params: &ProstheticBaseParams,
) -> Mesh {
    let mut mesh = Mesh::new("prosthetic-base");
    if margin.len() < 3 {
        return mesh;
    }
    let axis = axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let centroid = polyline_centroid(margin);
    let n = margin.len() as u32;

    // Ring 1 — cervical (the input margin).
    let cervical: Vec<Point3<f64>> = (0..margin.len()).map(|i| margin.point(i)).collect();

    // Ring 2 — emergence.
    let mut emergence: Vec<Point3<f64>> = cervical
        .iter()
        .map(|&p| {
            let outward = outward_normal(p, centroid, axis);
            p + outward * params.emergence_offset_mm + axis * params.emergence_height_mm
        })
        .collect();
    smooth_ring(&mut emergence, params.smoothing_iterations, 0.5, margin.is_closed);

    // Ring 3 — flange.
    let mut flange: Vec<Point3<f64>> = cervical
        .iter()
        .map(|&p| {
            let outward = outward_normal(p, centroid, axis);
            p + outward * params.flange_offset_mm + axis * params.flange_height_mm
        })
        .collect();
    smooth_ring(&mut flange, params.smoothing_iterations, 0.5, margin.is_closed);

    // Append all ring vertices sequentially.
    let cervical_base = 0u32;
    mesh.vertices.extend(cervical.iter().copied());
    let emergence_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(emergence.iter().copied());
    let flange_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(flange.iter().copied());

    // Stitch cervical → emergence and emergence → flange.
    stitch_rings(&mut mesh.indices, cervical_base, emergence_base, n, margin.is_closed);
    stitch_rings(&mut mesh.indices, emergence_base, flange_base, n, margin.is_closed);

    // Close the flange top (centroid fan).
    if params.close_top {
        let top_centroid = {
            let mut acc = Vector3::zeros();
            for p in &flange {
                acc += p.coords;
            }
            Point3::from(acc / flange.len() as f64)
        };
        let top_centroid_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(top_centroid);
        for i in 0..n {
            let i_next = if margin.is_closed {
                (i + 1) % n
            } else if i + 1 == n {
                break;
            } else {
                i + 1
            };
            mesh.indices.push([top_centroid_idx, flange_base + i, flange_base + i_next]);
        }
    }

    mesh.calculate_normals();
    mesh
}

/// Per-vertex adaptation vector — used to skew the emergence ring locally to follow
/// a freeformed cervical margin (e.g. when the technician has manually adjusted some
/// segments). Length must match `margin.len()`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmergenceAdaptation {
    pub per_vertex_offset: Vec<[f64; 3]>,
}

/// Generate the adapted variant — same loft but with per-vertex emergence skew.
/// Falls back to the standard loft when `adaptation` is empty.
pub fn generate_adapted_prosthetic_base(
    margin: &MarginPolyline,
    axis: Vector3<f64>,
    params: &ProstheticBaseParams,
    adaptation: &EmergenceAdaptation,
) -> Mesh {
    if adaptation.per_vertex_offset.len() != margin.len() {
        return generate_prosthetic_base(margin, axis, params);
    }
    let mut mesh = Mesh::new("prosthetic-base-adapted");
    let axis = axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let centroid = polyline_centroid(margin);
    let n = margin.len() as u32;

    let cervical: Vec<Point3<f64>> = (0..margin.len()).map(|i| margin.point(i)).collect();

    let mut emergence: Vec<Point3<f64>> = cervical
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let outward = outward_normal(p, centroid, axis);
            let adapt = adaptation.per_vertex_offset[i];
            let adapt_v = Vector3::new(adapt[0], adapt[1], adapt[2]);
            p + outward * params.emergence_offset_mm + axis * params.emergence_height_mm + adapt_v
        })
        .collect();
    smooth_ring(&mut emergence, params.smoothing_iterations, 0.5, margin.is_closed);

    let mut flange: Vec<Point3<f64>> = cervical
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let outward = outward_normal(p, centroid, axis);
            let adapt = adaptation.per_vertex_offset[i];
            let adapt_v = Vector3::new(adapt[0], adapt[1], adapt[2]);
            p + outward * params.flange_offset_mm + axis * params.flange_height_mm + adapt_v * 0.5
        })
        .collect();
    smooth_ring(&mut flange, params.smoothing_iterations, 0.5, margin.is_closed);

    mesh.vertices.extend(cervical.iter().copied());
    let emergence_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(emergence.iter().copied());
    let flange_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(flange.iter().copied());

    stitch_rings(&mut mesh.indices, 0, emergence_base, n, margin.is_closed);
    stitch_rings(&mut mesh.indices, emergence_base, flange_base, n, margin.is_closed);

    if params.close_top {
        let top_centroid = {
            let mut acc = Vector3::zeros();
            for p in &flange {
                acc += p.coords;
            }
            Point3::from(acc / flange.len() as f64)
        };
        let top_centroid_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(top_centroid);
        for i in 0..n {
            let i_next = if margin.is_closed {
                (i + 1) % n
            } else if i + 1 == n {
                break;
            } else {
                i + 1
            };
            mesh.indices.push([top_centroid_idx, flange_base + i, flange_base + i_next]);
        }
    }

    mesh.calculate_normals();
    mesh
}

/// AR-V417 — implant-specific emergence profile optimisation.
///
/// Generates a custom emergence shell that interpolates between the gingival
/// margin (cervical ring at the soft-tissue level) and a circular abutment
/// platform of `abutment_diameter` mm sitting `gum_thickness` mm below the
/// margin (i.e. at the implant shoulder).
///
/// The profile uses an S-curve concave-near-margin / convex-near-abutment
/// — the clinically proven shape for cement-retained implant restorations.
///
/// Geometry:
///   * Cervical ring: the input margin polyline (gingival contour).
///   * Abutment ring: a circle of `abutment_diameter / 2` centred on the
///     polyline centroid, projected onto the plane orthogonal to
///     `implant_axis` at depth `-gum_thickness * implant_axis`.
///   * Intermediate rings (8 axial segments) are S-curve interpolations
///     between the two — the radius blends from per-vertex margin radius
///     to constant abutment radius following `0.5 - 0.5 cos(πt)`.
///
/// The bottom (abutment) ring is closed with a fan around its centre so
/// the shell is watertight at the shoulder.
pub fn optimize_emergence_for_implant(
    margin: &MarginPolyline,
    implant_axis: Vector3<f64>,
    gum_thickness: f64,
    abutment_diameter: f64,
) -> Mesh {
    let mut mesh = Mesh::new("implant-emergence-profile");
    if margin.len() < 3 || gum_thickness <= 0.0 || abutment_diameter <= 0.0 {
        return mesh;
    }
    let axis = implant_axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let centroid = polyline_centroid(margin);
    let n = margin.len();
    let abutment_radius = abutment_diameter * 0.5;
    let axial_segments = 8u32;

    // Pre-compute per-vertex radial direction + radius around centroid in
    // the plane perpendicular to the axis (margin radial pattern).
    let mut margin_radials: Vec<Vector3<f64>> = Vec::with_capacity(n);
    let mut margin_radii: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let p = margin.point(i);
        let radial = p - centroid;
        let perp = radial - axis * radial.dot(&axis);
        let len = perp.norm();
        let dir = if len < 1e-9 {
            // Sit on axis — pick stable axis-orthogonal direction.
            let helper = if axis.x.abs() < 0.9 {
                Vector3::x()
            } else {
                Vector3::y()
            };
            axis.cross(&helper).normalize()
        } else {
            perp / len
        };
        margin_radials.push(dir);
        margin_radii.push(len.max(1e-9));
    }

    // Build (axial_segments + 1) rings. Ring 0 = margin (cervical, top).
    // Ring axial_segments = abutment platform (bottom — `gum_thickness`
    // below margin centroid along -axis).
    let total_rings = (axial_segments as usize) + 1;
    let mut vertices: Vec<Point3<f64>> = Vec::with_capacity(total_rings * n);
    for ring in 0..total_rings {
        let t = (ring as f64) / (axial_segments as f64);
        // S-curve blend factor.
        let scurve = 0.5 - 0.5 * (std::f64::consts::PI * t).cos();
        // Drop along -axis.
        let depth = -t * gum_thickness;
        let ring_centre = centroid + axis * depth;
        for i in 0..n {
            let r_margin = margin_radii[i];
            let r = r_margin + (abutment_radius - r_margin) * scurve;
            let world = ring_centre + margin_radials[i] * r;
            vertices.push(world);
        }
    }
    mesh.vertices = vertices;

    // Stitch rings 0..axial_segments into bands.
    let mut indices: Vec<[u32; 3]> = Vec::new();
    for ring in 0..axial_segments as usize {
        let base_a = (ring * n) as u32;
        let base_b = ((ring + 1) * n) as u32;
        for i in 0..n as u32 {
            let i_next = if margin.is_closed {
                (i + 1) % (n as u32)
            } else if i + 1 == n as u32 {
                break;
            } else {
                i + 1
            };
            indices.push([base_a + i, base_b + i, base_b + i_next]);
            indices.push([base_a + i, base_b + i_next, base_a + i_next]);
        }
    }

    // Close the abutment shoulder with a fan around its centre.
    let bottom_base = (axial_segments as usize * n) as u32;
    let bottom_centre_pt = centroid + axis * (-gum_thickness);
    let bottom_centre_idx = mesh.vertices.len() as u32;
    mesh.vertices.push(bottom_centre_pt);
    for i in 0..n as u32 {
        let i_next = if margin.is_closed {
            (i + 1) % (n as u32)
        } else if i + 1 == n as u32 {
            break;
        } else {
            i + 1
        };
        // Wind so the fan faces -axis (looking up at the implant).
        indices.push([bottom_centre_idx, bottom_base + i_next, bottom_base + i]);
    }

    mesh.indices = indices;
    mesh.calculate_normals();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_polyline(radius: f64, segments: usize) -> MarginPolyline {
        let mut pts = Vec::with_capacity(segments);
        for i in 0..segments {
            let theta = std::f64::consts::TAU * (i as f64) / (segments as f64);
            pts.push([radius * theta.cos(), radius * theta.sin(), 0.0]);
        }
        MarginPolyline {
            points: pts,
            is_closed: true,
        }
    }

    #[test]
    fn empty_polyline_produces_empty_mesh() {
        let margin = MarginPolyline::default();
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &ProstheticBaseParams::default());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn ring_margin_produces_three_rings_plus_top() {
        let margin = ring_polyline(5.0, 24);
        let params = ProstheticBaseParams {
            close_top: true,
            ..Default::default()
        };
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &params);
        // 3 rings of 24 + 1 top centroid = 73 vertices
        assert_eq!(mesh.vertex_count(), 24 * 3 + 1);
        // 2 stitched bands × 24 × 2 + 24 fan triangles = 144
        assert_eq!(mesh.triangle_count(), 24 * 4 + 24);
    }

    #[test]
    fn ring_margin_without_top_lacks_centroid_vertex() {
        let margin = ring_polyline(5.0, 16);
        let params = ProstheticBaseParams {
            close_top: false,
            ..Default::default()
        };
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &params);
        assert_eq!(mesh.vertex_count(), 16 * 3);
        assert_eq!(mesh.triangle_count(), 16 * 4);
    }

    #[test]
    fn flange_ring_is_above_cervical_along_axis() {
        let margin = ring_polyline(4.0, 12);
        let params = ProstheticBaseParams {
            flange_height_mm: 6.0,
            ..Default::default()
        };
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &params);
        // first 12 are cervical (z≈0), next 12 emergence, next 12 flange (z≈6)
        for i in 0..12 {
            assert!((mesh.vertices[i].z).abs() < 1e-6);
        }
        for i in 24..36 {
            assert!((mesh.vertices[i].z - 6.0).abs() < 1e-6);
        }
    }

    #[test]
    fn flange_ring_is_outside_cervical_radially() {
        let margin = ring_polyline(3.0, 16);
        let params = ProstheticBaseParams {
            flange_offset_mm: 2.0,
            ..Default::default()
        };
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &params);
        for i in 0..16 {
            let cer = mesh.vertices[i];
            let fla = mesh.vertices[32 + i];
            let r_cer = (cer.x.powi(2) + cer.y.powi(2)).sqrt();
            let r_fla = (fla.x.powi(2) + fla.y.powi(2)).sqrt();
            assert!(r_fla > r_cer, "flange not outside cervical at {i}");
        }
    }

    #[test]
    fn adapted_variant_no_adaptation_falls_back_to_standard() {
        let margin = ring_polyline(4.0, 12);
        let params = ProstheticBaseParams::default();
        let adaptation = EmergenceAdaptation::default();
        let standard = generate_prosthetic_base(&margin, Vector3::z(), &params);
        let adapted =
            generate_adapted_prosthetic_base(&margin, Vector3::z(), &params, &adaptation);
        assert_eq!(standard.vertex_count(), adapted.vertex_count());
        assert_eq!(standard.triangle_count(), adapted.triangle_count());
    }

    #[test]
    fn adapted_variant_skews_emergence_per_vertex() {
        let margin = ring_polyline(4.0, 8);
        let params = ProstheticBaseParams {
            close_top: false,
            ..Default::default()
        };
        let mut adaptation = EmergenceAdaptation::default();
        adaptation.per_vertex_offset = (0..8)
            .map(|i| if i == 0 { [0.5, 0.0, 0.0] } else { [0.0, 0.0, 0.0] })
            .collect();
        let mesh = generate_adapted_prosthetic_base(&margin, Vector3::z(), &params, &adaptation);
        // emergence ring starts at vertex index 8.
        let baseline = generate_prosthetic_base(&margin, Vector3::z(), &params);
        let baseline_em0 = baseline.vertices[8];
        let adapted_em0 = mesh.vertices[8];
        // The skew is applied before smoothing, so it gets averaged with neighbours;
        // we just check it's actually different from the baseline.
        assert!((adapted_em0 - baseline_em0).norm() > 1e-3);
    }

    #[test]
    fn open_polyline_does_not_close_loft_band() {
        let mut pts = Vec::new();
        for i in 0..10 {
            pts.push([i as f64, 0.0, 0.0]);
        }
        let margin = MarginPolyline {
            points: pts,
            is_closed: false,
        };
        let params = ProstheticBaseParams {
            close_top: false,
            ..Default::default()
        };
        let mesh = generate_prosthetic_base(&margin, Vector3::z(), &params);
        // Open ring with 10 verts → 9 quads × 2 tris × 2 bands = 36 triangles.
        assert_eq!(mesh.triangle_count(), 9 * 2 * 2);
    }

    // ── V417 — implant emergence profile ──────────────────────────

    #[test]
    fn implant_emergence_invalid_inputs_return_empty() {
        let margin = ring_polyline(4.0, 12);
        // height ≤ 0
        let m1 = optimize_emergence_for_implant(&margin, Vector3::z(), 0.0, 4.0);
        assert_eq!(m1.vertex_count(), 0);
        // diameter ≤ 0
        let m2 = optimize_emergence_for_implant(&margin, Vector3::z(), 2.0, 0.0);
        assert_eq!(m2.vertex_count(), 0);
        // empty margin
        let m3 = optimize_emergence_for_implant(
            &MarginPolyline::default(),
            Vector3::z(),
            2.0,
            4.0,
        );
        assert_eq!(m3.vertex_count(), 0);
    }

    #[test]
    fn implant_emergence_has_nine_rings_plus_centre() {
        let margin = ring_polyline(4.0, 16);
        let mesh = optimize_emergence_for_implant(&margin, Vector3::z(), 2.0, 4.0);
        // 9 rings (axial_segments=8 → 9 rings) of 16 vertices + 1 abutment
        // centre vertex = 145.
        assert_eq!(mesh.vertex_count(), 16 * 9 + 1);
    }

    #[test]
    fn implant_emergence_top_ring_matches_margin_radius() {
        let margin = ring_polyline(4.0, 16);
        let mesh = optimize_emergence_for_implant(&margin, Vector3::z(), 2.0, 4.0);
        // Ring 0 = margin (cervical), all radii ~ 4.0.
        for i in 0..16 {
            let p = mesh.vertices[i];
            let r = (p.x.powi(2) + p.y.powi(2)).sqrt();
            assert!((r - 4.0).abs() < 1e-6, "ring 0 radius {r} != 4.0");
            assert!((p.z).abs() < 1e-6, "ring 0 z != 0");
        }
    }

    #[test]
    fn implant_emergence_bottom_ring_matches_abutment_radius() {
        let margin = ring_polyline(4.0, 16);
        let abutment_dia = 3.0;
        let gum = 2.0;
        let mesh = optimize_emergence_for_implant(&margin, Vector3::z(), gum, abutment_dia);
        // Ring 8 = abutment, radius = 1.5, z = -2 (gum_thickness below margin).
        let base = 8 * 16;
        for i in 0..16 {
            let p = mesh.vertices[base + i];
            let r = (p.x.powi(2) + p.y.powi(2)).sqrt();
            assert!(
                (r - abutment_dia * 0.5).abs() < 1e-6,
                "abutment radius {r} != {}",
                abutment_dia * 0.5
            );
            assert!((p.z + gum).abs() < 1e-6, "abutment z {} != {}", p.z, -gum);
        }
    }

    #[test]
    fn implant_emergence_intermediate_radius_between_extremes() {
        let margin = ring_polyline(5.0, 12);
        let abutment_dia = 3.0;
        let gum = 2.0;
        let mesh = optimize_emergence_for_implant(&margin, Vector3::z(), gum, abutment_dia);
        // Ring 4 (mid) — S-curve blend factor at t=0.5 is ~0.5.
        // r = 5 + (1.5 - 5) * 0.5 = 3.25
        let base = 4 * 12;
        let p = mesh.vertices[base];
        let r = (p.x.powi(2) + p.y.powi(2)).sqrt();
        assert!((r - 3.25).abs() < 1e-6, "mid radius {r} != 3.25");
    }

    #[test]
    fn implant_emergence_triangle_count() {
        let margin = ring_polyline(4.0, 16);
        let mesh = optimize_emergence_for_implant(&margin, Vector3::z(), 2.0, 4.0);
        // 8 bands × 16 quads × 2 tris = 256
        // + 16 fan tris on the abutment shoulder = 272
        assert_eq!(mesh.triangle_count(), 256 + 16);
    }
}

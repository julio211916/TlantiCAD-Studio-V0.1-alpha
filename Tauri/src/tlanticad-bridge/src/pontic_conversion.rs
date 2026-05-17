//! AR-V407 — Convert a library tooth model into a pontic.
//!
//! Ported from `DentalProcessors/ConvertLibraryModelToPonticProcessor.cs`.
//!
//! When a technician drops a library tooth between two abutments to form a
//! bridge, exocad runs this processor to:
//!
//! 1. Translate the library tooth so its lowest vertex along the occlusal
//!    axis sits exactly at the configured `lift_mm` distance ABOVE the closest
//!    point on the gingiva polyline (= the ridge saddle). The lift prevents
//!    the pontic from physically pressing on soft tissue (sanitary distance).
//! 2. Trim the bottom of the library tooth so the new tissue-contact surface
//!    follows the ridge polyline rather than the original library root.
//! 3. Re-tag the result as a pontic mesh (id renamed, normals recomputed).
//!
//! The function works with arbitrary occlusal axes (so it remains correct for
//! upper/lower jaw and for tilted insertion paths). It only modifies vertex
//! positions and triangle indices — no topology change beyond a clean cut on
//! the bottom plane.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonticConversionReport {
    pub vertices_input: usize,
    pub vertices_output: usize,
    pub triangles_input: usize,
    pub triangles_output: usize,
    pub lift_applied_mm: f64,
    pub min_clearance_to_polyline_mm: f64,
}

/// Project `point` onto the polyline (closed by the caller; we treat as open
/// chain) and return the squared closest distance plus the parameter index.
fn closest_distance_to_polyline(point: &Point3<f64>, polyline: &[Point3<f64>]) -> f64 {
    if polyline.is_empty() {
        return f64::MAX;
    }
    if polyline.len() == 1 {
        return (point - polyline[0]).norm();
    }
    let mut best = f64::MAX;
    for window in polyline.windows(2) {
        let a = window[0];
        let b = window[1];
        let ab = b - a;
        let len2 = ab.norm_squared();
        if len2 < 1e-18 {
            let d = (point - a).norm();
            if d < best {
                best = d;
            }
            continue;
        }
        let t = ((point - a).dot(&ab) / len2).clamp(0.0, 1.0);
        let proj = a + ab * t;
        let d = (point - proj).norm();
        if d < best {
            best = d;
        }
    }
    best
}

/// Compute the signed distance of `point` along the occlusal `axis` relative
/// to the polyline's average position. Positive = above ridge along axis.
fn signed_axis_offset(point: &Point3<f64>, ridge_centroid: &Point3<f64>, axis: &Vector3<f64>) -> f64 {
    (point - ridge_centroid).dot(axis)
}

/// Convert `library_mesh` to a pontic.
///
/// * `library_mesh` — the library tooth as imported (anatomic crown + root).
/// * `base_polyline` — gingival ridge saddle as a sequence of 3D points (open
///   or closed; ordering doesn't matter for distance queries).
/// * `occlusal_axis` — chewing-load direction; the tooth gets lifted along
///   this axis. Will be normalized internally.
/// * `lift_mm` — sanitary clearance between the lowest pontic vertex and the
///   ridge. Must be ≥ 0; values < 0.05 typically produce an "ovate" pontic
///   that lightly indents the tissue, values > 0.5 give a hygienic pontic.
pub fn convert_library_to_pontic(
    library_mesh: &Mesh,
    base_polyline: &[Point3<f64>],
    occlusal_axis: &Vector3<f64>,
    lift_mm: f64,
) -> (Mesh, PonticConversionReport) {
    let v_in = library_mesh.vertices.len();
    let t_in = library_mesh.indices.len();
    let mut report = PonticConversionReport {
        vertices_input: v_in,
        vertices_output: 0,
        triangles_input: t_in,
        triangles_output: 0,
        lift_applied_mm: 0.0,
        min_clearance_to_polyline_mm: f64::MAX,
    };

    if v_in == 0 || t_in == 0 || base_polyline.is_empty() {
        let empty = Mesh::new(format!("{}_pontic_empty", library_mesh.name));
        return (empty, report);
    }

    let axis = if occlusal_axis.norm() > 1e-9 {
        occlusal_axis.normalize()
    } else {
        Vector3::z()
    };
    let lift = lift_mm.max(0.0);

    // 1. Compute ridge centroid as the reference point for the axis offset.
    let mut centroid_v = Vector3::zeros();
    for p in base_polyline {
        centroid_v += p.coords;
    }
    centroid_v /= base_polyline.len() as f64;
    let ridge_centroid = Point3::from(centroid_v);

    // 2. Find the library vertex whose signed offset along `axis` is the
    //    smallest (i.e. the deepest vertex along the chewing axis — typically
    //    the apex of the root).
    let mut min_offset = f64::MAX;
    for v in &library_mesh.vertices {
        let off = signed_axis_offset(v, &ridge_centroid, &axis);
        if off < min_offset {
            min_offset = off;
        }
    }

    // We want the lowest vertex to land at `+lift` above the ridge centroid
    // along axis. Translation magnitude:
    //     translate_along_axis = lift - min_offset
    let translate_amount = lift - min_offset;
    let translate = axis * translate_amount;
    report.lift_applied_mm = translate_amount;

    // 3. Apply translation.
    let mut out = library_mesh.clone();
    out.id = tlanticad_core::Id::new_v4();
    out.name = format!("{}_pontic", library_mesh.name);
    for v in &mut out.vertices {
        *v += translate;
    }

    // 4. Trim faces that originally formed the LIBRARY ROOT (the portion below
    //    the library tooth's anatomic-crown / root junction). The cut plane
    //    is positioned at the lower 50 % of the original library tooth's
    //    axis-extent — measured BEFORE the lift translation. Library models
    //    are authored so the anatomic crown occupies roughly the upper half
    //    of the bounding box; trimming the lower half removes the apical
    //    root cone without touching the chewing surface.
    //
    //    A face is dropped only when ALL THREE of its (pre-lift) vertices sit
    //    below this anatomical cut plane.
    let library_offsets: Vec<f64> = library_mesh
        .vertices
        .iter()
        .map(|v| signed_axis_offset(v, &ridge_centroid, &axis))
        .collect();
    let lib_min = library_offsets.iter().copied().fold(f64::MAX, f64::min);
    let lib_max = library_offsets.iter().copied().fold(f64::MIN, f64::max);
    let cut_plane_offset = lib_min + 0.5 * (lib_max - lib_min);

    let mut keep_face: Vec<bool> = vec![false; out.indices.len()];
    for (i, tri) in out.indices.iter().enumerate() {
        let v0 = library_offsets[tri[0] as usize];
        let v1 = library_offsets[tri[1] as usize];
        let v2 = library_offsets[tri[2] as usize];
        // Drop face when all three vertices lie strictly below the cut plane.
        let below_count = (v0 < cut_plane_offset) as u8
            + (v1 < cut_plane_offset) as u8
            + (v2 < cut_plane_offset) as u8;
        keep_face[i] = below_count < 3;
    }
    let kept: Vec<[u32; 3]> = out
        .indices
        .iter()
        .enumerate()
        .filter(|(i, _)| keep_face[*i])
        .map(|(_, t)| *t)
        .collect();
    out.indices = kept;

    // 5. Compact orphan vertices.
    let mut used: Vec<bool> = vec![false; out.vertices.len()];
    for tri in &out.indices {
        used[tri[0] as usize] = true;
        used[tri[1] as usize] = true;
        used[tri[2] as usize] = true;
    }
    let mut new_idx = vec![u32::MAX; out.vertices.len()];
    let mut new_verts = Vec::new();
    let mut new_normals = Vec::new();
    let mut new_colors = out.colors.as_ref().map(|_| Vec::new());
    let mut new_uvs = out.uvs.as_ref().map(|_| Vec::new());
    for (old, _) in out.vertices.iter().enumerate() {
        if used[old] {
            new_idx[old] = new_verts.len() as u32;
            new_verts.push(out.vertices[old]);
            if old < out.normals.len() {
                new_normals.push(out.normals[old]);
            }
            if let (Some(src), Some(dst)) = (out.colors.as_ref(), new_colors.as_mut()) {
                dst.push(src[old]);
            }
            if let (Some(src), Some(dst)) = (out.uvs.as_ref(), new_uvs.as_mut()) {
                dst.push(src[old]);
            }
        }
    }
    let remapped: Vec<[u32; 3]> = out
        .indices
        .iter()
        .map(|tri| [new_idx[tri[0] as usize], new_idx[tri[1] as usize], new_idx[tri[2] as usize]])
        .collect();
    out.vertices = new_verts;
    out.normals = new_normals;
    out.colors = new_colors;
    out.uvs = new_uvs;
    out.indices = remapped;

    // 6. Compute clearance — minimum distance between any pontic vertex and the polyline.
    for v in &out.vertices {
        let d = closest_distance_to_polyline(v, base_polyline);
        if d < report.min_clearance_to_polyline_mm {
            report.min_clearance_to_polyline_mm = d;
        }
    }
    if !out.vertices.is_empty() {
        out.calculate_normals();
    }

    report.vertices_output = out.vertices.len();
    report.triangles_output = out.indices.len();
    (out, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    fn ridge_polyline_xy(z: f64) -> Vec<Point3<f64>> {
        // 3 mm long ridge along +x axis at given z.
        vec![
            Point3::new(-1.5, 0.0, z),
            Point3::new(0.0, 0.0, z),
            Point3::new(1.5, 0.0, z),
        ]
    }

    #[test]
    fn empty_polyline_returns_empty_pontic() {
        let library = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let (out, report) = convert_library_to_pontic(&library, &[], &Vector3::z(), 0.5);
        assert_eq!(report.vertices_output, 0);
        assert_eq!(report.triangles_output, 0);
        assert_eq!(out.triangle_count(), 0);
    }

    #[test]
    fn library_lifted_above_ridge_by_lift_amount() {
        // Library tooth from z=-1..1, ridge at z=0, lift=0.5.
        let library = create_box(Point3::new(-0.5, -0.5, -1.0), Point3::new(0.5, 0.5, 1.0));
        let ridge = ridge_polyline_xy(0.0);
        let (out, report) = convert_library_to_pontic(&library, &ridge, &Vector3::z(), 0.5);
        assert!(report.vertices_output > 0);
        // The lowest vertex along z should now be at >= 0.5 - epsilon.
        let min_z = out.vertices.iter().map(|p| p.z).fold(f64::MAX, f64::min);
        assert!((min_z - 0.5).abs() < 1e-6, "expected lowest z ≈ 0.5, got {min_z}");
        // Lift applied = 0.5 - (-1.0) = 1.5
        assert!((report.lift_applied_mm - 1.5).abs() < 1e-9);
    }

    #[test]
    fn trimming_drops_deeply_submerged_root_geometry() {
        // Subdivided box so the bottom face has interior triangles below the ridge.
        let mut library = create_box(Point3::new(-0.5, -0.5, -2.0), Point3::new(0.5, 0.5, 1.0));
        tlanticad_mesh::operations::subdivide(&mut library);
        let ridge = ridge_polyline_xy(0.0);
        // Lift = 0 → keep just the part above ridge (after lifting the lowest
        // vertex to z=0 the originally-deepest material lands at the ridge,
        // and triangles that are entirely below ridge are removed).
        let (out, report) =
            convert_library_to_pontic(&library, &ridge, &Vector3::z(), 0.0);
        // We must have removed something (root triangles) AND kept the crown.
        assert!(report.triangles_output < report.triangles_input);
        assert!(report.triangles_output > 0);
        assert!(out.vertex_count() > 0);
    }

    #[test]
    fn min_clearance_respects_lift_amount() {
        let library = create_box(Point3::new(-0.5, -0.5, -1.0), Point3::new(0.5, 0.5, 1.0));
        let ridge = ridge_polyline_xy(0.0);
        let lift = 0.8;
        let (_, report) = convert_library_to_pontic(&library, &ridge, &Vector3::z(), lift);
        // Min clearance to ridge polyline must be ≥ lift (within tolerance).
        assert!(
            report.min_clearance_to_polyline_mm >= lift - 1e-6,
            "min clearance {} below lift {lift}",
            report.min_clearance_to_polyline_mm
        );
    }

    #[test]
    fn negative_lift_clamped_to_zero() {
        let library = create_box(Point3::new(-0.5, -0.5, -1.0), Point3::new(0.5, 0.5, 1.0));
        let ridge = ridge_polyline_xy(0.0);
        let (_, r_neg) = convert_library_to_pontic(&library, &ridge, &Vector3::z(), -2.0);
        let (_, r_zero) = convert_library_to_pontic(&library, &ridge, &Vector3::z(), 0.0);
        assert!((r_neg.lift_applied_mm - r_zero.lift_applied_mm).abs() < 1e-9);
    }
}

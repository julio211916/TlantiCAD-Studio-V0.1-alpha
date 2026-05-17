//! AR-V404 — Generic visualization mesh conversion (4 variants).
//!
//! Ported from `DentalProcessors/ConvertGenericVisualizationMesh{...}Processor.cs`
//! (FromAntagonist / FromToothModel / ToAntagonist / ToToothModel).
//!
//! Exocad keeps a single in-memory mesh per scan but tags every vertex with a
//! semantic role (antagonist, tooth model, gingiva, …) so that a viewer shader
//! can recolor a region without duplicating geometry. These four processors
//! convert vertices between the "generic visualization" tag space and the
//! domain-specific tag (antagonist / tooth-model). They also:
//!
//! * Optionally generate a downsampled LOD copy when the source mesh exceeds a
//!   threshold (faster preview painting in the OCC view).
//! * Optionally normalize coordinates into a unit-cube (-1..1) so the
//!   visualization shader can apply a uniform scale matrix.
//!
//! All four functions return a fresh `Mesh` so the original can be retained
//! for further processing — this matches exocad's pure-functional pipeline.

use crate::Mesh;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Semantic role tags encoded in a vertex color slot. Values map to the
/// `[r, g, b, a]` byte tuple in `Mesh::colors`. The colors themselves are
/// hand-picked to render legibly in the OCC view shader at default lighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum VisualizationTag {
    /// Antagonist surface — light pink (#FFD9D9).
    Antagonist = 1,
    /// Tooth model surface — ivory white (#FFF6E0).
    ToothModel = 2,
    /// Generic / unclassified visualization mesh — neutral grey (#C8C8C8).
    Generic = 3,
}

impl VisualizationTag {
    /// RGBA color for this tag, in 0..=255 sRGB.
    pub fn rgba(self) -> [u8; 4] {
        match self {
            VisualizationTag::Antagonist => [0xFF, 0xD9, 0xD9, 0xFF],
            VisualizationTag::ToothModel => [0xFF, 0xF6, 0xE0, 0xFF],
            VisualizationTag::Generic => [0xC8, 0xC8, 0xC8, 0xFF],
        }
    }

    /// Recover the tag from an RGBA tuple. Falls back to `Generic` for unknown
    /// triples (alpha is ignored).
    pub fn from_rgba(rgba: [u8; 4]) -> Self {
        match (rgba[0], rgba[1], rgba[2]) {
            (0xFF, 0xD9, 0xD9) => VisualizationTag::Antagonist,
            (0xFF, 0xF6, 0xE0) => VisualizationTag::ToothModel,
            _ => VisualizationTag::Generic,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualizationOptions {
    /// If true, normalize vertex coordinates into the cube [-1, 1] (preserves
    /// aspect ratio — uses the largest extent as the divisor).
    pub normalize: bool,
    /// If `Some(threshold)` and the source mesh has more triangles than the
    /// threshold, build a decimated LOD copy and return it as a sibling mesh.
    /// (We don't fold both into a single output; the caller decides which to
    /// hand to the shader. The decimated copy is exposed via `lod_copy` on
    /// the `VisualizationConvertReport`.)
    pub lod_threshold_tris: Option<usize>,
    /// Target triangle count for LOD generation when threshold is exceeded.
    /// Decimation is uniform face-stride sampling (deterministic).
    pub lod_target_tris: usize,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            normalize: false,
            lod_threshold_tris: None,
            lod_target_tris: 5_000,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VisualizationConvertReport {
    pub vertices_tagged: usize,
    pub triangles_kept: usize,
    pub normalized: bool,
    pub lod_copy: Option<Mesh>,
}

fn tag_all_vertices(mesh: &mut Mesh, tag: VisualizationTag) -> usize {
    let rgba = tag.rgba();
    if mesh.colors.is_none() {
        mesh.colors = Some(vec![rgba; mesh.vertices.len()]);
        return mesh.vertices.len();
    }
    let buf = mesh.colors.as_mut().unwrap();
    if buf.len() < mesh.vertices.len() {
        buf.resize(mesh.vertices.len(), rgba);
    }
    let mut count = 0;
    for slot in buf.iter_mut() {
        *slot = rgba;
        count += 1;
    }
    count
}

/// Center the mesh around its bounding-box midpoint and scale isotropically
/// so the largest extent fits in [-1, 1]. Fully no-op for empty meshes.
fn normalize_to_unit_cube(mesh: &mut Mesh) {
    if mesh.vertices.is_empty() {
        return;
    }
    let (min, max) = mesh.calculate_bounds();
    let center = Point3::from((min.coords + max.coords) * 0.5);
    let extents = max - min;
    let max_extent = extents.x.max(extents.y).max(extents.z);
    if max_extent < 1e-9 {
        // Degenerate (single point) — translate to origin and stop.
        for v in &mut mesh.vertices {
            *v = Point3::origin();
        }
        return;
    }
    let scale = 2.0 / max_extent;
    for v in &mut mesh.vertices {
        let local: Vector3<f64> = v.coords - center.coords;
        *v = Point3::from(local * scale);
    }
}

/// Build a decimated LOD copy via uniform face-stride sampling. Vertices are
/// compacted so the result is a valid standalone mesh. Returns `None` when the
/// source has fewer triangles than the target.
fn build_lod_copy(mesh: &Mesh, target_tris: usize) -> Option<Mesh> {
    if mesh.indices.len() <= target_tris || target_tris == 0 {
        return None;
    }
    let stride = (mesh.indices.len() + target_tris - 1) / target_tris;
    let mut sampled: Vec<[u32; 3]> = Vec::with_capacity(target_tris + 1);
    for (i, tri) in mesh.indices.iter().enumerate() {
        if i % stride == 0 {
            sampled.push(*tri);
        }
    }
    // Compact vertices used by sampled tris.
    let mut new_idx = vec![u32::MAX; mesh.vertices.len()];
    let mut new_verts = Vec::new();
    let mut new_colors = mesh.colors.as_ref().map(|_| Vec::new());
    for tri in &sampled {
        for &v in tri {
            if new_idx[v as usize] == u32::MAX {
                new_idx[v as usize] = new_verts.len() as u32;
                new_verts.push(mesh.vertices[v as usize]);
                if let (Some(src), Some(dst)) = (mesh.colors.as_ref(), new_colors.as_mut()) {
                    dst.push(src[v as usize]);
                }
            }
        }
    }
    let remapped: Vec<[u32; 3]> = sampled
        .iter()
        .map(|tri| [new_idx[tri[0] as usize], new_idx[tri[1] as usize], new_idx[tri[2] as usize]])
        .collect();
    let mut lod = Mesh::new(format!("{}_lod", mesh.name));
    lod.vertices = new_verts;
    lod.indices = remapped;
    lod.colors = new_colors;
    lod.calculate_normals();
    Some(lod)
}

/// Common conversion engine. Clones source, retags every vertex, optionally
/// normalizes & emits LOD.
fn convert(mesh: &Mesh, target_tag: VisualizationTag, options: &VisualizationOptions) -> (Mesh, VisualizationConvertReport) {
    let mut out = mesh.clone();
    out.id = tlanticad_core::Id::new_v4();
    out.name = format!("{}_{}", mesh.name, match target_tag {
        VisualizationTag::Antagonist => "antagonist",
        VisualizationTag::ToothModel => "tooth_model",
        VisualizationTag::Generic => "generic_viz",
    });

    let tagged = tag_all_vertices(&mut out, target_tag);
    if options.normalize {
        normalize_to_unit_cube(&mut out);
    }
    let lod_copy = options
        .lod_threshold_tris
        .filter(|t| out.indices.len() > *t)
        .and_then(|_| build_lod_copy(&out, options.lod_target_tris));
    let triangles_kept = out.indices.len();
    out.calculate_normals();

    (
        out,
        VisualizationConvertReport {
            vertices_tagged: tagged,
            triangles_kept,
            normalized: options.normalize,
            lod_copy,
        },
    )
}

/// Convert a generic visualization mesh into an antagonist tag.
pub fn convert_from_generic_to_antagonist(
    mesh: &Mesh,
    options: &VisualizationOptions,
) -> (Mesh, VisualizationConvertReport) {
    convert(mesh, VisualizationTag::Antagonist, options)
}

/// Convert a generic visualization mesh into a tooth-model tag.
pub fn convert_from_generic_to_tooth_model(
    mesh: &Mesh,
    options: &VisualizationOptions,
) -> (Mesh, VisualizationConvertReport) {
    convert(mesh, VisualizationTag::ToothModel, options)
}

/// Convert any tagged mesh BACK to a neutral generic visualization tag,
/// originally tagged as antagonist (drops antagonist semantics).
pub fn convert_to_generic_from_antagonist(
    mesh: &Mesh,
    options: &VisualizationOptions,
) -> (Mesh, VisualizationConvertReport) {
    convert(mesh, VisualizationTag::Generic, options)
}

/// Convert any tagged mesh BACK to a neutral generic visualization tag,
/// originally tagged as tooth-model (drops tooth-model semantics).
pub fn convert_to_generic_from_tooth_model(
    mesh: &Mesh,
    options: &VisualizationOptions,
) -> (Mesh, VisualizationConvertReport) {
    convert(mesh, VisualizationTag::Generic, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_box;
    use nalgebra::Point3;

    #[test]
    fn from_generic_to_antagonist_tags_all_vertices() {
        let src = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let (out, rep) = convert_from_generic_to_antagonist(&src, &VisualizationOptions::default());
        assert_eq!(rep.vertices_tagged, src.vertex_count());
        let cols = out.colors.as_ref().expect("colors must be populated");
        for c in cols {
            assert_eq!(*c, VisualizationTag::Antagonist.rgba());
        }
    }

    #[test]
    fn from_generic_to_tooth_model_normalizes_when_requested() {
        let src = create_box(Point3::new(10.0, 10.0, 10.0), Point3::new(20.0, 20.0, 20.0));
        let opts = VisualizationOptions {
            normalize: true,
            ..Default::default()
        };
        let (out, rep) = convert_from_generic_to_tooth_model(&src, &opts);
        assert!(rep.normalized);
        let (min, max) = out.calculate_bounds();
        assert!(min.x >= -1.0001 && max.x <= 1.0001);
        assert!(min.y >= -1.0001 && max.y <= 1.0001);
        assert!(min.z >= -1.0001 && max.z <= 1.0001);
        // Tag check.
        let cols = out.colors.as_ref().unwrap();
        assert_eq!(cols[0], VisualizationTag::ToothModel.rgba());
    }

    #[test]
    fn to_generic_from_antagonist_emits_generic_tag() {
        let mut src = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        // Pretend it was antagonist-tagged.
        src.colors = Some(vec![VisualizationTag::Antagonist.rgba(); src.vertex_count()]);
        let (out, _) = convert_to_generic_from_antagonist(&src, &VisualizationOptions::default());
        let cols = out.colors.as_ref().unwrap();
        for c in cols {
            assert_eq!(*c, VisualizationTag::Generic.rgba());
            assert_eq!(VisualizationTag::from_rgba(*c), VisualizationTag::Generic);
        }
    }

    #[test]
    fn to_generic_from_tooth_model_preserves_geometry() {
        let mut src = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        src.colors = Some(vec![VisualizationTag::ToothModel.rgba(); src.vertex_count()]);
        let v_before = src.vertex_count();
        let t_before = src.triangle_count();
        let (out, _) = convert_to_generic_from_tooth_model(&src, &VisualizationOptions::default());
        assert_eq!(out.vertex_count(), v_before);
        assert_eq!(out.triangle_count(), t_before);
    }

    #[test]
    fn lod_copy_is_emitted_when_threshold_exceeded() {
        let src = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        // Subdivide so we cross the threshold.
        let mut big = src.clone();
        crate::operations::subdivide(&mut big);
        crate::operations::subdivide(&mut big);
        let opts = VisualizationOptions {
            normalize: false,
            lod_threshold_tris: Some(20),
            lod_target_tris: 10,
        };
        let (_, rep) = convert_from_generic_to_antagonist(&big, &opts);
        let lod = rep.lod_copy.expect("LOD copy must be present");
        assert!(lod.triangle_count() > 0);
        assert!(lod.triangle_count() < big.triangle_count());
    }

    #[test]
    fn lod_copy_skipped_when_below_threshold() {
        let src = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let opts = VisualizationOptions {
            normalize: false,
            lod_threshold_tris: Some(10_000),
            lod_target_tris: 100,
        };
        let (_, rep) = convert_from_generic_to_tooth_model(&src, &opts);
        assert!(rep.lod_copy.is_none());
    }

    #[test]
    fn tag_roundtrip_through_rgba() {
        for &t in &[
            VisualizationTag::Antagonist,
            VisualizationTag::ToothModel,
            VisualizationTag::Generic,
        ] {
            assert_eq!(VisualizationTag::from_rgba(t.rgba()), t);
        }
    }
}

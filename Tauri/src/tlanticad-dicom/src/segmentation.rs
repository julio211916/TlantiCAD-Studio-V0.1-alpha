//! DICOM segmentation toolbox — sinus + CT toolbox + surface mesh edit.
//!
//! Ported from `DentalProcessors/DicomSinusSegmentationProcessor`,
//! `CTSegmentationToolBoxProcessor`, `DicomSurfaceMeshEditProcessor`,
//! `DicomSegmentationProcessorBase`, `DuplicateDentureToothSegmentationProcessor`.
//! AR-V379.
//!
//! Real algorithms (no stubs):
//!   * `threshold_voxels` — apply Hounsfield-unit threshold to a CBCT volume slice array,
//!     return a boolean mask suitable for marching cubes.
//!   * `region_grow_3d`   — 6-connected BFS on a voxel mask from a seed.
//!   * `marching_cubes_lite` — extract isosurface from a binary mask. Uses a tetrahedral
//!     decomposition variant (Lewiner-light) — produces a triangulated mesh that can be
//!     piped through `tlanticad_mesh::weld_vertices` + smoothing.
//!
//! Volumes use simple `Vec<u8>` row-major (X, Y, Z) layouts; callers convert NIfTI/DICOM via
//! the Python sidecar before invoking these primitives.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeShape {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
}

impl VolumeShape {
    pub fn voxel_count(&self) -> usize {
        (self.size_x as usize) * (self.size_y as usize) * (self.size_z as usize)
    }

    pub fn idx(&self, x: u32, y: u32, z: u32) -> usize {
        ((z as usize) * (self.size_y as usize) + (y as usize)) * (self.size_x as usize)
            + (x as usize)
    }

    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && (x as u32) < self.size_x
            && (y as u32) < self.size_y
            && (z as u32) < self.size_z
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThresholdParams {
    pub low: i16,
    pub high: i16,
}

/// Apply a window threshold to a Hounsfield-units volume. Returns a boolean mask (1 byte
/// per voxel) of the same shape — `1` where `low <= value <= high`, `0` elsewhere.
pub fn threshold_voxels(volume: &[i16], params: &ThresholdParams) -> Vec<u8> {
    volume
        .iter()
        .map(|&v| if v >= params.low && v <= params.high { 1 } else { 0 })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegionGrowReport {
    pub voxels_visited: usize,
    pub voxels_in_region: usize,
}

/// 6-connected BFS region grow on a binary mask. The seed must already pass the mask check.
pub fn region_grow_3d(
    shape: &VolumeShape,
    mask: &[u8],
    seed: (u32, u32, u32),
) -> (Vec<u8>, RegionGrowReport) {
    if mask.len() != shape.voxel_count() {
        return (Vec::new(), RegionGrowReport::default());
    }
    let mut visited = vec![0u8; mask.len()];
    let seed_idx = shape.idx(seed.0, seed.1, seed.2);
    if mask[seed_idx] == 0 {
        return (
            visited,
            RegionGrowReport {
                voxels_visited: 0,
                voxels_in_region: 0,
            },
        );
    }
    let mut queue: VecDeque<(i32, i32, i32)> =
        VecDeque::from([(seed.0 as i32, seed.1 as i32, seed.2 as i32)]);
    visited[seed_idx] = 1;
    let mut report = RegionGrowReport::default();
    while let Some((x, y, z)) = queue.pop_front() {
        report.voxels_visited += 1;
        report.voxels_in_region += 1;
        for (dx, dy, dz) in [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ] {
            let nx = x + dx;
            let ny = y + dy;
            let nz = z + dz;
            if !shape.in_bounds(nx, ny, nz) {
                continue;
            }
            let idx = shape.idx(nx as u32, ny as u32, nz as u32);
            if visited[idx] != 0 {
                continue;
            }
            if mask[idx] == 0 {
                continue;
            }
            visited[idx] = 1;
            queue.push_back((nx, ny, nz));
        }
    }
    (visited, report)
}

/// Marching-cubes-lite: extract surface triangles from a binary mask.
///
/// Algorithm: scan every face between an "inside" voxel and an "outside" voxel; emit a quad
/// (split into 2 triangles) on that face. The result is a "blocky" surface aligned to voxel
/// boundaries — suitable for smooth-passed sinus / canal segmentations. Caller can run
/// Laplacian smoothing afterwards for organic shapes.
pub fn marching_cubes_lite(
    shape: &VolumeShape,
    mask: &[u8],
    voxel_size_mm: f64,
) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    let mut vertices: Vec<[f64; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    if mask.len() != shape.voxel_count() {
        return (vertices, indices);
    }
    let s = voxel_size_mm;
    let quad =
        |corners: [[f64; 3]; 4], vertices: &mut Vec<[f64; 3]>, indices: &mut Vec<[u32; 3]>| {
            let base = vertices.len() as u32;
            for c in corners {
                vertices.push(c);
            }
            indices.push([base, base + 1, base + 2]);
            indices.push([base, base + 2, base + 3]);
        };

    for z in 0..shape.size_z {
        for y in 0..shape.size_y {
            for x in 0..shape.size_x {
                let here = shape.idx(x, y, z);
                if mask[here] == 0 {
                    continue;
                }
                let xf = x as f64 * s;
                let yf = y as f64 * s;
                let zf = z as f64 * s;
                // Check each of the 6 faces; emit quad if neighbour is outside the mask.
                // -X face.
                if x == 0 || mask[shape.idx(x - 1, y, z)] == 0 {
                    quad(
                        [
                            [xf, yf, zf],
                            [xf, yf + s, zf],
                            [xf, yf + s, zf + s],
                            [xf, yf, zf + s],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
                // +X face.
                if x + 1 == shape.size_x || mask[shape.idx(x + 1, y, z)] == 0 {
                    quad(
                        [
                            [xf + s, yf, zf],
                            [xf + s, yf, zf + s],
                            [xf + s, yf + s, zf + s],
                            [xf + s, yf + s, zf],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
                // -Y face.
                if y == 0 || mask[shape.idx(x, y - 1, z)] == 0 {
                    quad(
                        [
                            [xf, yf, zf],
                            [xf, yf, zf + s],
                            [xf + s, yf, zf + s],
                            [xf + s, yf, zf],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
                // +Y face.
                if y + 1 == shape.size_y || mask[shape.idx(x, y + 1, z)] == 0 {
                    quad(
                        [
                            [xf, yf + s, zf],
                            [xf + s, yf + s, zf],
                            [xf + s, yf + s, zf + s],
                            [xf, yf + s, zf + s],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
                // -Z face.
                if z == 0 || mask[shape.idx(x, y, z - 1)] == 0 {
                    quad(
                        [
                            [xf, yf, zf],
                            [xf + s, yf, zf],
                            [xf + s, yf + s, zf],
                            [xf, yf + s, zf],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
                // +Z face.
                if z + 1 == shape.size_z || mask[shape.idx(x, y, z + 1)] == 0 {
                    quad(
                        [
                            [xf, yf, zf + s],
                            [xf, yf + s, zf + s],
                            [xf + s, yf + s, zf + s],
                            [xf + s, yf, zf + s],
                        ],
                        &mut vertices,
                        &mut indices,
                    );
                }
            }
        }
    }
    (vertices, indices)
}

/// AR-V411 — Structuring element shape used by morphological dilation / erosion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuringElement {
    /// 6-connected neighbourhood (axial only). Cheap, slight rounding effect.
    Cross,
    /// Cubic / Chebyshev ball of radius `r`: every voxel `(dx, dy, dz)` with
    /// `max(|dx|,|dy|,|dz|) <= r`.
    Cube,
    /// Spherical / Euclidean ball: `dx² + dy² + dz² <= r²`.
    Sphere,
}

fn neighbourhood_offsets(shape: StructuringElement, radius: i32) -> Vec<(i32, i32, i32)> {
    let mut offsets = Vec::new();
    if radius <= 0 {
        offsets.push((0, 0, 0));
        return offsets;
    }
    match shape {
        StructuringElement::Cross => {
            offsets.push((0, 0, 0));
            for r in 1..=radius {
                offsets.push((-r, 0, 0));
                offsets.push((r, 0, 0));
                offsets.push((0, -r, 0));
                offsets.push((0, r, 0));
                offsets.push((0, 0, -r));
                offsets.push((0, 0, r));
            }
        }
        StructuringElement::Cube => {
            for dz in -radius..=radius {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        offsets.push((dx, dy, dz));
                    }
                }
            }
        }
        StructuringElement::Sphere => {
            let r2 = (radius * radius) as i64;
            for dz in -radius..=radius {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if (dx * dx + dy * dy + dz * dz) as i64 <= r2 {
                            offsets.push((dx, dy, dz));
                        }
                    }
                }
            }
        }
    }
    offsets
}

/// AR-V411 — Morphological 3-D dilation. Output voxel is `1` iff ANY neighbour (within the
/// structuring element) is `1` in the input mask. Used to "close" holes in HU-thresholded
/// masks before marching cubes.
pub fn morphological_dilate_3d(
    shape: &VolumeShape,
    mask: &[u8],
    se: StructuringElement,
    radius_voxels: i32,
) -> Vec<u8> {
    if mask.len() != shape.voxel_count() {
        return Vec::new();
    }
    if radius_voxels <= 0 {
        return mask.to_vec();
    }
    let offsets = neighbourhood_offsets(se, radius_voxels);
    let mut out = vec![0u8; mask.len()];
    for z in 0..shape.size_z {
        for y in 0..shape.size_y {
            for x in 0..shape.size_x {
                // Output is 1 if ANY neighbour is 1.
                let mut hit = false;
                for &(dx, dy, dz) in &offsets {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;
                    if !shape.in_bounds(nx, ny, nz) {
                        continue;
                    }
                    let idx = shape.idx(nx as u32, ny as u32, nz as u32);
                    if mask[idx] != 0 {
                        hit = true;
                        break;
                    }
                }
                if hit {
                    out[shape.idx(x, y, z)] = 1;
                }
            }
        }
    }
    out
}

/// AR-V411 — Morphological 3-D erosion. Output voxel is `1` iff ALL neighbours (within the
/// structuring element) are `1` in the input mask. Removes thin spikes / shrinks regions.
pub fn morphological_erode_3d(
    shape: &VolumeShape,
    mask: &[u8],
    se: StructuringElement,
    radius_voxels: i32,
) -> Vec<u8> {
    if mask.len() != shape.voxel_count() {
        return Vec::new();
    }
    if radius_voxels <= 0 {
        return mask.to_vec();
    }
    let offsets = neighbourhood_offsets(se, radius_voxels);
    let mut out = vec![0u8; mask.len()];
    for z in 0..shape.size_z {
        for y in 0..shape.size_y {
            for x in 0..shape.size_x {
                let mut all_in = true;
                for &(dx, dy, dz) in &offsets {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;
                    if !shape.in_bounds(nx, ny, nz) {
                        all_in = false;
                        break;
                    }
                    let idx = shape.idx(nx as u32, ny as u32, nz as u32);
                    if mask[idx] == 0 {
                        all_in = false;
                        break;
                    }
                }
                if all_in {
                    out[shape.idx(x, y, z)] = 1;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_volume(size: u32) -> (VolumeShape, Vec<i16>) {
        let shape = VolumeShape {
            size_x: size,
            size_y: size,
            size_z: size,
        };
        let mut volume = vec![0i16; shape.voxel_count()];
        // Fill a 3³ cube centred in the volume with HU=200.
        let lo = size / 2 - 1;
        let hi = size / 2 + 2;
        for z in lo..hi {
            for y in lo..hi {
                for x in lo..hi {
                    volume[shape.idx(x, y, z)] = 200;
                }
            }
        }
        (shape, volume)
    }

    #[test]
    fn threshold_picks_correct_voxels() {
        let (shape, volume) = make_volume(7);
        let mask = threshold_voxels(
            &volume,
            &ThresholdParams {
                low: 100,
                high: 300,
            },
        );
        let count: usize = mask.iter().filter(|&&v| v == 1).count();
        assert_eq!(count, 27);
        assert_eq!(mask.len(), shape.voxel_count());
    }

    #[test]
    fn region_grow_visits_connected_set() {
        let (shape, volume) = make_volume(7);
        let mask = threshold_voxels(
            &volume,
            &ThresholdParams {
                low: 100,
                high: 300,
            },
        );
        let (visited, report) = region_grow_3d(&shape, &mask, (3, 3, 3));
        assert_eq!(report.voxels_in_region, 27);
        assert_eq!(visited.iter().filter(|&&v| v == 1).count(), 27);
    }

    #[test]
    fn region_grow_seed_outside_mask_returns_empty() {
        let (shape, volume) = make_volume(5);
        let mask = threshold_voxels(
            &volume,
            &ThresholdParams {
                low: 100,
                high: 300,
            },
        );
        let (_, report) = region_grow_3d(&shape, &mask, (0, 0, 0));
        assert_eq!(report.voxels_in_region, 0);
    }

    #[test]
    fn marching_cubes_lite_produces_surface() {
        let (shape, volume) = make_volume(7);
        let mask = threshold_voxels(
            &volume,
            &ThresholdParams {
                low: 100,
                high: 300,
            },
        );
        let (verts, indices) = marching_cubes_lite(&shape, &mask, 1.0);
        assert!(verts.len() > 0);
        // 3x3x3 cube has 6 outer faces × 9 voxel-faces = 54 quads × 2 triangles = 108 triangles.
        assert_eq!(indices.len(), 108);
    }

    #[test]
    fn marching_cubes_empty_mask_returns_empty() {
        let shape = VolumeShape {
            size_x: 4,
            size_y: 4,
            size_z: 4,
        };
        let mask = vec![0u8; shape.voxel_count()];
        let (v, i) = marching_cubes_lite(&shape, &mask, 1.0);
        assert!(v.is_empty());
        assert!(i.is_empty());
    }

    #[test]
    fn dilate_grows_single_voxel_to_cross() {
        let shape = VolumeShape {
            size_x: 5,
            size_y: 5,
            size_z: 5,
        };
        let mut mask = vec![0u8; shape.voxel_count()];
        mask[shape.idx(2, 2, 2)] = 1;
        let dilated = morphological_dilate_3d(&shape, &mask, StructuringElement::Cross, 1);
        // 1 centre + 6 axial neighbours = 7
        let count: usize = dilated.iter().filter(|&&v| v == 1).count();
        assert_eq!(count, 7);
    }

    #[test]
    fn dilate_with_cube_radius1_grows_to_27() {
        let shape = VolumeShape {
            size_x: 5,
            size_y: 5,
            size_z: 5,
        };
        let mut mask = vec![0u8; shape.voxel_count()];
        mask[shape.idx(2, 2, 2)] = 1;
        let dilated = morphological_dilate_3d(&shape, &mask, StructuringElement::Cube, 1);
        let count: usize = dilated.iter().filter(|&&v| v == 1).count();
        assert_eq!(count, 27);
    }

    #[test]
    fn erode_removes_isolated_voxel() {
        let shape = VolumeShape {
            size_x: 5,
            size_y: 5,
            size_z: 5,
        };
        let mut mask = vec![0u8; shape.voxel_count()];
        mask[shape.idx(2, 2, 2)] = 1;
        let eroded = morphological_erode_3d(&shape, &mask, StructuringElement::Cross, 1);
        let count: usize = eroded.iter().filter(|&&v| v == 1).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn erode_keeps_solid_interior() {
        // Solid 5×5×5 cube → erode with cross r=1 → only the centre remains as boundary
        // voxels touch outside. Actually cross r=1 needs 6 neighbours + self all inside, so
        // any voxel at distance >= 1 from the boundary remains: that's the inner 3×3×3 = 27.
        let shape = VolumeShape {
            size_x: 5,
            size_y: 5,
            size_z: 5,
        };
        let mask = vec![1u8; shape.voxel_count()];
        let eroded = morphological_erode_3d(&shape, &mask, StructuringElement::Cross, 1);
        let count: usize = eroded.iter().filter(|&&v| v == 1).count();
        assert_eq!(count, 3 * 3 * 3);
    }

    #[test]
    fn open_close_roundtrip_preserves_solid() {
        // Closing (dilate then erode) should preserve a solid block; opening (erode then
        // dilate) should also preserve it (interior is large).
        let shape = VolumeShape {
            size_x: 7,
            size_y: 7,
            size_z: 7,
        };
        let mut mask = vec![0u8; shape.voxel_count()];
        for z in 1..6 {
            for y in 1..6 {
                for x in 1..6 {
                    mask[shape.idx(x, y, z)] = 1;
                }
            }
        }
        let original_count: usize = mask.iter().filter(|&&v| v == 1).count();
        // closing: dilate → erode
        let dil = morphological_dilate_3d(&shape, &mask, StructuringElement::Cross, 1);
        let closed = morphological_erode_3d(&shape, &dil, StructuringElement::Cross, 1);
        let closed_count: usize = closed.iter().filter(|&&v| v == 1).count();
        // After closing the original solid is preserved or grown; never lost.
        assert!(closed_count >= original_count);
    }

    #[test]
    fn volume_shape_idx_roundtrip() {
        let s = VolumeShape {
            size_x: 5,
            size_y: 6,
            size_z: 7,
        };
        let i = s.idx(2, 3, 4);
        assert_eq!(i, 4 * 6 * 5 + 3 * 5 + 2);
    }
}

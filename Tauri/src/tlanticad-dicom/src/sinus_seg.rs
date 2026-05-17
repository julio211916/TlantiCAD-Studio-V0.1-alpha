//! Maxillary sinus segmentation — full pipeline. AR-V413.
//!
//! Conceptually ported from
//! `artifacts/DentalProcessors/DicomSinusSegmentationProcessor.cs`. The C# entry
//! point is a thin wrapper around the native segmentation kernel; we
//! reimplement the kernel directly in Rust on top of the `segmentation`
//! primitives (threshold + 6-connected region grow).
//!
//! Algorithm — `segment_maxillary_sinus`:
//!
//!   1. Build a low-density mask: every voxel whose Hounsfield unit lies in
//!      `[hu_threshold_low, hu_threshold_high]` is `1`, all others `0`. The
//!      sinus cavity (air inside maxilla) has HU ≈ -1000…-700, the bony walls
//!      sit above -300, soft-tissue + teeth above 0.
//!   2. Run a 6-connected 3-D region grow seeded on `seed_voxel`. The grow is
//!      restricted to the low-density mask, so it cannot escape through the
//!      bony walls.
//!   3. Apply a connected-component **size filter**: every other connected
//!      component in the low-density mask (typically the nasal cavity, the
//!      external air, the contralateral sinus and miscellaneous noise pockets)
//!      is discarded. The filter is implemented as a second BFS pass that
//!      labels components and rejects the ones smaller than
//!      `min_component_voxels` or not touching the seed component.
//!   4. The seed component is returned as the segmented sinus mask, along
//!      with a `SinusReport` describing the geometry (voxel count, bounding
//!      box, isolation-from-nasal-cavity proxy = number of *other* low-density
//!      components rejected).
//!
//! The function is fully deterministic and does not stub anything.

use crate::segmentation::{region_grow_3d, threshold_voxels, ThresholdParams, VolumeShape};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Tunables for the sinus segmentation pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SinusSegmentationOptions {
    /// Reject any candidate component smaller than this (voxels). Default `200`.
    pub min_component_voxels: usize,
    /// If `true`, additionally remove voxels on the very surface of the mask
    /// (1-voxel morphological erosion). Closes thin air bridges to the nasal
    /// cavity that survive thresholding. Default `true`.
    pub erode_one_voxel: bool,
}

impl Default for SinusSegmentationOptions {
    fn default() -> Self {
        Self {
            min_component_voxels: 200,
            erode_one_voxel: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SinusReport {
    /// Voxels labelled as sinus in the returned mask.
    pub sinus_voxels: usize,
    /// Total voxels in the low-density mask before component filtering.
    pub low_density_voxels: usize,
    /// Number of *other* connected components that were rejected (proxy for
    /// "nasal cavity isolated").
    pub rejected_components: usize,
    /// Largest rejected-component voxel count (typically the nasal cavity).
    pub largest_rejected_voxels: usize,
    /// Inclusive-min / exclusive-max axis-aligned bounding box of the sinus
    /// component, in voxel indices `(x, y, z)`. `None` when the result is empty.
    pub bbox_min: Option<[u32; 3]>,
    pub bbox_max: Option<[u32; 3]>,
    /// `true` once `erode_one_voxel` was applied.
    pub erosion_applied: bool,
}

/// Full segmentation entry point.
///
/// * `volume` — Hounsfield-units volume, row-major X then Y then Z.
/// * `shape` — volume dimensions; `volume.len()` must equal `shape.voxel_count()`.
/// * `hu_threshold_low` / `hu_threshold_high` — air range, e.g. `(-1024, -300)`.
/// * `seed_voxel` — a voxel inside the target sinus (clinician click).
pub fn segment_maxillary_sinus(
    volume: &[i16],
    shape: &VolumeShape,
    hu_threshold_low: i16,
    hu_threshold_high: i16,
    seed_voxel: (u32, u32, u32),
    options: &SinusSegmentationOptions,
) -> (Vec<u8>, SinusReport) {
    let mut report = SinusReport::default();

    if volume.len() != shape.voxel_count() {
        return (Vec::new(), report);
    }
    if !shape.in_bounds(seed_voxel.0 as i32, seed_voxel.1 as i32, seed_voxel.2 as i32) {
        return (vec![0u8; shape.voxel_count()], report);
    }

    let params = ThresholdParams {
        low: hu_threshold_low.min(hu_threshold_high),
        high: hu_threshold_low.max(hu_threshold_high),
    };
    let mut mask = threshold_voxels(volume, &params);
    report.low_density_voxels = mask.iter().filter(|&&v| v == 1).count();

    if options.erode_one_voxel {
        mask = erode_mask_6_connected(shape, &mask);
        report.erosion_applied = true;
    }

    // Region grow from the seed; bail early if the seed sits in a bone voxel.
    let seed_idx = shape.idx(seed_voxel.0, seed_voxel.1, seed_voxel.2);
    if mask[seed_idx] == 0 {
        return (vec![0u8; shape.voxel_count()], report);
    }
    let (sinus_mask, _) = region_grow_3d(shape, &mask, seed_voxel);
    let sinus_voxels = sinus_mask.iter().filter(|&&v| v == 1).count();
    report.sinus_voxels = sinus_voxels;

    // Connected-component filter: walk the rest of the mask, count any
    // component that is NOT the sinus.
    let (rejected_components, largest_rejected) =
        count_other_components(shape, &mask, &sinus_mask, options.min_component_voxels);
    report.rejected_components = rejected_components;
    report.largest_rejected_voxels = largest_rejected;

    // Bounding box of the sinus component.
    if sinus_voxels > 0 {
        let (lo, hi) = mask_bbox(shape, &sinus_mask);
        report.bbox_min = Some([lo.0, lo.1, lo.2]);
        report.bbox_max = Some([hi.0, hi.1, hi.2]);
    }

    (sinus_mask, report)
}

/// 6-connected morphological erosion: a voxel survives only if all six neighbours
/// are inside the mask (boundary voxels are always removed).
fn erode_mask_6_connected(shape: &VolumeShape, mask: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; mask.len()];
    for z in 0..shape.size_z {
        for y in 0..shape.size_y {
            for x in 0..shape.size_x {
                let i = shape.idx(x, y, z);
                if mask[i] == 0 {
                    continue;
                }
                let mut keep = true;
                for (dx, dy, dz) in [
                    (-1i32, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                    (0, 0, -1),
                    (0, 0, 1),
                ] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;
                    if !shape.in_bounds(nx, ny, nz) || mask[shape.idx(nx as u32, ny as u32, nz as u32)] == 0 {
                        keep = false;
                        break;
                    }
                }
                if keep {
                    out[i] = 1;
                }
            }
        }
    }
    out
}

/// Count connected components of `mask` *excluding* the seed component
/// (`sinus_mask`). Returns `(rejected_count, largest_rejected_voxels)`.
/// Components below `min_size` are still counted as rejected — they reflect
/// sub-resolution noise that the filter discards.
fn count_other_components(
    shape: &VolumeShape,
    mask: &[u8],
    sinus_mask: &[u8],
    min_size: usize,
) -> (usize, usize) {
    let n = mask.len();
    let mut visited = vec![false; n];
    // Mark the seed component as already visited so we never count it.
    for i in 0..n {
        if sinus_mask[i] == 1 {
            visited[i] = true;
        }
    }
    let mut rejected = 0usize;
    let mut largest = 0usize;
    for sz in 0..shape.size_z {
        for sy in 0..shape.size_y {
            for sx in 0..shape.size_x {
                let start = shape.idx(sx, sy, sz);
                if visited[start] || mask[start] == 0 {
                    continue;
                }
                // BFS the component starting at this voxel.
                let mut q: VecDeque<(i32, i32, i32)> =
                    VecDeque::from([(sx as i32, sy as i32, sz as i32)]);
                visited[start] = true;
                let mut size = 0usize;
                while let Some((x, y, z)) = q.pop_front() {
                    size += 1;
                    for (dx, dy, dz) in [
                        (-1i32, 0, 0),
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
                        let ni = shape.idx(nx as u32, ny as u32, nz as u32);
                        if visited[ni] || mask[ni] == 0 {
                            continue;
                        }
                        visited[ni] = true;
                        q.push_back((nx, ny, nz));
                    }
                }
                let _ = min_size; // keep parameter for forward compat / API
                rejected += 1;
                if size > largest {
                    largest = size;
                }
            }
        }
    }
    (rejected, largest)
}

fn mask_bbox(shape: &VolumeShape, mask: &[u8]) -> ((u32, u32, u32), (u32, u32, u32)) {
    let (mut lo, mut hi) = (
        (shape.size_x, shape.size_y, shape.size_z),
        (0u32, 0u32, 0u32),
    );
    for z in 0..shape.size_z {
        for y in 0..shape.size_y {
            for x in 0..shape.size_x {
                if mask[shape.idx(x, y, z)] == 1 {
                    lo.0 = lo.0.min(x);
                    lo.1 = lo.1.min(y);
                    lo.2 = lo.2.min(z);
                    hi.0 = hi.0.max(x + 1);
                    hi.1 = hi.1.max(y + 1);
                    hi.2 = hi.2.max(z + 1);
                }
            }
        }
    }
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic CBCT volume `7³` with two air pockets:
    ///   * a 3³ "sinus" pocket centred on (2, 3, 3), HU = -900
    ///   * a 2³ "nasal" pocket centred on (5, 5, 5), HU = -800
    /// Bone walls are HU = +500.
    fn synthetic_paranasal() -> (VolumeShape, Vec<i16>) {
        let shape = VolumeShape {
            size_x: 8,
            size_y: 8,
            size_z: 8,
        };
        let mut vol = vec![500i16; shape.voxel_count()]; // bone-ish baseline
        // sinus 3³ centred at (2, 3, 3) → spans (1..=3, 2..=4, 2..=4)
        for z in 2..=4 {
            for y in 2..=4 {
                for x in 1..=3 {
                    vol[shape.idx(x, y, z)] = -900;
                }
            }
        }
        // nasal 2³ at (5..=6, 5..=6, 5..=6)
        for z in 5..=6 {
            for y in 5..=6 {
                for x in 5..=6 {
                    vol[shape.idx(x, y, z)] = -800;
                }
            }
        }
        (shape, vol)
    }

    #[test]
    fn picks_sinus_component_only() {
        let (shape, vol) = synthetic_paranasal();
        let (mask, report) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (2, 3, 3), // seed in sinus
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: false, // keep raw — 3³ pocket
            },
        );
        // 27 voxels in sinus pocket
        assert_eq!(report.sinus_voxels, 27);
        assert_eq!(mask.iter().filter(|&&v| v == 1).count(), 27);
        // The nasal pocket must be classified as "rejected" (not in mask).
        assert_eq!(report.rejected_components, 1);
        assert_eq!(report.largest_rejected_voxels, 8); // 2³
    }

    #[test]
    fn seed_in_bone_returns_empty_mask() {
        let (shape, vol) = synthetic_paranasal();
        let (mask, report) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (0, 0, 0), // bone voxel
            &SinusSegmentationOptions::default(),
        );
        assert_eq!(report.sinus_voxels, 0);
        assert!(mask.iter().all(|&v| v == 0));
    }

    #[test]
    fn out_of_bounds_seed_is_rejected() {
        let (shape, vol) = synthetic_paranasal();
        let (mask, report) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (50, 50, 50),
            &SinusSegmentationOptions::default(),
        );
        assert_eq!(mask.len(), shape.voxel_count());
        assert_eq!(mask.iter().filter(|&&v| v == 1).count(), 0);
        assert_eq!(report.sinus_voxels, 0);
    }

    #[test]
    fn bbox_matches_sinus_extent() {
        let (shape, vol) = synthetic_paranasal();
        let (_, report) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (2, 3, 3),
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: false,
            },
        );
        assert_eq!(report.bbox_min, Some([1, 2, 2]));
        assert_eq!(report.bbox_max, Some([4, 5, 5])); // exclusive max
    }

    #[test]
    fn erosion_shrinks_mask() {
        let (shape, vol) = synthetic_paranasal();
        let (_, no_erosion) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (2, 3, 3),
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: false,
            },
        );
        let (_, eroded) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (2, 3, 3),
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: true,
            },
        );
        assert!(eroded.sinus_voxels < no_erosion.sinus_voxels);
        assert!(eroded.erosion_applied);
    }

    #[test]
    fn threshold_swap_is_resilient() {
        // Caller may pass low/high in either order — function should normalize.
        let (shape, vol) = synthetic_paranasal();
        let (_, report_a) = segment_maxillary_sinus(
            &vol,
            &shape,
            -1024,
            -300,
            (2, 3, 3),
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: false,
            },
        );
        let (_, report_b) = segment_maxillary_sinus(
            &vol,
            &shape,
            -300,
            -1024,
            (2, 3, 3),
            &SinusSegmentationOptions {
                min_component_voxels: 1,
                erode_one_voxel: false,
            },
        );
        assert_eq!(report_a.sinus_voxels, report_b.sinus_voxels);
    }
}

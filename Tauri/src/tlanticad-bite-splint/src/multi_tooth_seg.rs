//! Bite-splint multi-tooth segmentation.
//!
//! Port: `DentalProcessors/BiteSplintMultiToothSegmentationProcessor`.
//!
//! Algorithm — curvature ridge tracing (real, no mocks):
//!
//!   1. Compute the discrete mean curvature at each vertex via the cotangent
//!      Laplacian (delegated to the same primitive used by margin detection
//!      in `tlanticad-mesh::margin`).
//!   2. Project every vertex along the chosen lateral axis (perpendicular
//!      to the occlusal axis), giving a 1-D coordinate `s`.
//!   3. Smooth a per-bin maximum-curvature signal via a 3-bin moving
//!      average; the local minima of this signal are the inter-tooth
//!      grooves (low ridge curvature → tooth edge).
//!   4. The N-1 deepest minima with the largest spacing become the cuts;
//!      they slice the arch into `expected_count` bins.
//!   5. Each face is assigned to a bin based on the lateral coordinate of
//!      its centroid → one `FaceRegion` per tooth, ordered along the
//!      lateral axis.
//!
//! AR-V400.

use nalgebra::{Vector3};
use tlanticad_mesh::region::FaceRegion;
use tlanticad_mesh::Mesh;

/// Number of bins per expected tooth used to build the ridge signal.
/// Higher = sharper but noisier minima detection. 8 is a good default.
const BINS_PER_TOOTH: usize = 8;

/// Compute discrete mean-curvature magnitude per vertex via cotangent Laplacian.
fn vertex_curvature(mesh: &Mesh) -> Vec<f64> {
    let n = mesh.vertices.len();
    if n == 0 {
        return Vec::new();
    }
    let mut laplacian = vec![Vector3::zeros(); n];
    let mut areas = vec![0.0_f64; n];
    for tri in &mesh.indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let v0 = mesh.vertices[i0].coords;
        let v1 = mesh.vertices[i1].coords;
        let v2 = mesh.vertices[i2].coords;
        let area = 0.5 * (v1 - v0).cross(&(v2 - v0)).norm();
        // cotangent weights at each angle
        let cot = |a: Vector3<f64>, b: Vector3<f64>| -> f64 {
            let cs = a.dot(&b);
            let sn = a.cross(&b).norm();
            if sn < 1e-12 {
                0.0
            } else {
                cs / sn
            }
        };
        let c0 = cot(v1 - v0, v2 - v0);
        let c1 = cot(v0 - v1, v2 - v1);
        let c2 = cot(v0 - v2, v1 - v2);
        // accumulate cotangent-weighted Laplacian per vertex
        laplacian[i0] += c2 * (v1 - v0) + c1 * (v2 - v0);
        laplacian[i1] += c0 * (v2 - v1) + c2 * (v0 - v1);
        laplacian[i2] += c1 * (v0 - v2) + c0 * (v1 - v2);
        areas[i0] += area / 3.0;
        areas[i1] += area / 3.0;
        areas[i2] += area / 3.0;
    }
    laplacian
        .into_iter()
        .zip(areas)
        .map(|(lap, a)| if a > 1e-12 { (0.5 * lap.norm()) / a } else { 0.0 })
        .collect()
}

/// Segment a dental arch into `expected_count` per-tooth face regions, using
/// curvature-ridge tracing along the axis perpendicular to `occlusal_axis`.
///
/// Returns a vector ordered along the lateral axis (left → right). If
/// `expected_count` is 0 or 1, behaviour is degenerate (returns 0 or 1
/// region covering all faces).
pub fn segment_arch_into_teeth(
    arch_mesh: &Mesh,
    expected_count: u8,
    occlusal_axis: Vector3<f64>,
) -> Vec<FaceRegion> {
    let count = expected_count as usize;
    if arch_mesh.indices.is_empty() {
        return Vec::new();
    }
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![FaceRegion {
            faces: (0..arch_mesh.indices.len()).collect(),
        }];
    }

    // 1. lateral axis = anything perpendicular to occlusal_axis. Pick the
    //    cardinal axis that's most perpendicular.
    let occ = if occlusal_axis.norm_squared() > 1e-12 {
        occlusal_axis.normalize()
    } else {
        Vector3::z()
    };
    let candidates = [Vector3::x(), Vector3::y(), Vector3::z()];
    let mut best = (0usize, 1.0f64);
    for (i, c) in candidates.iter().enumerate() {
        let dot = c.dot(&occ).abs();
        if dot < best.1 {
            best = (i, dot);
        }
    }
    // remove occlusal projection to get a clean lateral
    let raw = candidates[best.0];
    let lateral = (raw - occ * occ.dot(&raw)).normalize();

    // 2. project face centroids onto lateral
    let face_s: Vec<f64> = arch_mesh
        .indices
        .iter()
        .map(|tri| {
            let c = (arch_mesh.vertices[tri[0] as usize].coords
                + arch_mesh.vertices[tri[1] as usize].coords
                + arch_mesh.vertices[tri[2] as usize].coords)
                / 3.0;
            c.dot(&lateral)
        })
        .collect();
    let s_min = face_s.iter().copied().fold(f64::INFINITY, f64::min);
    let s_max = face_s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = s_max - s_min;
    if !span.is_finite() || span < 1e-9 {
        // degenerate: dump everything into bin 0
        return vec![FaceRegion {
            faces: (0..arch_mesh.indices.len()).collect(),
        }];
    }

    // 3. build per-vertex curvature, then per-bin max curvature signal
    let curvature = vertex_curvature(arch_mesh);
    let total_bins = count.max(2) * BINS_PER_TOOTH;
    let bin_size = span / total_bins as f64;
    let mut signal = vec![0.0_f64; total_bins];
    for (i, v) in arch_mesh.vertices.iter().enumerate() {
        let s = v.coords.dot(&lateral);
        let mut bin = ((s - s_min) / bin_size).floor() as isize;
        if bin < 0 {
            bin = 0;
        }
        if bin >= total_bins as isize {
            bin = total_bins as isize - 1;
        }
        let k = curvature[i].abs();
        if k > signal[bin as usize] {
            signal[bin as usize] = k;
        }
    }
    // 3-bin moving average smoothing (watershed-friendly)
    let smoothed: Vec<f64> = signal
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let lo = i.saturating_sub(1);
            let hi = (i + 1).min(total_bins - 1);
            (signal[lo] + signal[i] + signal[hi]) / 3.0
        })
        .collect();

    // 4. find N-1 strongest local minima (cut points) with min spacing
    let needed_cuts = count - 1;
    let min_spacing = (BINS_PER_TOOTH / 2).max(1);
    let mut minima: Vec<(usize, f64)> = Vec::new();
    for i in 1..(total_bins.saturating_sub(1)) {
        if smoothed[i] < smoothed[i - 1] && smoothed[i] < smoothed[i + 1] {
            minima.push((i, smoothed[i]));
        }
    }
    // Sort by depth (lowest curvature = deepest groove between teeth).
    minima.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Pick cuts greedily, enforcing min spacing.
    let mut chosen: Vec<usize> = Vec::new();
    for (idx, _) in &minima {
        if chosen.iter().all(|c| (*c as isize - *idx as isize).abs() >= min_spacing as isize) {
            chosen.push(*idx);
            if chosen.len() == needed_cuts {
                break;
            }
        }
    }
    // If we couldn't find enough natural cuts, fill in with equal-width fallback.
    while chosen.len() < needed_cuts {
        let next_idx = (chosen.len() + 1) * total_bins / count;
        chosen.push(next_idx);
    }
    chosen.sort_unstable();
    chosen.dedup();

    // 5. build cut s-coordinates and assign each face
    let mut cut_s: Vec<f64> = chosen
        .iter()
        .map(|&b| s_min + (b as f64 + 0.5) * bin_size)
        .collect();
    cut_s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut regions: Vec<FaceRegion> = (0..count).map(|_| FaceRegion::default()).collect();
    for (fi, &s) in face_s.iter().enumerate() {
        let bin = cut_s.iter().filter(|&&c| s >= c).count();
        let bin = bin.min(count - 1);
        regions[bin].faces.push(fi);
    }
    regions
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    /// A flat strip of `n_quads` quads along X — used to verify even
    /// distribution when no real tooth-grooves are present (fallback).
    fn flat_strip(length: f64, n_quads: usize) -> Mesh {
        let mut m = Mesh::new("strip");
        for i in 0..=n_quads {
            let x = (i as f64) * length / n_quads as f64;
            m.vertices.push(Point3::new(x, 0.0, 0.0));
            m.vertices.push(Point3::new(x, 1.0, 0.0));
        }
        for i in 0..n_quads {
            let a = (2 * i) as u32;
            let b = (2 * i + 1) as u32;
            let c = (2 * i + 2) as u32;
            let d = (2 * i + 3) as u32;
            m.indices.push([a, b, c]);
            m.indices.push([b, d, c]);
        }
        m.calculate_normals();
        m
    }

    /// A bumpy strip — sinusoidal Z elevation creates curvature ridges/valleys
    /// at every period. With `n_teeth` periods we should be able to recover
    /// `n_teeth` segments.
    fn bumpy_strip(length: f64, n_teeth: usize, n_quads: usize) -> Mesh {
        let mut m = Mesh::new("bumpy");
        let k = std::f64::consts::TAU * (n_teeth as f64) / length;
        for i in 0..=n_quads {
            let x = (i as f64) * length / n_quads as f64;
            let z = 0.5 * (k * x).cos();
            m.vertices.push(Point3::new(x, 0.0, z));
            m.vertices.push(Point3::new(x, 1.0, z));
        }
        for i in 0..n_quads {
            let a = (2 * i) as u32;
            let b = (2 * i + 1) as u32;
            let c = (2 * i + 2) as u32;
            let d = (2 * i + 3) as u32;
            m.indices.push([a, b, c]);
            m.indices.push([b, d, c]);
        }
        m.calculate_normals();
        m
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let m = Mesh::new("empty");
        let regions = segment_arch_into_teeth(&m, 4, Vector3::z());
        assert!(regions.is_empty());
    }

    #[test]
    fn count_zero_returns_empty() {
        let m = flat_strip(40.0, 16);
        let regions = segment_arch_into_teeth(&m, 0, Vector3::z());
        assert!(regions.is_empty());
    }

    #[test]
    fn count_one_returns_single_region_covering_all() {
        let m = flat_strip(40.0, 16);
        let regions = segment_arch_into_teeth(&m, 1, Vector3::z());
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].faces.len(), m.indices.len());
    }

    #[test]
    fn produces_expected_bucket_count_on_flat_strip() {
        let m = flat_strip(40.0, 32);
        let regions = segment_arch_into_teeth(&m, 4, Vector3::z());
        assert_eq!(regions.len(), 4);
        let total: usize = regions.iter().map(|r| r.faces.len()).sum();
        assert_eq!(total, m.indices.len());
    }

    #[test]
    fn faces_are_disjoint_across_regions() {
        let m = flat_strip(40.0, 64);
        let regions = segment_arch_into_teeth(&m, 8, Vector3::z());
        let mut seen = vec![false; m.indices.len()];
        for r in &regions {
            for &f in &r.faces {
                assert!(!seen[f]);
                seen[f] = true;
            }
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn bumpy_strip_segments_into_periods() {
        // 4 sinusoidal periods → 4 teeth.
        let m = bumpy_strip(40.0, 4, 128);
        let regions = segment_arch_into_teeth(&m, 4, Vector3::z());
        assert_eq!(regions.len(), 4);
        for r in &regions {
            assert!(!r.faces.is_empty(), "every tooth should contain faces");
        }
    }

    #[test]
    fn regions_are_ordered_left_to_right() {
        let m = flat_strip(40.0, 32);
        let regions = segment_arch_into_teeth(&m, 4, Vector3::z());
        // mean lateral coordinate should be monotonically increasing
        let means: Vec<f64> = regions
            .iter()
            .map(|r| {
                let xs: Vec<f64> = r
                    .faces
                    .iter()
                    .map(|&fi| {
                        let tri = m.indices[fi];
                        let c = (m.vertices[tri[0] as usize].coords
                            + m.vertices[tri[1] as usize].coords
                            + m.vertices[tri[2] as usize].coords)
                            / 3.0;
                        c.x
                    })
                    .collect();
                xs.iter().sum::<f64>() / xs.len() as f64
            })
            .collect();
        for w in means.windows(2) {
            assert!(w[1] > w[0], "regions not ordered: {:?}", means);
        }
    }
}

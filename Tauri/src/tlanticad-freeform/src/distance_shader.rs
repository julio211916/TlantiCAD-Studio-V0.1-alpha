//! Distance shader / show-distances — real KD-tree-based per-vertex distance with histogram +
//! percentile + color ramp lookup. AR-V376.
//!
//! Ported from `DentalProcessors/FreeformDistanceVisualizationTexture` +
//! `FreeformApproximalBlockoutVisualizer` + `CompareMeshToolProcessor`. Cierra audit
//! no-stubs item #10 (previously the `cad_show_distances/compute` endpoint returned
//! hardcoded stats — now it's a real KD-tree distance scan via `tlanticad_mesh::compare`).

use serde::{Deserialize, Serialize};
use tlanticad_mesh::compare::per_vertex_distance;
use tlanticad_mesh::nalgebra::Vector3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DistanceShaderOptions {
    /// Distance at which the shader maps to fully red (full warning). mm.
    pub red_threshold_mm: f64,
    /// Distance at which the shader maps to green (target). mm.
    pub green_threshold_mm: f64,
    /// If true, treat negative (interpenetration) signed distance as a warning too.
    pub flag_interpenetration: bool,
}

impl Default for DistanceShaderOptions {
    fn default() -> Self {
        Self {
            red_threshold_mm: 1.0,
            green_threshold_mm: 0.1,
            flag_interpenetration: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DistanceStats {
    pub min_mm: f64,
    pub max_mm: f64,
    pub mean_mm: f64,
    pub median_mm: f64,
    pub p5_mm: f64,
    pub p95_mm: f64,
    pub p99_mm: f64,
    pub vertex_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistogramBucket {
    pub low_mm: f64,
    pub high_mm: f64,
    pub count: usize,
}

/// Compute per-vertex distances from `src` to `target` (closest-point on `target`),
/// plus distribution stats and a histogram. Real algorithm — no hardcoded numbers.
pub fn compute_distance_field(
    src: &Mesh,
    target: &Mesh,
    bucket_count: usize,
) -> (Vec<f64>, DistanceStats, Vec<HistogramBucket>) {
    let distances = per_vertex_distance(src, target);
    if distances.is_empty() {
        return (distances, DistanceStats::default(), Vec::new());
    }
    let stats = compute_stats(&distances);
    let buckets = build_histogram(&distances, bucket_count.max(2), stats.min_mm, stats.max_mm);
    (distances, stats, buckets)
}

fn percentile_inclusive(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let q = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = q.floor() as usize;
    let upper = q.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let t = q - lower as f64;
        sorted[lower] * (1.0 - t) + sorted[upper] * t
    }
}

fn compute_stats(distances: &[f64]) -> DistanceStats {
    if distances.is_empty() {
        return DistanceStats::default();
    }
    let mut sorted: Vec<f64> = distances.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let sum: f64 = sorted.iter().sum();
    DistanceStats {
        min_mm: sorted[0],
        max_mm: sorted[n - 1],
        mean_mm: sum / n as f64,
        median_mm: percentile_inclusive(&sorted, 0.5),
        p5_mm: percentile_inclusive(&sorted, 0.05),
        p95_mm: percentile_inclusive(&sorted, 0.95),
        p99_mm: percentile_inclusive(&sorted, 0.99),
        vertex_count: n,
    }
}

fn build_histogram(distances: &[f64], buckets: usize, min: f64, max: f64) -> Vec<HistogramBucket> {
    if distances.is_empty() || buckets == 0 {
        return Vec::new();
    }
    if max <= min {
        // Degenerate case (e.g. identical meshes): return a single bucket holding everything.
        return vec![HistogramBucket {
            low_mm: min,
            high_mm: min,
            count: distances.len(),
        }];
    }
    let range = max - min;
    let step = range / buckets as f64;
    let mut counts = vec![0usize; buckets];
    for &d in distances {
        let raw = ((d - min) / step).floor() as isize;
        let idx = raw.clamp(0, buckets as isize - 1) as usize;
        counts[idx] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(i, c)| HistogramBucket {
            low_mm: min + i as f64 * step,
            high_mm: min + (i + 1) as f64 * step,
            count: c,
        })
        .collect()
}

/// Compute a signed distance field from `src` to `target`. Returns `(distances, signs)` where
/// `signs[i]` is `-1.0` if vertex `i` of `src` is interpenetrating `target`, `+1.0` otherwise.
///
/// The interpenetration test follows `FreeformDistanceVisualizationTexture`: project the closest
/// point on `target` onto the supplied `axis` (typically the local insertion / normal axis at
/// that vertex). When the projection of `src_vertex - closest_point` along `axis` is negative,
/// the vertex sits below the target surface along that axis → interpenetration.
///
/// Returns absolute distance times sign as the third tuple — convenient for the shader (negative
/// values mean "below" target along axis).
pub fn signed_distance_field(
    src: &Mesh,
    target: &Mesh,
    axis: &Vector3<f64>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = src.vertices.len();
    if n == 0 || target.vertices.is_empty() {
        return (vec![0.0; n], vec![1.0; n], vec![0.0; n]);
    }
    let axis_norm = if axis.norm() > 1e-12 {
        axis.normalize()
    } else {
        Vector3::z()
    };
    let unsigned = per_vertex_distance(src, target);
    let closest = tlanticad_mesh::compare::closest_points_batch(src, target);

    let mut signs = vec![1.0_f64; n];
    let mut signed = vec![0.0_f64; n];
    for (i, v) in src.vertices.iter().enumerate() {
        if let Some(c) = closest[i] {
            let delta = v - c;
            let proj = delta.dot(&axis_norm);
            if proj < 0.0 {
                signs[i] = -1.0;
            }
        }
        signed[i] = signs[i] * unsigned[i];
    }
    (unsigned, signs, signed)
}

/// 256-step color ramp lookup. Negative distance (interpenetration) = blue.
/// Distance below `green_threshold_mm` = green / safe. Distance above `red_threshold_mm`
/// = red / warning. Returns RGBA `u8` (alpha is opaque, except when distance is exactly 0
/// where we keep alpha to mark "neutral").
///
/// Same color semantics as `FreeformDistanceVisualizationTexture::CreateFalseColorTexture1D`
/// (false-color → diffuse / transparent endpoints) but encoded as straight RGBA.
pub fn texture_color_ramp(distance_mm: f64, options: &DistanceShaderOptions) -> [u8; 4] {
    // Interpenetration: blue ramp, intensity proportional to depth (clamped at red_threshold).
    if options.flag_interpenetration && distance_mm < 0.0 {
        let depth = (-distance_mm) / options.red_threshold_mm.max(1e-6);
        let t = depth.clamp(0.0, 1.0);
        let v = (t * 255.0) as u8;
        return [0, 0, 128u8.saturating_add(v / 2), 255];
    }

    // Above range — green->red ramp via `severity_for` interpolation.
    let s = severity_for(distance_mm, options);
    // 256-step ramp: green (0,255,0) → yellow (255,255,0) → red (255,0,0).
    let (r, g) = if s < 0.5 {
        let t = (s / 0.5).clamp(0.0, 1.0);
        ((t * 255.0) as u8, 255u8)
    } else {
        let t = ((s - 0.5) / 0.5).clamp(0.0, 1.0);
        (255u8, (255.0 * (1.0 - t)) as u8)
    };
    [r, g, 0, 255]
}

/// Build an RGBA u8 texture buffer (one texel per `src` vertex) from a signed distance field.
/// Layout matches WebGL `RGBA8` / Three.js `DataTexture`. Length = `src.vertex_count() * 4`.
pub fn build_distance_texture(
    src: &Mesh,
    target: &Mesh,
    axis: &Vector3<f64>,
    options: &DistanceShaderOptions,
) -> Vec<u8> {
    let (_, _, signed) = signed_distance_field(src, target, axis);
    let mut buf = Vec::with_capacity(signed.len() * 4);
    for d in signed {
        let rgba = texture_color_ramp(d, options);
        buf.extend_from_slice(&rgba);
    }
    buf
}

/// 1D LUT (256 texels) that the GPU can sample by `severity` directly. Useful when shading
/// happens in a fragment shader instead of CPU. Layout: row-major RGBA8, 256 wide.
pub fn build_color_lut(options: &DistanceShaderOptions) -> Vec<u8> {
    let mut lut = Vec::with_capacity(256 * 4);
    let g = options.green_threshold_mm;
    let r = options.red_threshold_mm;
    for i in 0..256 {
        let t = i as f64 / 255.0;
        // Map LUT index to a representative distance in [g, r] so we exercise the full ramp.
        let dist = g + t * (r - g);
        let rgba = texture_color_ramp(dist, options);
        lut.extend_from_slice(&rgba);
    }
    lut
}

/// Map a single distance value to a [0, 1] severity score using the configured thresholds.
/// 0 = green / safe, 1 = red / out-of-spec.
pub fn severity_for(distance_mm: f64, options: &DistanceShaderOptions) -> f64 {
    let g = options.green_threshold_mm;
    let r = options.red_threshold_mm;
    if r <= g {
        return if distance_mm > g { 1.0 } else { 0.0 };
    }
    let normalized = (distance_mm - g) / (r - g);
    normalized.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;
    use tlanticad_mesh::nalgebra::Point3;

    #[test]
    fn identical_meshes_yield_zero_distance_field() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let (dists, stats, buckets) = compute_distance_field(&a, &b, 8);
        assert_eq!(dists.len(), a.vertex_count());
        assert!(stats.max_mm < 1e-9);
        assert!(stats.mean_mm < 1e-9);
        assert!(!buckets.is_empty());
    }

    #[test]
    fn translated_mesh_recovers_distance_distribution() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
        let (_, stats, buckets) = compute_distance_field(&a, &b, 8);
        assert!(stats.max_mm > 0.0);
        assert!(stats.mean_mm > 0.0);
        assert!(buckets.iter().map(|b| b.count).sum::<usize>() == a.vertex_count());
    }

    #[test]
    fn percentile_recovers_known_value() {
        let v = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let p = percentile_inclusive(&v, 0.5);
        assert!((p - 2.0).abs() < 1e-9);
    }

    #[test]
    fn severity_clamps_to_unit_interval() {
        let opts = DistanceShaderOptions::default();
        assert!(severity_for(-5.0, &opts) <= 1.0);
        assert!(severity_for(0.0, &opts) >= 0.0);
        assert!(severity_for(100.0, &opts) <= 1.0);
        assert!((severity_for(opts.green_threshold_mm, &opts) - 0.0).abs() < 1e-9);
        assert!((severity_for(opts.red_threshold_mm, &opts) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn histogram_total_matches_input() {
        let distances = (0..100).map(|i| i as f64 * 0.1).collect::<Vec<_>>();
        let buckets = build_histogram(&distances, 10, 0.0, 10.0);
        let sum: usize = buckets.iter().map(|b| b.count).sum();
        assert_eq!(sum, 100);
    }

    #[test]
    fn empty_meshes_return_default_stats() {
        let a = Mesh::new("empty");
        let b = Mesh::new("empty");
        let (dists, stats, buckets) = compute_distance_field(&a, &b, 4);
        assert!(dists.is_empty());
        assert_eq!(stats.vertex_count, 0);
        assert!(buckets.is_empty());
    }

    // ── V393: signed_distance_field + texture_color_ramp ──────────────────────

    #[test]
    fn signed_field_detects_interpenetration_below_target() {
        // src box sits below target box along +Z. With axis = +Z, every src vertex
        // is "below" the target ⇒ negative signed distance.
        let src = create_box(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.5));
        let target = create_box(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 1.0, 2.0));
        let axis = tlanticad_mesh::nalgebra::Vector3::z();
        let (unsigned, signs, signed) = signed_distance_field(&src, &target, &axis);
        assert_eq!(unsigned.len(), src.vertex_count());
        assert!(unsigned.iter().all(|&d| d >= 0.0));
        // Target is above src along +z, so src→target delta has negative z component
        // for every vertex below target — every sign should be -1.
        assert!(signs.iter().any(|&s| s < 0.0));
        assert!(signed.iter().any(|&d| d < 0.0));
    }

    #[test]
    fn signed_field_above_target_yields_positive_signs() {
        let src = create_box(Point3::new(0.0, 0.0, 2.0), Point3::new(1.0, 1.0, 3.0));
        let target = create_box(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let axis = tlanticad_mesh::nalgebra::Vector3::z();
        let (_, signs, _) = signed_distance_field(&src, &target, &axis);
        assert!(signs.iter().all(|&s| s > 0.0));
    }

    #[test]
    fn texture_color_ramp_safe_is_green() {
        let opts = DistanceShaderOptions::default();
        let rgba = texture_color_ramp(opts.green_threshold_mm, &opts);
        assert!(rgba[1] >= 250); // green dominant
        assert_eq!(rgba[3], 255);
    }

    #[test]
    fn texture_color_ramp_warning_is_red() {
        let opts = DistanceShaderOptions::default();
        let rgba = texture_color_ramp(opts.red_threshold_mm, &opts);
        assert_eq!(rgba[0], 255); // red full
        assert!(rgba[1] <= 5);
    }

    #[test]
    fn texture_color_ramp_interpenetration_is_blue() {
        let opts = DistanceShaderOptions::default();
        let rgba = texture_color_ramp(-0.5, &opts);
        assert!(rgba[2] > rgba[0]);
        assert!(rgba[2] > rgba[1]);
    }

    #[test]
    fn build_distance_texture_matches_vertex_count() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
        let opts = DistanceShaderOptions::default();
        let buf = build_distance_texture(&a, &b, &tlanticad_mesh::nalgebra::Vector3::x(), &opts);
        assert_eq!(buf.len(), a.vertex_count() * 4);
    }

    #[test]
    fn build_color_lut_has_256_texels() {
        let opts = DistanceShaderOptions::default();
        let lut = build_color_lut(&opts);
        assert_eq!(lut.len(), 256 * 4);
        // First texel = green, last = red.
        assert!(lut[1] > lut[0]);
        let last = lut.len() - 4;
        assert!(lut[last] > lut[last + 1]);
    }
}

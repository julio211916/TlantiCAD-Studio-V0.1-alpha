//! AI Tooth Segmentation
//!
//! Automatic segmentation of dental scans into individual teeth,
//! gingiva, and anatomical landmarks using geometric heuristics
//! (production would use ML models via ONNX runtime).

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Segment label for each mesh region
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SegmentLabel {
    Tooth(u8),          // FDI tooth number
    Gingiva,
    PreparedTooth(u8),
    Implant(u8),
    SoftTissue,
    Artifact,
    Unknown,
}

impl SegmentLabel {
    pub fn is_tooth(&self) -> bool {
        matches!(self, Self::Tooth(_) | Self::PreparedTooth(_))
    }

    pub fn is_clinical(&self) -> bool {
        !matches!(self, Self::Artifact | Self::Unknown)
    }
}

/// Segmented region of a mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedRegion {
    pub label: SegmentLabel,
    pub vertex_indices: Vec<usize>,
    pub confidence: f64,
    pub centroid: [f64; 3],
    pub bounding_box_min: [f64; 3],
    pub bounding_box_max: [f64; 3],
}

impl SegmentedRegion {
    pub fn vertex_count(&self) -> usize {
        self.vertex_indices.len()
    }

    pub fn bounding_box_volume(&self) -> f64 {
        let dx = self.bounding_box_max[0] - self.bounding_box_min[0];
        let dy = self.bounding_box_max[1] - self.bounding_box_min[1];
        let dz = self.bounding_box_max[2] - self.bounding_box_min[2];
        dx * dy * dz
    }
}

/// Full segmentation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationResult {
    pub regions: Vec<SegmentedRegion>,
    pub total_vertices: usize,
    pub processing_method: String,
    pub overall_confidence: f64,
}

impl SegmentationResult {
    pub fn teeth(&self) -> Vec<&SegmentedRegion> {
        self.regions.iter().filter(|r| r.label.is_tooth()).collect()
    }

    pub fn gingiva(&self) -> Option<&SegmentedRegion> {
        self.regions.iter().find(|r| r.label == SegmentLabel::Gingiva)
    }

    pub fn coverage_pct(&self) -> f64 {
        let segmented: usize = self.regions.iter().map(|r| r.vertex_count()).sum();
        if self.total_vertices == 0 { return 0.0; }
        segmented as f64 / self.total_vertices as f64 * 100.0
    }

    pub fn find_tooth(&self, number: u8) -> Option<&SegmentedRegion> {
        self.regions.iter().find(|r| matches!(r.label, SegmentLabel::Tooth(n) | SegmentLabel::PreparedTooth(n) if n == number))
    }
}

/// Segment a full-arch scan using height-based clustering
/// (heuristic approach; production would use ML model)
pub fn segment_arch(mesh: &Mesh) -> SegmentationResult {
    if mesh.vertices.is_empty() {
        return SegmentationResult {
            regions: Vec::new(),
            total_vertices: 0,
            processing_method: "height-clustering".into(),
            overall_confidence: 0.0,
        };
    }

    let (min, max) = mesh.calculate_bounds();
    let height_range = max.z - min.z;

    // Separate gingiva (lower portion) from teeth (upper portion)
    let gingiva_threshold = min.z + height_range * 0.35;

    let mut gingiva_indices = Vec::new();
    let mut tooth_indices = Vec::new();

    for (i, v) in mesh.vertices.iter().enumerate() {
        if v.z < gingiva_threshold {
            gingiva_indices.push(i);
        } else {
            tooth_indices.push(i);
        }
    }

    let mut regions = Vec::new();

    // Gingiva region
    if !gingiva_indices.is_empty() {
        let centroid = compute_centroid(mesh, &gingiva_indices);
        let (bmin, bmax) = compute_bbox(mesh, &gingiva_indices);
        regions.push(SegmentedRegion {
            label: SegmentLabel::Gingiva,
            vertex_indices: gingiva_indices,
            confidence: 0.7,
            centroid,
            bounding_box_min: bmin,
            bounding_box_max: bmax,
        });
    }

    // Cluster tooth vertices by X position (simplified arch segmentation)
    // For a full arch, teeth are distributed along the arch curve
    if !tooth_indices.is_empty() {
        let tooth_regions = cluster_by_position(mesh, &tooth_indices, 14); // ~14 teeth per arch
        for (_idx, cluster) in tooth_regions.into_iter().enumerate() {
            let tooth_num = estimate_tooth_number(mesh, &cluster, min.x, max.x);
            let centroid = compute_centroid(mesh, &cluster);
            let (bmin, bmax) = compute_bbox(mesh, &cluster);
            regions.push(SegmentedRegion {
                label: SegmentLabel::Tooth(tooth_num),
                vertex_indices: cluster,
                confidence: 0.55,
                centroid,
                bounding_box_min: bmin,
                bounding_box_max: bmax,
            });
        }
    }

    let total = mesh.vertex_count();
    let avg_conf = if regions.is_empty() {
        0.0
    } else {
        regions.iter().map(|r| r.confidence).sum::<f64>() / regions.len() as f64
    };

    SegmentationResult {
        regions,
        total_vertices: total,
        processing_method: "height-clustering".into(),
        overall_confidence: avg_conf,
    }
}

/// Simple clustering by X position along the arch
fn cluster_by_position(mesh: &Mesh, indices: &[usize], target_clusters: usize) -> Vec<Vec<usize>> {
    if indices.is_empty() || target_clusters == 0 {
        return Vec::new();
    }

    // Sort by x-coordinate
    let mut sorted: Vec<(usize, f64)> = indices.iter()
        .map(|&i| (i, mesh.vertices[i].x))
        .collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let chunk_size = (sorted.len() / target_clusters).max(1);
    sorted.chunks(chunk_size)
        .map(|chunk| chunk.iter().map(|(i, _)| *i).collect())
        .collect()
}

fn estimate_tooth_number(_mesh: &Mesh, cluster: &[usize], _min_x: f64, _max_x: f64) -> u8 {
    // Simplified: assign sequential FDI numbers
    // Production would use ML-predicted landmarks
    let idx = cluster.first().copied().unwrap_or(0);
    let hash = (idx % 28) as u8;
    let quadrant = if hash < 7 { 10 } else if hash < 14 { 20 } else if hash < 21 { 30 } else { 40 };
    let position = (hash % 7) as u8 + 1;
    quadrant + position
}

fn compute_centroid(mesh: &Mesh, indices: &[usize]) -> [f64; 3] {
    if indices.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let sum: Vector3<f64> = indices.iter().map(|&i| mesh.vertices[i].coords).sum();
    let n = indices.len() as f64;
    [sum.x / n, sum.y / n, sum.z / n]
}

fn compute_bbox(mesh: &Mesh, indices: &[usize]) -> ([f64; 3], [f64; 3]) {
    let mut bmin = [f64::MAX; 3];
    let mut bmax = [f64::MIN; 3];
    for &i in indices {
        let v = mesh.vertices[i];
        bmin[0] = bmin[0].min(v.x);
        bmin[1] = bmin[1].min(v.y);
        bmin[2] = bmin[2].min(v.z);
        bmax[0] = bmax[0].max(v.x);
        bmax[1] = bmax[1].max(v.y);
        bmax[2] = bmax[2].max(v.z);
    }
    (bmin, bmax)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch_mesh() -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        // Create a simple arch shape: teeth on top, gingiva below
        // Gingiva plane (z=0..1)
        for i in 0..10 {
            for j in 0..10 {
                vertices.push(Point3::new(i as f64 * 2.0, j as f64 * 2.0, 0.5));
            }
        }
        // Tooth bumps (z=3..5) distributed along x
        for t in 0..7 {
            let cx = t as f64 * 3.0 + 1.0;
            for dy in 0..3 {
                for dz in 0..3 {
                    vertices.push(Point3::new(cx, dy as f64, 3.0 + dz as f64));
                }
            }
        }
        // Minimal triangulation
        for i in 0..9 {
            for j in 0..9 {
                let idx = i * 10 + j;
                indices.push([idx as u32, (idx + 1) as u32, (idx + 10) as u32]);
            }
        }
        let mut mesh = Mesh::new("arch");
        mesh.vertices = vertices;
        mesh.normals = Vec::new();
        mesh.indices = indices;
        mesh
    }

    #[test]
    fn test_segment_empty_mesh() {
        let mesh = Mesh::new("empty");
        let result = segment_arch(&mesh);
        assert_eq!(result.regions.len(), 0);
        assert_eq!(result.total_vertices, 0);
    }

    #[test]
    fn test_segment_arch_has_regions() {
        let mesh = make_arch_mesh();
        let result = segment_arch(&mesh);
        assert!(!result.regions.is_empty());
        assert!(result.total_vertices > 0);
        assert!(result.overall_confidence > 0.0);
    }

    #[test]
    fn test_segment_coverage() {
        let mesh = make_arch_mesh();
        let result = segment_arch(&mesh);
        assert!(result.coverage_pct() > 0.0);
        assert!(result.coverage_pct() <= 100.0);
    }

    #[test]
    fn test_segment_has_gingiva() {
        let mesh = make_arch_mesh();
        let result = segment_arch(&mesh);
        assert!(result.gingiva().is_some());
    }

    #[test]
    fn test_segment_has_teeth() {
        let mesh = make_arch_mesh();
        let result = segment_arch(&mesh);
        assert!(!result.teeth().is_empty());
    }

    #[test]
    fn test_segment_label_is_tooth() {
        assert!(SegmentLabel::Tooth(11).is_tooth());
        assert!(SegmentLabel::PreparedTooth(14).is_tooth());
        assert!(!SegmentLabel::Gingiva.is_tooth());
    }

    #[test]
    fn test_segment_label_is_clinical() {
        assert!(SegmentLabel::Tooth(11).is_clinical());
        assert!(!SegmentLabel::Artifact.is_clinical());
    }

    #[test]
    fn test_region_bounding_box_volume() {
        let region = SegmentedRegion {
            label: SegmentLabel::Tooth(11),
            vertex_indices: vec![0, 1, 2],
            confidence: 0.8,
            centroid: [5.0, 5.0, 5.0],
            bounding_box_min: [0.0, 0.0, 0.0],
            bounding_box_max: [10.0, 10.0, 10.0],
        };
        assert!((region.bounding_box_volume() - 1000.0).abs() < 1e-10);
    }
}

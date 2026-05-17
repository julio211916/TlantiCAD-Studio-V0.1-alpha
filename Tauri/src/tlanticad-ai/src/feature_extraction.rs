//! AI Feature Extraction from 3D Dental Meshes
//!
//! Extract geometric features (curvature, normals, thickness, landmarks)
//! that serve as input to ML models or heuristic analysis pipelines.

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Per-vertex feature vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexFeatures {
    pub vertex_index: usize,
    pub position: [f64; 3],
    pub normal: [f64; 3],
    pub mean_curvature: f64,
    pub gaussian_curvature: f64,
    pub local_thickness: f64,
    pub height_normalized: f64,
    pub distance_to_centroid: f64,
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub compute_curvature: bool,
    pub compute_thickness: bool,
    pub neighborhood_radius: f64,
    pub normalize: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            compute_curvature: true,
            compute_thickness: true,
            neighborhood_radius: 2.0,
            normalize: true,
        }
    }
}

/// Global mesh features (summary statistics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshFeatureSummary {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub surface_area: f64,
    pub bounding_box_volume: f64,
    pub aspect_ratio: f64,
    pub center_of_mass: [f64; 3],
    pub max_dimension: f64,
    pub compactness: f64,      // how sphere-like the shape is
    pub mean_edge_length: f64,
}

/// Extract global feature summary for a mesh
pub fn extract_summary(mesh: &Mesh) -> MeshFeatureSummary {
    let (min, max) = mesh.calculate_bounds();
    let dims = max - min;
    let max_dim = dims.x.max(dims.y).max(dims.z);
    let min_dim = dims.x.min(dims.y).min(dims.z).max(1e-10);
    let bb_vol = dims.x * dims.y * dims.z;
    let area = tlanticad_mesh::surface_area(mesh);

    // Center of mass (approximate via vertex average)
    let center = if mesh.vertices.is_empty() {
        [0.0; 3]
    } else {
        let sum: Vector3<f64> = mesh.vertices.iter().map(|v| v.coords).sum();
        let n = mesh.vertices.len() as f64;
        [sum.x / n, sum.y / n, sum.z / n]
    };

    // Mean edge length
    let mut total_edge = 0.0;
    let mut edge_count = 0usize;
    for tri in &mesh.indices {
        for pair in [[0, 1], [1, 2], [2, 0]] {
            let v0 = mesh.vertices[tri[pair[0]] as usize];
            let v1 = mesh.vertices[tri[pair[1]] as usize];
            total_edge += (v1 - v0).norm();
            edge_count += 1;
        }
    }
    let mean_edge = if edge_count > 0 { total_edge / edge_count as f64 } else { 0.0 };

    // Compactness: 36*pi*V^2 / A^3 (1.0 for sphere)
    let compactness = if area > 0.0 {
        let volume_approx = bb_vol * 0.5; // rough estimate
        (36.0 * std::f64::consts::PI * volume_approx.powi(2) / area.powi(3)).min(1.0)
    } else {
        0.0
    };

    MeshFeatureSummary {
        vertex_count: mesh.vertex_count(),
        triangle_count: mesh.triangle_count(),
        surface_area: area,
        bounding_box_volume: bb_vol,
        aspect_ratio: max_dim / min_dim,
        center_of_mass: center,
        max_dimension: max_dim,
        compactness,
        mean_edge_length: mean_edge,
    }
}

/// Extract per-vertex features for ML input
pub fn extract_vertex_features(mesh: &Mesh, config: &FeatureConfig) -> Vec<VertexFeatures> {
    if mesh.vertices.is_empty() {
        return Vec::new();
    }

    let (min, max) = mesh.calculate_bounds();
    let height_range = (max.z - min.z).max(1e-10);

    let center: Vector3<f64> = mesh.vertices.iter().map(|v| v.coords).sum::<Vector3<f64>>()
        / mesh.vertices.len() as f64;

    mesh.vertices.iter().enumerate().map(|(i, v)| {
        let normal = if i < mesh.normals.len() {
            [mesh.normals[i].x, mesh.normals[i].y, mesh.normals[i].z]
        } else {
            [0.0, 0.0, 1.0]
        };

        let height_normalized = (v.z - min.z) / height_range;
        let dist_to_center = (v.coords - center).norm();

        // Curvature estimation (simplified: based on normal variation in neighborhood)
        let (mean_curv, gauss_curv) = if config.compute_curvature {
            estimate_curvature(mesh, i, config.neighborhood_radius)
        } else {
            (0.0, 0.0)
        };

        VertexFeatures {
            vertex_index: i,
            position: [v.x, v.y, v.z],
            normal,
            mean_curvature: mean_curv,
            gaussian_curvature: gauss_curv,
            local_thickness: 0.0, // requires ray-casting
            height_normalized,
            distance_to_centroid: dist_to_center,
        }
    }).collect()
}

/// Simplified curvature estimation using normal variation
fn estimate_curvature(mesh: &Mesh, vertex_idx: usize, radius: f64) -> (f64, f64) {
    let v = mesh.vertices[vertex_idx];
    let r2 = radius * radius;

    let mut normal_var = 0.0;
    let mut count = 0usize;
    let base_normal = if vertex_idx < mesh.normals.len() {
        mesh.normals[vertex_idx]
    } else {
        return (0.0, 0.0);
    };

    for (j, other) in mesh.vertices.iter().enumerate() {
        if j == vertex_idx { continue; }
        let d2 = (other - v).norm_squared();
        if d2 < r2 && j < mesh.normals.len() {
            let dot = base_normal.dot(&mesh.normals[j]).clamp(-1.0, 1.0);
            normal_var += (1.0 - dot).abs();
            count += 1;
        }
    }

    if count == 0 {
        return (0.0, 0.0);
    }

    let mean_curvature = normal_var / count as f64;
    // Gaussian curvature approximated as (mean_curvature)^2 * sign
    let gaussian = mean_curvature * mean_curvature;
    (mean_curvature, gaussian)
}

/// Convert features to flat f32 array for model input
pub fn features_to_tensor(features: &[VertexFeatures]) -> Vec<f32> {
    let mut data = Vec::with_capacity(features.len() * 8);
    for f in features {
        data.push(f.position[0] as f32);
        data.push(f.position[1] as f32);
        data.push(f.position[2] as f32);
        data.push(f.normal[0] as f32);
        data.push(f.normal[1] as f32);
        data.push(f.normal[2] as f32);
        data.push(f.mean_curvature as f32);
        data.push(f.height_normalized as f32);
    }
    data
}

/// Compute curvature histogram for mesh characterization
pub fn curvature_histogram(features: &[VertexFeatures], bins: usize) -> Vec<usize> {
    if features.is_empty() || bins == 0 {
        return vec![0; bins];
    }
    let max_curv = features.iter()
        .map(|f| f.mean_curvature)
        .fold(0.0f64, |a, b| a.max(b))
        .max(1e-10);

    let mut histogram = vec![0usize; bins];
    for f in features {
        let bin = ((f.mean_curvature / max_curv) * (bins - 1) as f64).floor() as usize;
        let bin = bin.min(bins - 1);
        histogram[bin] += 1;
    }
    histogram
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mesh() -> Mesh {
        // Simple box-like mesh
        let mut mesh = Mesh::new("test");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 0.0),
            Point3::new(0.0, 10.0, 0.0),
            Point3::new(5.0, 5.0, 8.0),
        ];
        mesh.normals = vec![
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        mesh.indices = vec![
            [0, 1, 4],
            [1, 2, 4],
            [2, 3, 4],
            [3, 0, 4],
            [0, 1, 2],
        ];
        mesh
    }

    #[test]
    fn test_extract_summary() {
        let mesh = make_test_mesh();
        let summary = extract_summary(&mesh);
        assert_eq!(summary.vertex_count, 5);
        assert_eq!(summary.triangle_count, 5);
        assert!(summary.surface_area > 0.0);
        assert!(summary.bounding_box_volume > 0.0);
        assert!(summary.max_dimension > 0.0);
    }

    #[test]
    fn test_extract_summary_empty() {
        let mesh = Mesh::new("empty");
        let summary = extract_summary(&mesh);
        assert_eq!(summary.vertex_count, 0);
    }

    #[test]
    fn test_vertex_features() {
        let mesh = make_test_mesh();
        let features = extract_vertex_features(&mesh, &FeatureConfig::default());
        assert_eq!(features.len(), 5);
        // Height normalized should be between 0 and 1
        for f in &features {
            assert!(f.height_normalized >= 0.0 && f.height_normalized <= 1.0);
            assert!(f.distance_to_centroid >= 0.0);
        }
    }

    #[test]
    fn test_features_to_tensor() {
        let mesh = make_test_mesh();
        let features = extract_vertex_features(&mesh, &FeatureConfig { compute_curvature: false, ..Default::default() });
        let tensor = features_to_tensor(&features);
        assert_eq!(tensor.len(), features.len() * 8);
    }

    #[test]
    fn test_curvature_histogram() {
        let mesh = make_test_mesh();
        let features = extract_vertex_features(&mesh, &FeatureConfig::default());
        let hist = curvature_histogram(&features, 10);
        assert_eq!(hist.len(), 10);
        let total: usize = hist.iter().sum();
        assert_eq!(total, features.len());
    }

    #[test]
    fn test_feature_config_default() {
        let cfg = FeatureConfig::default();
        assert!(cfg.compute_curvature);
        assert!(cfg.normalize);
        assert!(cfg.neighborhood_radius > 0.0);
    }

    #[test]
    fn test_summary_aspect_ratio() {
        let mesh = make_test_mesh();
        let summary = extract_summary(&mesh);
        assert!(summary.aspect_ratio >= 1.0);
    }

    #[test]
    fn test_summary_mean_edge() {
        let mesh = make_test_mesh();
        let summary = extract_summary(&mesh);
        assert!(summary.mean_edge_length > 0.0);
    }
}

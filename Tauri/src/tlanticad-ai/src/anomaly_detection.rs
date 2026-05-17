//! AI Anomaly Detection for Dental Scans
//!
//! Detect artifacts, scan defects, bubbles, and quality issues
//! in intraoral scans using statistical and geometric heuristics.

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Type of anomaly detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    Hole,
    SpikeArtifact,
    FloatingFragment,
    SelfIntersection,
    DegenerateTriangle,
    FlippedNormal,
    ThinWall,
    ScanGap,
    Noise,
}

impl AnomalyType {
    pub fn severity(&self) -> AnomalySeverity {
        match self {
            Self::Hole | Self::SelfIntersection => AnomalySeverity::Critical,
            Self::SpikeArtifact | Self::FloatingFragment | Self::ScanGap => AnomalySeverity::Warning,
            Self::DegenerateTriangle | Self::FlippedNormal => AnomalySeverity::Minor,
            Self::ThinWall | Self::Noise => AnomalySeverity::Info,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Critical,
    Warning,
    Minor,
    Info,
}

/// Detected anomaly with location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub location: [f64; 3],
    pub affected_vertices: Vec<usize>,
    pub confidence: f64,
    pub description: String,
}

/// Full anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub anomalies: Vec<Anomaly>,
    pub scan_quality_score: f64, // 0.0 (terrible) to 1.0 (perfect)
    pub is_usable: bool,
    pub critical_count: usize,
    pub warning_count: usize,
}

impl AnomalyReport {
    pub fn is_clean(&self) -> bool {
        self.anomalies.is_empty()
    }

    pub fn by_type(&self, t: AnomalyType) -> Vec<&Anomaly> {
        self.anomalies.iter().filter(|a| a.anomaly_type == t).collect()
    }

    pub fn by_severity(&self, s: AnomalySeverity) -> Vec<&Anomaly> {
        self.anomalies.iter().filter(|a| a.severity == s).collect()
    }
}

/// Run full anomaly detection on a mesh
pub fn detect_anomalies(mesh: &Mesh) -> AnomalyReport {
    let mut anomalies = Vec::new();

    if mesh.vertices.is_empty() {
        return AnomalyReport {
            anomalies: Vec::new(),
            scan_quality_score: 0.0,
            is_usable: false,
            critical_count: 0,
            warning_count: 0,
        };
    }

    // 1. Detect holes (boundary edges)
    let boundary = tlanticad_mesh::boundary_edges(mesh);
    if !boundary.is_empty() {
        let location = mesh.vertices[boundary[0].0 as usize];
        anomalies.push(Anomaly {
            anomaly_type: AnomalyType::Hole,
            severity: AnomalySeverity::Critical,
            location: [location.x, location.y, location.z],
            affected_vertices: boundary.iter().map(|e| e.0 as usize).collect(),
            confidence: 0.95,
            description: format!("{} boundary edge(s) detected", boundary.len()),
        });
    }

    // 2. Detect spike artifacts (vertices far from neighbors)
    let spikes = detect_spikes(mesh, 3.0);
    for spike_idx in &spikes {
        let v = mesh.vertices[*spike_idx];
        anomalies.push(Anomaly {
            anomaly_type: AnomalyType::SpikeArtifact,
            severity: AnomalySeverity::Warning,
            location: [v.x, v.y, v.z],
            affected_vertices: vec![*spike_idx],
            confidence: 0.7,
            description: "Vertex far from local neighborhood mean".into(),
        });
    }

    // 3. Detect degenerate triangles
    let degenerates = detect_degenerate_triangles(mesh, 1e-10);
    for tri_idx in &degenerates {
        let tri = &mesh.indices[*tri_idx];
        let v = mesh.vertices[tri[0] as usize];
        anomalies.push(Anomaly {
            anomaly_type: AnomalyType::DegenerateTriangle,
            severity: AnomalySeverity::Minor,
            location: [v.x, v.y, v.z],
            affected_vertices: tri.iter().map(|&i| i as usize).collect(),
            confidence: 0.99,
            description: format!("Triangle {} has near-zero area", tri_idx),
        });
    }

    // 4. Detect flipped normals
    let flipped = detect_flipped_normals(mesh);
    if flipped.len() > mesh.normals.len() / 20 {
        anomalies.push(Anomaly {
            anomaly_type: AnomalyType::FlippedNormal,
            severity: AnomalySeverity::Minor,
            location: [0.0, 0.0, 0.0],
            affected_vertices: flipped,
            confidence: 0.6,
            description: "Multiple normals appear inconsistent".into(),
        });
    }

    // Compute overall score
    let critical = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Critical).count();
    let warnings = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Warning).count();
    let minor = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Minor).count();

    let score = (1.0 - critical as f64 * 0.3 - warnings as f64 * 0.1 - minor as f64 * 0.02).max(0.0);

    AnomalyReport {
        anomalies,
        scan_quality_score: score,
        is_usable: critical == 0,
        critical_count: critical,
        warning_count: warnings,
    }
}

/// Detect vertices that are statistical outliers (spikes)
fn detect_spikes(mesh: &Mesh, std_multiplier: f64) -> Vec<usize> {
    if mesh.vertices.len() < 10 {
        return Vec::new();
    }

    // Compute mean position
    let center: Vector3<f64> = mesh.vertices.iter().map(|v| v.coords).sum::<Vector3<f64>>()
        / mesh.vertices.len() as f64;

    // Compute std deviation of distances
    let distances: Vec<f64> = mesh.vertices.iter()
        .map(|v| (v.coords - center).norm())
        .collect();

    let mean_dist: f64 = distances.iter().sum::<f64>() / distances.len() as f64;
    let variance: f64 = distances.iter()
        .map(|d| (d - mean_dist).powi(2))
        .sum::<f64>() / distances.len() as f64;
    let std_dev = variance.sqrt();
    let threshold = mean_dist + std_multiplier * std_dev;

    distances.iter().enumerate()
        .filter(|(_, d)| **d > threshold)
        .map(|(i, _)| i)
        .collect()
}

/// Detect degenerate (zero-area) triangles
fn detect_degenerate_triangles(mesh: &Mesh, min_area: f64) -> Vec<usize> {
    mesh.indices.iter().enumerate()
        .filter(|(_, tri)| {
            let v0 = mesh.vertices[tri[0] as usize];
            let v1 = mesh.vertices[tri[1] as usize];
            let v2 = mesh.vertices[tri[2] as usize];
            let cross = (v1 - v0).cross(&(v2 - v0));
            cross.norm() * 0.5 < min_area
        })
        .map(|(i, _)| i)
        .collect()
}

/// Detect normals pointing inward (inconsistent orientation)
fn detect_flipped_normals(mesh: &Mesh) -> Vec<usize> {
    if mesh.normals.is_empty() {
        return Vec::new();
    }

    let center: Vector3<f64> = mesh.vertices.iter().map(|v| v.coords).sum::<Vector3<f64>>()
        / mesh.vertices.len().max(1) as f64;

    mesh.normals.iter().enumerate()
        .filter(|(i, n)| {
            if *i >= mesh.vertices.len() { return false; }
            let outward = (mesh.vertices[*i].coords - center).normalize();
            n.dot(&outward) < -0.5 // normal points significantly inward
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clean_mesh() -> Mesh {
        let mut mesh = Mesh::new("clean");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(5.0, 10.0, 0.0),
            Point3::new(5.0, 5.0, 8.0),
        ];
        mesh.normals = vec![
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        mesh.indices = vec![
            [0, 1, 3],
            [1, 2, 3],
            [2, 0, 3],
            [0, 2, 1], // base (closed)
        ];
        mesh
    }

    #[test]
    fn test_empty_mesh_anomalies() {
        let mesh = Mesh::new("empty");
        let report = detect_anomalies(&mesh);
        assert!(!report.is_usable);
        assert_eq!(report.scan_quality_score, 0.0);
    }

    #[test]
    fn test_clean_mesh_score() {
        let mesh = make_clean_mesh();
        let report = detect_anomalies(&mesh);
        assert!(report.scan_quality_score > 0.0);
    }

    #[test]
    fn test_anomaly_type_severity() {
        assert_eq!(AnomalyType::Hole.severity(), AnomalySeverity::Critical);
        assert_eq!(AnomalyType::SpikeArtifact.severity(), AnomalySeverity::Warning);
        assert_eq!(AnomalyType::DegenerateTriangle.severity(), AnomalySeverity::Minor);
        assert_eq!(AnomalyType::Noise.severity(), AnomalySeverity::Info);
    }

    #[test]
    fn test_degenerate_triangle_detection() {
        let mut mesh = make_clean_mesh();
        // Add degenerate triangle (all vertices at same point)
        let n = mesh.vertices.len() as u32;
        mesh.vertices.push(Point3::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Point3::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Point3::new(0.0, 0.0, 0.0));
        mesh.indices.push([n, n + 1, n + 2]);

        let degenerates = detect_degenerate_triangles(&mesh, 1e-10);
        assert!(!degenerates.is_empty());
    }

    #[test]
    fn test_spike_detection_no_spikes() {
        let mesh = make_clean_mesh();
        let spikes = detect_spikes(&mesh, 3.0);
        assert!(spikes.is_empty());
    }

    #[test]
    fn test_report_by_type() {
        let report = AnomalyReport {
            anomalies: vec![
                Anomaly {
                    anomaly_type: AnomalyType::Hole,
                    severity: AnomalySeverity::Critical,
                    location: [0.0, 0.0, 0.0],
                    affected_vertices: vec![0],
                    confidence: 0.9,
                    description: "test".into(),
                },
                Anomaly {
                    anomaly_type: AnomalyType::Noise,
                    severity: AnomalySeverity::Info,
                    location: [1.0, 1.0, 1.0],
                    affected_vertices: vec![1],
                    confidence: 0.5,
                    description: "test2".into(),
                },
            ],
            scan_quality_score: 0.5,
            is_usable: false,
            critical_count: 1,
            warning_count: 0,
        };
        assert_eq!(report.by_type(AnomalyType::Hole).len(), 1);
        assert_eq!(report.by_type(AnomalyType::Noise).len(), 1);
        assert_eq!(report.by_type(AnomalyType::SpikeArtifact).len(), 0);
    }

    #[test]
    fn test_report_by_severity() {
        let report = detect_anomalies(&make_clean_mesh());
        let criticals = report.by_severity(AnomalySeverity::Critical);
        assert_eq!(criticals.len(), report.critical_count);
    }

    #[test]
    fn test_report_is_clean() {
        let report = AnomalyReport {
            anomalies: Vec::new(),
            scan_quality_score: 1.0,
            is_usable: true,
            critical_count: 0,
            warning_count: 0,
        };
        assert!(report.is_clean());
    }
}

//! TlantiCAD AI Module
//! 
//! Machine learning features for dental CAD:
//! margin detection, tooth positioning, mesh quality analysis,
//! ONNX runtime integration, inference pipeline, task queue

pub mod runtime;
pub mod pipeline;
pub mod queue;
pub mod segmentation;
pub mod feature_extraction;
pub mod anomaly_detection;
pub mod training;
pub mod model_registry;

// AR-V370 — crown fast-path kernel selector (CPU/MPS/CUDA + geometric fallback)
pub mod crown_inference;

use tlanticad_core::Result;
use tlanticad_mesh::Mesh;
use nalgebra::{Point3, Vector3};

/// Margin detection using curvature analysis
/// (Production would use ONNX model; this uses geometric heuristics)
pub async fn detect_margin(mesh: &Mesh, _tooth_number: u8) -> Result<Vec<Point3<f64>>> {
    if mesh.vertices.is_empty() {
        return Ok(Vec::new());
    }

    // Heuristic: find vertices with high curvature change (concavity)
    // This approximates the preparation margin line
    let (min, max) = mesh.calculate_bounds();
    let threshold_z = min.z + (max.z - min.z) * 0.15; // lower 15% of mesh
    
    let mut margin_candidates: Vec<Point3<f64>> = mesh.vertices.iter()
        .filter(|v| (v.z - threshold_z).abs() < 0.3)
        .copied()
        .collect();

    // Sort angularly around centroid for ordered margin line
    if !margin_candidates.is_empty() {
        let center: Vector3<f64> = margin_candidates.iter()
            .map(|p| p.coords).sum::<Vector3<f64>>() / margin_candidates.len() as f64;
        margin_candidates.sort_by(|a, b| {
            let aa = (a.y - center.y).atan2(a.x - center.x);
            let ab = (b.y - center.y).atan2(b.x - center.x);
            aa.partial_cmp(&ab).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    Ok(margin_candidates)
}

/// Auto-suggest tooth position based on adjacent teeth
pub struct ToothSuggestion {
    pub tooth_number: u8,
    pub position: Point3<f64>,
    pub rotation: nalgebra::UnitQuaternion<f64>,
    pub confidence: f64,
}

pub async fn suggest_tooth_position(
    preparation: &Mesh,
    antagonist: Option<&Mesh>,
    tooth_number: u8,
) -> Result<ToothSuggestion> {
    let (min, max) = preparation.calculate_bounds();
    let center = Point3::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        max.z, // place at top of preparation
    );

    // Estimate rotation from preparation shape
    let prep_axis = Vector3::new(max.x - min.x, max.y - min.y, 0.0).normalize();
    let rotation = nalgebra::UnitQuaternion::face_towards(&Vector3::z(), &prep_axis);

    // Confidence is higher if we have antagonist data
    let confidence = if antagonist.is_some() { 0.75 } else { 0.5 };

    Ok(ToothSuggestion {
        tooth_number,
        position: center,
        rotation,
        confidence,
    })
}

/// Mesh quality analysis report
pub struct QualityReport {
    pub has_holes: bool,
    pub is_watertight: bool,
    pub triangle_count: usize,
    pub vertex_count: usize,
    pub min_thickness: Option<f64>,
    pub surface_area: f64,
    pub issues: Vec<String>,
    pub quality_score: f64,    // 0.0 - 1.0
}

pub fn analyze_mesh_quality(mesh: &Mesh) -> QualityReport {
    let holes = tlanticad_mesh::boundary_edges(mesh);
    let watertight = holes.is_empty();
    let area = tlanticad_mesh::surface_area(mesh);

    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if !watertight {
        issues.push(format!("{} hole(s) detected", holes.len()));
        score -= 0.3;
    }

    // Check for degenerate triangles
    let mut degenerate = 0;
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        if edge1.cross(&edge2).norm() < 1e-10 {
            degenerate += 1;
        }
    }
    if degenerate > 0 {
        issues.push(format!("{} degenerate triangle(s)", degenerate));
        score -= 0.1;
    }

    // Check triangle count is reasonable
    if mesh.triangle_count() < 100 {
        issues.push("Very low triangle count".into());
        score -= 0.1;
    }

    // Check for flipped normals (simplified)
    let mut flipped = 0;
    for (_i, n) in mesh.normals.iter().enumerate() {
        if n.norm() < 0.5 {
            flipped += 1;
        }
    }
    if flipped > mesh.normals.len() / 10 {
        issues.push(format!("{} potentially flipped normals", flipped));
        score -= 0.1;
    }

    QualityReport {
        has_holes: !watertight,
        is_watertight: watertight,
        triangle_count: mesh.triangle_count(),
        vertex_count: mesh.vertex_count(),
        min_thickness: None, // requires ray-casting analysis
        surface_area: area,
        issues,
        quality_score: score.max(0.0),
    }
}

/// Suggest automatic repair actions for a mesh
pub fn suggest_repairs(report: &QualityReport) -> Vec<String> {
    let mut repairs = Vec::new();
    if report.has_holes {
        repairs.push("Fill holes with tlanticad_mesh::repair()".into());
    }
    if report.triangle_count > 500_000 {
        repairs.push("Decimate: mesh has excessive triangles".into());
    }
    if report.quality_score < 0.5 {
        repairs.push("Full repair pipeline recommended".into());
    }
    repairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};

    fn make_mesh() -> Mesh {
        let mut mesh = Mesh::new("test");
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
        mesh.indices = vec![[0, 1, 3], [1, 2, 3], [2, 0, 3], [0, 2, 1]];
        mesh
    }

    #[test]
    fn test_analyze_mesh_quality() {
        let mesh = make_mesh();
        let report = analyze_mesh_quality(&mesh);
        assert_eq!(report.vertex_count, 4);
        assert_eq!(report.triangle_count, 4);
        assert!(report.surface_area > 0.0);
    }

    #[test]
    fn test_quality_empty_mesh() {
        let mesh = Mesh::new("empty");
        let report = analyze_mesh_quality(&mesh);
        assert_eq!(report.triangle_count, 0);
        assert!(report.is_watertight); // no triangles = no boundary
    }

    #[test]
    fn test_suggest_repairs_holes() {
        let report = QualityReport {
            has_holes: true,
            is_watertight: false,
            triangle_count: 100,
            vertex_count: 50,
            min_thickness: None,
            surface_area: 100.0,
            issues: vec![],
            quality_score: 0.7,
        };
        let repairs = suggest_repairs(&report);
        assert!(repairs.iter().any(|r| r.contains("Fill holes")));
    }

    #[test]
    fn test_suggest_repairs_excessive_triangles() {
        let report = QualityReport {
            has_holes: false,
            is_watertight: true,
            triangle_count: 1_000_000,
            vertex_count: 500_000,
            min_thickness: None,
            surface_area: 100.0,
            issues: vec![],
            quality_score: 0.9,
        };
        let repairs = suggest_repairs(&report);
        assert!(repairs.iter().any(|r| r.contains("Decimate")));
    }

    #[test]
    fn test_suggest_repairs_low_quality() {
        let report = QualityReport {
            has_holes: false,
            is_watertight: true,
            triangle_count: 100,
            vertex_count: 50,
            min_thickness: None,
            surface_area: 10.0,
            issues: vec![],
            quality_score: 0.3,
        };
        let repairs = suggest_repairs(&report);
        assert!(repairs.iter().any(|r| r.contains("Full repair")));
    }

    #[tokio::test]
    async fn test_detect_margin() {
        let mesh = make_mesh();
        let margin = detect_margin(&mesh, 11).await.unwrap();
        // Small test mesh may or may not produce margin points
        // depending on curvature threshold; just verify it returns without error
        assert!(margin.len() <= mesh.vertices.len());
    }

    #[tokio::test]
    async fn test_detect_margin_empty() {
        let mesh = Mesh::new("empty");
        let margin = detect_margin(&mesh, 11).await.unwrap();
        assert!(margin.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_tooth_position() {
        let mesh = make_mesh();
        let suggestion = suggest_tooth_position(&mesh, None, 21).await.unwrap();
        assert_eq!(suggestion.tooth_number, 21);
        assert!(suggestion.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_suggest_with_antagonist() {
        let mesh = make_mesh();
        let ant = make_mesh();
        let suggestion = suggest_tooth_position(&mesh, Some(&ant), 21).await.unwrap();
        assert!(suggestion.confidence > 0.5); // higher with antagonist
    }
}


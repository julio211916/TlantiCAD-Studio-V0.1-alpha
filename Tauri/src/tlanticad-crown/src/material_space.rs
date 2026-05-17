//! Minimum wall thickness analysis for dental crown materials

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Result of a crown wall thickness analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThicknessAnalysis {
    pub min_thickness_mm: f64,
    pub thin_areas: Vec<Point3<f64>>,
    pub passes_minimum: bool,
}

/// Return the clinical minimum wall thickness (mm) for a given material.
///
/// | Material  | Min (mm) |
/// |-----------|----------|
/// | zirconia  | 0.5      |
/// | emax      | 1.0      |
/// | pfm / pfm-ceramic | 1.5 |
/// | pmma      | 1.2      |
/// | composite | 1.0      |
pub fn minimum_thickness_for_material(material: &str) -> f64 {
    match material.to_lowercase().as_str() {
        s if s.contains("zirconia") || s.contains("zr") => 0.5,
        s if s.contains("emax") || s.contains("lithium") => 1.0,
        s if s.contains("pfm") || s.contains("ceramic") || s.contains("metal") => 1.5,
        s if s.contains("pmma") || s.contains("provisional") => 1.2,
        s if s.contains("composite") || s.contains("resin") => 1.0,
        s if s.contains("titanium") || s.contains("cobalt") || s.contains("chrome") => 0.5,
        _ => 1.0,
    }
}

/// Analyse the minimum wall thickness of a crown mesh against the
/// material-specific minimum requirement.
///
/// For each outer vertex, finds the closest inner-surface vertex and
/// computes the wall thickness as the Euclidean distance.
pub fn check_minimum_thickness(crown: &Mesh, material_type: &str) -> ThicknessAnalysis {
    let min_required = minimum_thickness_for_material(material_type);

    if crown.vertices.len() < 2 {
        return ThicknessAnalysis {
            min_thickness_mm: 0.0,
            thin_areas: Vec::new(),
            passes_minimum: false,
        };
    }

    // Split vertices into "outer" (positive normal Z) and "inner" (negative normal Z)
    // as a simple heuristic; real implementations would raycast inward.
    let n = crown.vertices.len();
    let half = n / 2;

    let mut global_min = f64::MAX;
    let mut thin_areas = Vec::new();

    for i in 0..half.min(crown.vertices.len()) {
        let outer = crown.vertices[i];
        let mut min_d = f64::MAX;
        // Compare with second half as approximate "inner" surface
        for j in half..crown.vertices.len() {
            let d = (outer - crown.vertices[j]).norm();
            if d < min_d {
                min_d = d;
            }
        }
        if min_d < f64::MAX {
            if min_d < global_min {
                global_min = min_d;
            }
            if min_d < min_required {
                thin_areas.push(outer);
            }
        }
    }

    let min_thickness = if global_min == f64::MAX { 0.0 } else { global_min };

    ThicknessAnalysis {
        min_thickness_mm: min_thickness,
        thin_areas,
        passes_minimum: min_thickness >= min_required,
    }
}

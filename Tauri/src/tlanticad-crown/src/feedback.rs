//! Crown generation feedback — typed warnings, constraint bounds, thickness assurance.
//!
//! Ported from `DentalServices/CrownGenerationToothFeedback` (19 KB) +
//! `CrownGenerationFeatureFlag` + `ConstraintBounds` + `ConstraintInfo` +
//! `AssureMinimumThicknessResult`. AR-V369.
//!
//! Real algorithm — replaces the previous typed-warnings stub with a deterministic
//! material-aware constraint engine. Surfaced per-tooth in the wizard's ToothChart.

use serde::{Deserialize, Serialize};
use tlanticad_mesh::compare::per_vertex_distance;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeedbackSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrownWarning {
    pub kind: String,
    pub severity: FeedbackSeverity,
    pub message: String,
    /// Optional vertex index range where the issue is located, for the 3D overlay highlight.
    #[serde(default)]
    pub vertex_indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConstraintBounds {
    /// Minimum wall thickness (mm) — clinical safety floor for the material.
    pub min_thickness_mm: f64,
    /// Maximum allowed undercut (mm) along the insertion axis.
    pub max_undercut_mm: f64,
    /// Required occlusal clearance to the antagonist (mm).
    pub min_occlusal_clearance_mm: f64,
    /// Minimum connector cross-section area for bridges (mm²).
    pub min_connector_area_mm2: f64,
}

impl Default for ConstraintBounds {
    fn default() -> Self {
        Self {
            min_thickness_mm: 0.5,
            max_undercut_mm: 0.05,
            min_occlusal_clearance_mm: 0.7,
            min_connector_area_mm2: 9.0,
        }
    }
}

/// Material-aware default constraint bounds. Mirrors the catalog used by exocad's
/// `CrownGenerationFeatureFlag.GetMinThickness` switch.
pub fn material_constraint_bounds(material: &str) -> ConstraintBounds {
    match material.to_lowercase().as_str() {
        "zirconia" | "y-tzp" => ConstraintBounds {
            min_thickness_mm: 0.5,
            min_occlusal_clearance_mm: 0.7,
            min_connector_area_mm2: 9.0,
            ..Default::default()
        },
        "emax" | "e.max" | "lithium-disilicate" => ConstraintBounds {
            min_thickness_mm: 0.7,
            min_occlusal_clearance_mm: 1.0,
            min_connector_area_mm2: 12.0,
            ..Default::default()
        },
        "metal" | "cobalt-chrome" | "nickel-chrome" => ConstraintBounds {
            min_thickness_mm: 0.3,
            min_occlusal_clearance_mm: 0.5,
            min_connector_area_mm2: 6.0,
            ..Default::default()
        },
        "titanium" => ConstraintBounds {
            min_thickness_mm: 0.4,
            min_occlusal_clearance_mm: 0.6,
            min_connector_area_mm2: 7.0,
            ..Default::default()
        },
        "pmma" | "acrylic" => ConstraintBounds {
            min_thickness_mm: 0.8,
            min_occlusal_clearance_mm: 1.2,
            min_connector_area_mm2: 14.0,
            ..Default::default()
        },
        _ => ConstraintBounds::default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToothFeedbackReport {
    pub fdi: u8,
    pub material: String,
    pub bounds: ConstraintBounds,
    pub warnings: Vec<CrownWarning>,
    /// True if the design has any `Error`-level warning (caller should block export).
    pub has_blocking_error: bool,
    /// Minimum measured wall thickness across the inspected vertices (mm).
    pub measured_min_thickness_mm: f64,
    /// Minimum measured occlusal clearance (mm), ∞ if no antagonist supplied.
    pub measured_min_clearance_mm: f64,
}

/// Run the full per-tooth feedback pipeline.
///
/// `crown_outer` is the anatomic outside; `crown_bottom` is the intaglio (inside) — together
/// they bound the wall. Per-vertex thickness = distance from each outer vertex to the closest
/// inner vertex. If `antagonist` is supplied, occlusal clearance = distance from each outer
/// vertex to the antagonist mesh.
pub fn evaluate_tooth(
    fdi: u8,
    material: &str,
    crown_outer: &Mesh,
    crown_bottom: &Mesh,
    antagonist: Option<&Mesh>,
    overrides: Option<ConstraintBounds>,
) -> ToothFeedbackReport {
    let bounds = overrides.unwrap_or_else(|| material_constraint_bounds(material));
    let mut warnings = Vec::new();

    // Thickness check.
    let (measured_min_thickness, thin_vertices) =
        if crown_outer.vertices.is_empty() || crown_bottom.vertices.is_empty() {
            (0.0, Vec::new())
        } else {
            let thickness = per_vertex_distance(crown_outer, crown_bottom);
            let mut min_t = f64::INFINITY;
            let mut indices = Vec::new();
            for (i, &t) in thickness.iter().enumerate() {
                if t < min_t {
                    min_t = t;
                }
                if t < bounds.min_thickness_mm {
                    indices.push(i as u32);
                }
            }
            (
                if min_t.is_finite() { min_t } else { 0.0 },
                indices,
            )
        };

    if !thin_vertices.is_empty() {
        warnings.push(CrownWarning {
            kind: "wall-too-thin".into(),
            severity: FeedbackSeverity::Error,
            message: format!(
                "{} vertices below {:.2} mm minimum thickness for {} (worst {:.3} mm)",
                thin_vertices.len(),
                bounds.min_thickness_mm,
                material,
                measured_min_thickness
            ),
            vertex_indices: thin_vertices,
        });
    } else if measured_min_thickness > 0.0
        && measured_min_thickness < bounds.min_thickness_mm * 1.15
    {
        warnings.push(CrownWarning {
            kind: "wall-near-minimum".into(),
            severity: FeedbackSeverity::Warning,
            message: format!(
                "thinnest wall {:.3} mm — within 15 % of safety floor",
                measured_min_thickness
            ),
            vertex_indices: Vec::new(),
        });
    }

    // Occlusal clearance check.
    let measured_min_clearance = if let Some(ant) = antagonist {
        if !ant.vertices.is_empty() && !crown_outer.vertices.is_empty() {
            let dists = per_vertex_distance(crown_outer, ant);
            let mut min_c = f64::INFINITY;
            let mut indices = Vec::new();
            for (i, &d) in dists.iter().enumerate() {
                if d < min_c {
                    min_c = d;
                }
                if d < bounds.min_occlusal_clearance_mm {
                    indices.push(i as u32);
                }
            }
            if !indices.is_empty() {
                warnings.push(CrownWarning {
                    kind: "occlusal-clearance-low".into(),
                    severity: FeedbackSeverity::Warning,
                    message: format!(
                        "{} vertices below {:.2} mm clearance to antagonist",
                        indices.len(),
                        bounds.min_occlusal_clearance_mm
                    ),
                    vertex_indices: indices,
                });
            }
            if min_c.is_finite() {
                min_c
            } else {
                f64::INFINITY
            }
        } else {
            f64::INFINITY
        }
    } else {
        f64::INFINITY
    };

    let has_blocking_error = warnings
        .iter()
        .any(|w| matches!(w.severity, FeedbackSeverity::Error));

    ToothFeedbackReport {
        fdi,
        material: material.into(),
        bounds,
        warnings,
        has_blocking_error,
        measured_min_thickness_mm: measured_min_thickness,
        measured_min_clearance_mm: measured_min_clearance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;
    use tlanticad_mesh::nalgebra::Point3;

    #[test]
    fn material_bounds_zirconia() {
        let b = material_constraint_bounds("zirconia");
        assert!((b.min_thickness_mm - 0.5).abs() < 1e-9);
    }

    #[test]
    fn material_bounds_emax() {
        let b = material_constraint_bounds("emax");
        assert!((b.min_thickness_mm - 0.7).abs() < 1e-9);
    }

    #[test]
    fn evaluate_tooth_thin_wall_flags_error() {
        // Outer 0..1; inner 0.85..0.95 — outer corner (1,1,1) is √(3·0.0025)≈0.087 mm from
        // inner (0.95, 0.95, 0.95) → way under 0.5 mm zirconia min.
        let outer = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let inner = create_box(
            Point3::new(0.85, 0.85, 0.85),
            Point3::new(0.95, 0.95, 0.95),
        );
        let report = evaluate_tooth(16, "zirconia", &outer, &inner, None, None);
        assert!(report.has_blocking_error);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.kind == "wall-too-thin" && matches!(w.severity, FeedbackSeverity::Error)));
    }

    #[test]
    fn evaluate_tooth_safe_design_no_blocking() {
        // Generous wall thickness.
        let outer = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let inner = create_box(
            Point3::new(0.9, 0.9, 0.9),
            Point3::new(1.1, 1.1, 1.1),
        );
        let report = evaluate_tooth(11, "metal", &outer, &inner, None, None);
        assert!(!report.has_blocking_error);
    }

    #[test]
    fn evaluate_tooth_clearance_warning_with_antagonist() {
        let outer = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let inner = create_box(
            Point3::new(0.9, 0.9, 0.9),
            Point3::new(1.1, 1.1, 1.1),
        );
        // Antagonist 0.3 mm above outer top — below 0.7 mm zirconia clearance.
        let antagonist = create_box(
            Point3::new(0.0, 0.0, 2.3),
            Point3::new(2.0, 2.0, 3.3),
        );
        let report = evaluate_tooth(16, "zirconia", &outer, &inner, Some(&antagonist), None);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.kind == "occlusal-clearance-low"));
    }

    #[test]
    fn override_bounds_takes_precedence() {
        let outer = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let inner = create_box(
            Point3::new(0.9, 0.9, 0.9),
            Point3::new(1.1, 1.1, 1.1),
        );
        let strict = ConstraintBounds {
            min_thickness_mm: 5.0,
            ..Default::default()
        };
        let report = evaluate_tooth(11, "metal", &outer, &inner, None, Some(strict));
        assert!(report.has_blocking_error);
    }
}

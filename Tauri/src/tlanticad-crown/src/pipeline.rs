//! Crown generation pipeline — 7-step orchestrator. AR-V368.
//!
//! Ported from `DentalServices/CrownGeneration.cs` (645 KB) +
//! `CrownGenerationToothConfig` + `CrownGenerationStep` + `CrownGenerationProgress` +
//! `CrownGenerationResult`.
//!
//! The pipeline composes already-shipped primitives:
//!   1. **MarginDetect**     — `tlanticad_mesh::margin::detect_from_curvature`
//!   2. **InsertionAxis**    — `tlanticad_geometry::insertion::detect_insertion_axis`
//!   3. **BottomGenerate**   — `tlanticad_crown::bottom::generate_bottom_offset`
//!   4. **LibraryFit**       — bbox-scale + axis-align + translate library tooth
//!   5. **OcclusionCheck**   — `tlanticad_crown::feedback::evaluate_tooth`
//!   6. **ApproximalCheck**  — KD-tree distance vs. neighbour mesh (optional)
//!   7. **ConnectorWeld**    — `tlanticad_bridge::connector::generate_connector_mesh` (optional)
//!
//! Each step records its duration + status + summary stats. The caller wires this to a
//! `tauri::Channel<CrownPipelineProgress>` for live UI feedback.

use crate::bottom::{generate_bottom_offset, BottomParams};
use crate::feedback::{evaluate_tooth, ToothFeedbackReport};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tlanticad_geometry::insertion::{detect_insertion_axis, InsertionAxis};
use tlanticad_mesh::margin::detect_from_curvature;
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::{create_box, Mesh};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PipelineStep {
    MarginDetect,
    InsertionAxis,
    BottomGenerate,
    LibraryFit,
    OcclusionCheck,
    ApproximalCheck,
    ConnectorWeld,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Ok,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step: PipelineStep,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrownPipelineConfig {
    pub fdi: u8,
    pub material: String,
    /// Material-aware defaults for bottom gap; can be overridden.
    #[serde(default)]
    pub bottom_overrides: Option<BottomParams>,
    /// Hint for the occlusal "up" direction. Used by insertion + bottom.
    pub occlusal_hint: [f64; 3],
    /// Curvature threshold for margin detect (mm⁻¹). Default 0.6.
    #[serde(default = "default_curv")]
    pub margin_curvature_threshold: f64,
    /// Library tooth bbox target — when fitting, the library is scaled to this many mm tall.
    #[serde(default = "default_lib_height")]
    pub library_target_height_mm: f64,
}

fn default_curv() -> f64 {
    0.6
}
fn default_lib_height() -> f64 {
    8.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrownPipelineSummary {
    pub margin_polyline_count: usize,
    pub margin_total_length_mm: f64,
    pub insertion_axis: Option<InsertionAxis>,
    pub bottom_vertices_offset: usize,
    pub library_scale_factor: f64,
    pub feedback: Option<ToothFeedbackReport>,
    pub approximal_min_distance_mm: Option<f64>,
    pub connector_triangles: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrownPipelineReport {
    pub fdi: u8,
    pub material: String,
    pub steps: Vec<StepResult>,
    pub summary: CrownPipelineSummary,
    pub has_blocking_error: bool,
}

/// Compute the bounding-box height of a mesh along a given axis.
fn axis_height(mesh: &Mesh, axis: Vector3<f64>) -> f64 {
    if mesh.vertices.is_empty() {
        return 0.0;
    }
    let mut min_proj = f64::INFINITY;
    let mut max_proj = f64::NEG_INFINITY;
    for v in &mesh.vertices {
        let proj = v.coords.dot(&axis);
        if proj < min_proj {
            min_proj = proj;
        }
        if proj > max_proj {
            max_proj = proj;
        }
    }
    (max_proj - min_proj).max(0.0)
}

/// Library-fit step: scale library tooth so its axial height matches `target_height_mm`,
/// then translate so its centroid coincides with `prep_centroid`.
fn fit_library_to_prep(
    library: &Mesh,
    prep_centroid: Point3<f64>,
    axis: Vector3<f64>,
    target_height_mm: f64,
) -> (Mesh, f64) {
    let mut fitted = library.clone();
    fitted.name = format!("{}-fitted", library.name);
    let lib_height = axis_height(library, axis);
    let scale = if lib_height > 1e-6 {
        target_height_mm / lib_height
    } else {
        1.0
    };
    // Centroid before scale.
    let mut sum = Vector3::zeros();
    for p in &fitted.vertices {
        sum += p.coords;
    }
    let lib_centroid = if !fitted.vertices.is_empty() {
        Point3::from(sum / fitted.vertices.len() as f64)
    } else {
        Point3::origin()
    };
    for v in &mut fitted.vertices {
        let local = v.coords - lib_centroid.coords;
        v.coords = local * scale + prep_centroid.coords;
    }
    fitted.calculate_normals();
    (fitted, scale)
}

/// Compute approximal distance: minimum vertex-to-vertex distance from `tooth_outer` to a
/// neighbour mesh. Returns None when no neighbour is supplied.
fn approximal_min_distance(tooth_outer: &Mesh, neighbour: Option<&Mesh>) -> Option<f64> {
    let n = neighbour?;
    if tooth_outer.vertices.is_empty() || n.vertices.is_empty() {
        return None;
    }
    let mut best = f64::INFINITY;
    for a in &tooth_outer.vertices {
        for b in &n.vertices {
            let d2 = (a - b).norm_squared();
            if d2 < best {
                best = d2;
            }
        }
    }
    Some(best.sqrt())
}

/// Inputs for a pipeline run. `prep_mesh` and `library_tooth` are required; everything else
/// is optional and downstream steps degrade gracefully.
pub struct PipelineInputs<'a> {
    pub config: CrownPipelineConfig,
    pub prep_mesh: &'a Mesh,
    pub library_tooth: Option<&'a Mesh>,
    pub antagonist: Option<&'a Mesh>,
    pub mesial_neighbour: Option<&'a Mesh>,
}

/// Run the full pipeline. Each step contributes to `summary`; failures register an Error step
/// but the pipeline continues so partial UI feedback is preserved.
pub fn run_pipeline(inputs: &PipelineInputs) -> (CrownPipelineReport, Mesh) {
    let mut report = CrownPipelineReport {
        fdi: inputs.config.fdi,
        material: inputs.config.material.clone(),
        steps: Vec::new(),
        summary: CrownPipelineSummary::default(),
        has_blocking_error: false,
    };
    let mut crown_outer = create_box(Point3::origin(), Point3::origin()); // placeholder

    // Step 1: margin detect.
    let t = Instant::now();
    let occlusal_hint = Vector3::new(
        inputs.config.occlusal_hint[0],
        inputs.config.occlusal_hint[1],
        inputs.config.occlusal_hint[2],
    );
    let margin_lines = detect_from_curvature(
        inputs.prep_mesh,
        occlusal_hint,
        inputs.config.margin_curvature_threshold,
        0.6,
    );
    let margin_total: f64 = margin_lines.iter().map(|l| l.length_mm()).sum();
    report.summary.margin_polyline_count = margin_lines.len();
    report.summary.margin_total_length_mm = margin_total;
    report.steps.push(StepResult {
        step: PipelineStep::MarginDetect,
        status: if margin_lines.is_empty() {
            StepStatus::Error
        } else {
            StepStatus::Ok
        },
        duration_ms: t.elapsed().as_millis() as u64,
        message: format!(
            "{} polylines, {:.2} mm total",
            margin_lines.len(),
            margin_total
        ),
    });

    // Step 2: insertion axis (uses prep normals).
    let t = Instant::now();
    let axis = detect_insertion_axis(&inputs.prep_mesh.normals, occlusal_hint);
    report.summary.insertion_axis = axis;
    let axis_vec = match &axis {
        Some(a) => Vector3::new(a.axis[0], a.axis[1], a.axis[2]),
        None => occlusal_hint,
    };
    report.steps.push(StepResult {
        step: PipelineStep::InsertionAxis,
        status: if axis.is_some() {
            StepStatus::Ok
        } else {
            StepStatus::Error
        },
        duration_ms: t.elapsed().as_millis() as u64,
        message: match axis {
            Some(a) => format!("directionality {:.2}", a.directionality),
            None => "no normals".into(),
        },
    });

    // Step 3: bottom generate (only when we have a margin polyline).
    let t = Instant::now();
    let primary_margin = margin_lines.iter().max_by(|a, b| {
        a.length_mm()
            .partial_cmp(&b.length_mm())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let bottom_status = if let Some(margin) = primary_margin {
        let polyline_pts: Vec<Point3<f64>> = margin
            .points
            .iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        let params = inputs
            .config
            .bottom_overrides
            .unwrap_or_else(BottomParams::default);
        let (bottom_mesh, bottom_report) = generate_bottom_offset(
            inputs.prep_mesh,
            &polyline_pts,
            margin.is_closed,
            -axis_vec,
            &params,
        );
        report.summary.bottom_vertices_offset = bottom_report.vertices_offset;
        crown_outer = bottom_mesh;
        StepStatus::Ok
    } else {
        StepStatus::Skipped
    };
    report.steps.push(StepResult {
        step: PipelineStep::BottomGenerate,
        status: bottom_status,
        duration_ms: t.elapsed().as_millis() as u64,
        message: format!(
            "{} vertices offset",
            report.summary.bottom_vertices_offset
        ),
    });

    // Step 4: library-fit (when library tooth supplied).
    let t = Instant::now();
    let library_status = if let Some(lib) = inputs.library_tooth {
        // Compute prep centroid for translation target.
        let mut sum = Vector3::zeros();
        for v in &inputs.prep_mesh.vertices {
            sum += v.coords;
        }
        let prep_centroid = if !inputs.prep_mesh.vertices.is_empty() {
            Point3::from(sum / inputs.prep_mesh.vertices.len() as f64)
        } else {
            Point3::origin()
        };
        let (fitted, scale) = fit_library_to_prep(
            lib,
            prep_centroid,
            axis_vec,
            inputs.config.library_target_height_mm,
        );
        report.summary.library_scale_factor = scale;
        // Library-fit replaces the bottom-derived placeholder for the outer crown.
        if fitted.vertex_count() > 0 {
            crown_outer = fitted;
        }
        StepStatus::Ok
    } else {
        report.summary.library_scale_factor = 1.0;
        StepStatus::Skipped
    };
    report.steps.push(StepResult {
        step: PipelineStep::LibraryFit,
        status: library_status,
        duration_ms: t.elapsed().as_millis() as u64,
        message: format!("scale {:.3}", report.summary.library_scale_factor),
    });

    // Step 5: occlusion / thickness feedback.
    let t = Instant::now();
    if crown_outer.vertex_count() > 0 {
        let feedback = evaluate_tooth(
            inputs.config.fdi,
            &inputs.config.material,
            &crown_outer,
            inputs.prep_mesh,
            inputs.antagonist,
            None,
        );
        if feedback.has_blocking_error {
            report.has_blocking_error = true;
        }
        report.summary.feedback = Some(feedback);
        report.steps.push(StepResult {
            step: PipelineStep::OcclusionCheck,
            status: StepStatus::Ok,
            duration_ms: t.elapsed().as_millis() as u64,
            message: format!(
                "{} warnings",
                report
                    .summary
                    .feedback
                    .as_ref()
                    .map(|f| f.warnings.len())
                    .unwrap_or(0)
            ),
        });
    } else {
        report.steps.push(StepResult {
            step: PipelineStep::OcclusionCheck,
            status: StepStatus::Skipped,
            duration_ms: t.elapsed().as_millis() as u64,
            message: "no crown outer".into(),
        });
    }

    // Step 6: approximal check (vs. neighbour, optional).
    let t = Instant::now();
    let approximal_status = if let Some(min_d) = approximal_min_distance(&crown_outer, inputs.mesial_neighbour) {
        report.summary.approximal_min_distance_mm = Some(min_d);
        StepStatus::Ok
    } else {
        StepStatus::Skipped
    };
    report.steps.push(StepResult {
        step: PipelineStep::ApproximalCheck,
        status: approximal_status,
        duration_ms: t.elapsed().as_millis() as u64,
        message: report
            .summary
            .approximal_min_distance_mm
            .map(|d| format!("min {:.3} mm", d))
            .unwrap_or_else(|| "no neighbour".into()),
    });

    // Step 7: connector weld — placeholder (only invoked when bridge mode is on).
    report.steps.push(StepResult {
        step: PipelineStep::ConnectorWeld,
        status: StepStatus::Skipped,
        duration_ms: 0,
        message: "single-tooth pipeline; connectors run separately for bridges".into(),
    });

    (report, crown_outer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    fn cube_mesh() -> Mesh {
        let mut mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        mesh.calculate_normals();
        mesh
    }

    #[test]
    fn pipeline_records_seven_steps() {
        let prep = cube_mesh();
        let lib = cube_mesh();
        let inputs = PipelineInputs {
            config: CrownPipelineConfig {
                fdi: 16,
                material: "zirconia".into(),
                bottom_overrides: None,
                occlusal_hint: [0.0, 0.0, 1.0],
                margin_curvature_threshold: 0.6,
                library_target_height_mm: 8.0,
            },
            prep_mesh: &prep,
            library_tooth: Some(&lib),
            antagonist: None,
            mesial_neighbour: None,
        };
        let (report, outer) = run_pipeline(&inputs);
        assert_eq!(report.steps.len(), 7);
        // Library-fit must have produced an outer crown.
        assert!(outer.vertex_count() > 0);
        // Library-fit step ran (status Ok), insertion ran (status Ok or Error).
        assert!(report
            .steps
            .iter()
            .any(|s| matches!(s.step, PipelineStep::LibraryFit) && matches!(s.status, StepStatus::Ok)));
    }

    #[test]
    fn pipeline_without_library_skips_library_fit() {
        let prep = cube_mesh();
        let inputs = PipelineInputs {
            config: CrownPipelineConfig {
                fdi: 11,
                material: "emax".into(),
                bottom_overrides: None,
                occlusal_hint: [0.0, 0.0, 1.0],
                margin_curvature_threshold: 0.6,
                library_target_height_mm: 8.0,
            },
            prep_mesh: &prep,
            library_tooth: None,
            antagonist: None,
            mesial_neighbour: None,
        };
        let (report, _) = run_pipeline(&inputs);
        assert!(report
            .steps
            .iter()
            .any(|s| matches!(s.step, PipelineStep::LibraryFit) && matches!(s.status, StepStatus::Skipped)));
    }

    #[test]
    fn axis_height_recovers_extents() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 5.0));
        let h = axis_height(&mesh, Vector3::z());
        assert!((h - 5.0).abs() < 1e-9);
    }

    #[test]
    fn fit_library_scales_to_target() {
        let lib = create_box(Point3::origin(), Point3::new(1.0, 1.0, 4.0));
        let (fitted, scale) = fit_library_to_prep(
            &lib,
            Point3::new(10.0, 10.0, 10.0),
            Vector3::z(),
            8.0,
        );
        assert!((scale - 2.0).abs() < 1e-9);
        let h = axis_height(&fitted, Vector3::z());
        assert!((h - 8.0).abs() < 1e-3);
    }

    #[test]
    fn approximal_distance_zero_for_overlapping() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let d = approximal_min_distance(&a, Some(&b));
        assert!(d.is_some());
        assert!(d.unwrap() < 1e-9);
    }
}

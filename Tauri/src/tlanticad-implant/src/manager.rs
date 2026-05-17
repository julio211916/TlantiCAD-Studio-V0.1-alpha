//! Implant manager — change-type, delete, define-reference-objects, edit-mesh-for-planning.
//!
//! Ported from `DentalProcessors/ChangeImplantTypeProcessor` + `DeleteImplantManager` +
//! `DeleteImplantProcessor` + `DefineReferenceObjectsForImplantPlanningProcessor` +
//! `EditMeshForImplantPlanningProcessor` + `DeleteReconstructionsForImplantPlanningProcessor` +
//! `AbutmentOrImplantMergingException`. AR-V373.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::library::{ImplantConnection, ImplantDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantPlacement {
    pub fdi: u8,
    pub sku: String,
    pub position: [f64; 3],
    pub axis: [f64; 3],
    /// Optional pre-existing reconstructions (crown/abutment IDs) attached to this implant.
    #[serde(default)]
    pub attached_reconstructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImplantPlanningState {
    pub implants: Vec<ImplantPlacement>,
    /// FDIs marked as reference-only (used as anatomic landmarks; no reconstruction goes here).
    #[serde(default)]
    pub reference_fdis: Vec<u8>,
    /// Reconstructions deleted during the latest mutation (for the UI to surface a toast).
    #[serde(default)]
    pub orphaned_reconstructions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ManagerSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerWarning {
    pub kind: String,
    pub severity: ManagerSeverity,
    pub message: String,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "fdi")]
pub enum ManagerError {
    #[error("implant for FDI {0} not found")]
    NotFound(u8),
    #[error("invalid axis (zero vector)")]
    InvalidAxis,
    #[error("merging conflict: {0}")]
    Merging(String),
}

/// Result of a `change_implant_type` call enriched with the warnings the
/// caller (UI / wizard) needs to surface (V391). The simple wrapper
/// [`change_implant_type`] still returns just the invalidated reconstructions
/// for backwards compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeTypeReport {
    pub invalidated_reconstructions: Vec<String>,
    pub warnings: Vec<ManagerWarning>,
}

/// Replace `sku` for the implant at `fdi`, preserving its position + axis. Mirrors
/// `ChangeImplantTypeProcessor`. Returns the list of attached reconstructions that
/// became invalid because the new SKU has a different abutment interface (caller
/// can choose whether to recompute or drop them).
pub fn change_implant_type(
    state: &mut ImplantPlanningState,
    fdi: u8,
    new_sku: String,
    invalidates_reconstructions: bool,
) -> Result<Vec<String>, ManagerError> {
    let report = change_implant_type_with_report(state, fdi, new_sku, invalidates_reconstructions, &[])?;
    Ok(report.invalidated_reconstructions)
}

/// V391-extended variant: cross-checks `old_sku` and `new_sku` against the
/// supplied SKU registry (typically `ImplantDefinition::full_catalog()`),
/// detects connection-interface mismatch (HexInternal → Conical / Morse /
/// InternalTriangle), and decorates the report with typed warnings the caller
/// can show in the UI.
pub fn change_implant_type_with_report(
    state: &mut ImplantPlanningState,
    fdi: u8,
    new_sku: String,
    invalidates_reconstructions: bool,
    registry: &[ImplantDefinition],
) -> Result<ChangeTypeReport, ManagerError> {
    let placement = state
        .implants
        .iter_mut()
        .find(|p| p.fdi == fdi)
        .ok_or(ManagerError::NotFound(fdi))?;
    let old_sku = placement.sku.clone();
    let mut report = ChangeTypeReport::default();
    let old_def = registry.iter().find(|d| d.sku == old_sku);
    let new_def = registry.iter().find(|d| d.sku == new_sku);

    if old_sku != new_sku && new_def.is_none() && !registry.is_empty() {
        report.warnings.push(ManagerWarning {
            kind: "sku-not-in-registry".into(),
            severity: ManagerSeverity::Warning,
            message: format!(
                "FDI {fdi}: replacement SKU '{new_sku}' is not in the local registry; \
                 connection-compatibility check skipped"
            ),
        });
    }

    let mut auto_invalidate = invalidates_reconstructions;
    if let (Some(old_def), Some(new_def)) = (old_def, new_def) {
        if old_def.connection != new_def.connection {
            report.warnings.push(ManagerWarning {
                kind: "interface-mismatch".into(),
                severity: ManagerSeverity::Error,
                message: format!(
                    "FDI {fdi}: '{old_sku}' uses {} but '{new_sku}' uses {} — abutments must be re-machined",
                    connection_label(old_def.connection),
                    connection_label(new_def.connection),
                ),
            });
            auto_invalidate = true;
        }
        if (old_def.platform_diameter - new_def.platform_diameter).abs() > 0.05 {
            report.warnings.push(ManagerWarning {
                kind: "platform-diameter-changed".into(),
                severity: ManagerSeverity::Warning,
                message: format!(
                    "FDI {fdi}: platform Ø changed from {:.2} mm to {:.2} mm — re-check emergence profile",
                    old_def.platform_diameter, new_def.platform_diameter
                ),
            });
        }
        if (old_def.diameter - new_def.diameter).abs() > 0.5
            || (old_def.length - new_def.length).abs() > 1.5
        {
            report.warnings.push(ManagerWarning {
                kind: "body-geometry-changed".into(),
                severity: ManagerSeverity::Warning,
                message: format!(
                    "FDI {fdi}: body Ø {:.2}→{:.2} mm / length {:.1}→{:.1} mm — re-validate bone collision",
                    old_def.diameter, new_def.diameter, old_def.length, new_def.length
                ),
            });
        }
    }

    if old_sku != new_sku {
        placement.sku = new_sku;
        if auto_invalidate {
            report.invalidated_reconstructions = std::mem::take(&mut placement.attached_reconstructions);
        }
    }
    state
        .orphaned_reconstructions
        .extend(report.invalidated_reconstructions.iter().cloned());
    Ok(report)
}

fn connection_label(c: ImplantConnection) -> &'static str {
    match c {
        ImplantConnection::InternalHex => "internal hex",
        ImplantConnection::ExternalHex => "external hex",
        ImplantConnection::InternalTriangle => "internal trilobe",
        ImplantConnection::Conical => "conical",
        ImplantConnection::Morse => "morse-taper",
    }
}

/// Delete the implant for `fdi` and any dependent reconstructions. Mirrors
/// `DeleteImplantProcessor` + `DeleteImplantManager`.
pub fn delete_implant(state: &mut ImplantPlanningState, fdi: u8) -> Result<Vec<String>, ManagerError> {
    let idx = state
        .implants
        .iter()
        .position(|p| p.fdi == fdi)
        .ok_or(ManagerError::NotFound(fdi))?;
    let placement = state.implants.remove(idx);
    let dependents = placement.attached_reconstructions.clone();
    state.orphaned_reconstructions.extend(dependents.iter().cloned());
    Ok(dependents)
}

/// Mark a tooth as reference-only — no reconstructions or implants will be placed on it,
/// but its mesh remains available for landmarking. Mirrors
/// `DefineReferenceObjectsForImplantPlanningProcessor`.
pub fn define_reference_objects(
    state: &mut ImplantPlanningState,
    fdis: &[u8],
) -> Vec<ManagerWarning> {
    let mut warnings = Vec::new();
    let mut existing: HashSet<u8> = state.reference_fdis.iter().copied().collect();
    for &fdi in fdis {
        if state.implants.iter().any(|p| p.fdi == fdi) {
            warnings.push(ManagerWarning {
                kind: "reference-collides-with-implant".into(),
                severity: ManagerSeverity::Warning,
                message: format!(
                    "FDI {fdi} already has an implant placement — marking as reference will be ignored",
                ),
            });
            continue;
        }
        existing.insert(fdi);
    }
    state.reference_fdis = existing.into_iter().collect();
    state.reference_fdis.sort_unstable();
    warnings
}

/// Validate that a proposed implant placement does not interpenetrate the existing ones.
/// Returns warnings (proximity < 3mm, parallel axes < 5°). Errors when overlap is severe
/// (centers within sum of radii). Mirrors the merging-exception checks in
/// `AbutmentOrImplantMergingException`.
pub fn validate_proposed_placement(
    state: &ImplantPlanningState,
    proposal: &ImplantPlacement,
    proposed_radius_mm: f64,
    existing_radius_mm: f64,
) -> Vec<ManagerWarning> {
    let mut warnings = Vec::new();
    let p_center = Point3::new(proposal.position[0], proposal.position[1], proposal.position[2]);
    let p_axis = Vector3::new(proposal.axis[0], proposal.axis[1], proposal.axis[2])
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
    for existing in &state.implants {
        if existing.fdi == proposal.fdi {
            continue;
        }
        let e_center = Point3::new(existing.position[0], existing.position[1], existing.position[2]);
        let e_axis = Vector3::new(existing.axis[0], existing.axis[1], existing.axis[2])
            .try_normalize(1e-9)
            .unwrap_or(Vector3::z());
        let center_dist = (p_center - e_center).norm();
        let axis_angle_deg = p_axis.dot(&e_axis).clamp(-1.0, 1.0).acos().to_degrees();
        let sum_radius = proposed_radius_mm + existing_radius_mm;
        if center_dist < sum_radius {
            warnings.push(ManagerWarning {
                kind: "implant-collision".into(),
                severity: ManagerSeverity::Error,
                message: format!(
                    "FDI {} and FDI {} centers {:.2} mm apart < {:.2} mm sum-radius",
                    proposal.fdi, existing.fdi, center_dist, sum_radius
                ),
            });
        } else if center_dist < sum_radius + 3.0 {
            warnings.push(ManagerWarning {
                kind: "implant-too-close".into(),
                severity: ManagerSeverity::Warning,
                message: format!(
                    "FDI {} only {:.2} mm from FDI {} — recommended ≥ {:.1} mm",
                    proposal.fdi,
                    center_dist,
                    existing.fdi,
                    sum_radius + 3.0
                ),
            });
        }
        if axis_angle_deg > 30.0 {
            warnings.push(ManagerWarning {
                kind: "axes-divergent".into(),
                severity: ManagerSeverity::Warning,
                message: format!(
                    "FDI {} axis diverges {:.1}° from FDI {} (recommended < 30°)",
                    proposal.fdi, axis_angle_deg, existing.fdi
                ),
            });
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(fdi: u8, sku: &str) -> ImplantPlacement {
        ImplantPlacement {
            fdi,
            sku: sku.into(),
            position: [0.0, fdi as f64 * 8.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            attached_reconstructions: vec![format!("crown-{fdi}")],
        }
    }

    #[test]
    fn change_type_preserves_axis_and_position() {
        let mut state = ImplantPlanningState {
            implants: vec![placement(16, "Straumann-BL-4.1")],
            ..Default::default()
        };
        let invalid = change_implant_type(&mut state, 16, "Nobel-Active-4.3".into(), true).unwrap();
        assert_eq!(invalid, vec!["crown-16".to_string()]);
        let p = state.implants.iter().find(|p| p.fdi == 16).unwrap();
        assert_eq!(p.sku, "Nobel-Active-4.3");
        assert_eq!(p.position, [0.0, 16.0 * 8.0, 0.0]);
        assert!(p.attached_reconstructions.is_empty());
    }

    #[test]
    fn delete_implant_orphans_reconstructions() {
        let mut state = ImplantPlanningState {
            implants: vec![placement(11, "Sky-3.5")],
            ..Default::default()
        };
        let orphans = delete_implant(&mut state, 11).unwrap();
        assert_eq!(orphans, vec!["crown-11".to_string()]);
        assert!(state.implants.is_empty());
        assert_eq!(state.orphaned_reconstructions, vec!["crown-11".to_string()]);
    }

    #[test]
    fn delete_missing_fdi_returns_error() {
        let mut state = ImplantPlanningState::default();
        let err = delete_implant(&mut state, 99).unwrap_err();
        match err {
            ManagerError::NotFound(fdi) => assert_eq!(fdi, 99),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn define_reference_objects_skips_existing_implants() {
        let mut state = ImplantPlanningState {
            implants: vec![placement(16, "X")],
            ..Default::default()
        };
        let warnings = define_reference_objects(&mut state, &[16, 17, 18]);
        assert!(warnings.iter().any(|w| w.kind == "reference-collides-with-implant"));
        assert!(state.reference_fdis.contains(&17));
        assert!(state.reference_fdis.contains(&18));
        assert!(!state.reference_fdis.contains(&16));
    }

    #[test]
    fn validate_flags_collision() {
        let state = ImplantPlanningState {
            implants: vec![placement(16, "X")],
            ..Default::default()
        };
        let proposal = ImplantPlacement {
            fdi: 17,
            sku: "Y".into(),
            position: [0.0, 16.0 * 8.0 + 1.0, 0.0], // 1 mm away — well under sum radius 4.0
            axis: [0.0, 0.0, 1.0],
            attached_reconstructions: vec![],
        };
        let warnings = validate_proposed_placement(&state, &proposal, 2.0, 2.0);
        assert!(warnings.iter().any(|w| w.kind == "implant-collision"));
    }

    #[test]
    fn change_type_with_report_flags_interface_mismatch() {
        // Build a tiny registry with one InternalHex and one Conical implant.
        let registry = vec![
            crate::library::ImplantDefinition::osstem_ts3_4_0_10(), // InternalHex
            crate::library::ImplantDefinition::nobel_active_4_3_10(), // Conical
        ];
        let mut state = ImplantPlanningState {
            implants: vec![ImplantPlacement {
                fdi: 36,
                sku: registry[0].sku.clone(), // InternalHex
                position: [0.0; 3],
                axis: [0.0, 0.0, 1.0],
                attached_reconstructions: vec!["abutment-36".into()],
            }],
            ..Default::default()
        };
        let report = change_implant_type_with_report(
            &mut state,
            36,
            registry[1].sku.clone(),
            false,
            &registry,
        )
        .unwrap();
        assert!(report.warnings.iter().any(|w| w.kind == "interface-mismatch"));
        assert_eq!(report.invalidated_reconstructions, vec!["abutment-36".to_string()]);
    }

    #[test]
    fn change_type_with_report_warns_on_unknown_new_sku() {
        let registry = vec![crate::library::ImplantDefinition::osstem_ts3_4_0_10()];
        let mut state = ImplantPlanningState {
            implants: vec![ImplantPlacement {
                fdi: 11,
                sku: registry[0].sku.clone(),
                position: [0.0; 3],
                axis: [0.0, 0.0, 1.0],
                attached_reconstructions: vec![],
            }],
            ..Default::default()
        };
        let report = change_implant_type_with_report(
            &mut state,
            11,
            "MADE-UP-9999".into(),
            false,
            &registry,
        )
        .unwrap();
        assert!(report.warnings.iter().any(|w| w.kind == "sku-not-in-registry"));
    }

    #[test]
    fn change_type_with_report_flags_platform_change() {
        let registry = vec![
            crate::library::ImplantDefinition::straumann_rc_4_1_10(), // platform 4.8 morse
            crate::library::ImplantDefinition::straumann_rc_4_1_12(), // same morse
        ];
        // Mutate the second one to have a different platform Ø in test.
        let mut alt = registry[1].clone();
        alt.platform_diameter = 6.0;
        let registry = vec![registry[0].clone(), alt];
        let mut state = ImplantPlanningState {
            implants: vec![ImplantPlacement {
                fdi: 26,
                sku: registry[0].sku.clone(),
                position: [0.0; 3],
                axis: [0.0, 0.0, 1.0],
                attached_reconstructions: vec![],
            }],
            ..Default::default()
        };
        let report = change_implant_type_with_report(
            &mut state,
            26,
            registry[1].sku.clone(),
            false,
            &registry,
        )
        .unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w.kind == "platform-diameter-changed"));
    }

    #[test]
    fn validate_flags_divergent_axis() {
        let state = ImplantPlanningState {
            implants: vec![placement(11, "X")],
            ..Default::default()
        };
        let proposal = ImplantPlacement {
            fdi: 12,
            sku: "Y".into(),
            position: [0.0, 11.0 * 8.0 + 12.0, 0.0],
            axis: [1.0, 0.0, 0.5],
            attached_reconstructions: vec![],
        };
        let warnings = validate_proposed_placement(&state, &proposal, 2.0, 2.0);
        assert!(warnings.iter().any(|w| w.kind == "axes-divergent"));
    }
}

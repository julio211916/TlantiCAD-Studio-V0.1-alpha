//! Reference-object definition for implant planning. AR-V388.
//!
//! Ported from `DentalProcessors/DefineReferenceObjectsForImplantPlanningProcessor.cs`
//! (and its dongle-gated `GetApplicableTeeth` / `FinalizeAction` lifecycle, plus the
//! `ScaleDirectionObject` helper which manipulates a per-object world transform). The
//! original processor only ran when:
//!
//!   1. `ImplantPlanningMode` was on,
//!   2. The jaw had a valid `JawPlane`, finite `JawAxisTop/Left/Front`, and a ridge
//!      spline, and
//!   3. At least one DICOM scan was loaded.
//!
//! In TlantiCAD we don't have a DentalData runtime, so we model the equivalent by
//! validating a typed `ReferenceObjectSpec` set against an `ImplantPlanningState` and
//! emitting structured `ReferenceWarning`s the UI can surface. The processor's
//! "reference object" had two flavours visible in `ImplantPlanningGeneralReferenceDirection`
//! and `ImplantPlanningGeneralReferencePlane`; we expose four roles that match how the
//! exocad wizard surfaces them in the wizard step (anatomic landmarks above the ridge,
//! occlusal plane reference, soft-tissue landmarks, and root-apex references).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::manager::ImplantPlanningState;

/// What the reference object represents semantically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceRole {
    /// Bony landmark visible on the DICOM (e.g. mental foramen, incisive canal).
    AnatomicLandmark,
    /// Plane / curve marking the occlusal reference for axis alignment.
    OcclusalReference,
    /// Soft-tissue landmark (papilla tip, marginal gingiva apex).
    SoftTissueLandmark,
    /// Root apex / radicular reference for adjacent natural teeth.
    RootReference,
}

impl ReferenceRole {
    fn label(self) -> &'static str {
        match self {
            ReferenceRole::AnatomicLandmark => "anatomic-landmark",
            ReferenceRole::OcclusalReference => "occlusal-reference",
            ReferenceRole::SoftTissueLandmark => "soft-tissue-landmark",
            ReferenceRole::RootReference => "root-reference",
        }
    }
}

/// User-defined reference object: pinned to an FDI tooth, pulling its mesh from
/// `source_mesh` (path on disk, e.g. an exported segment STL). The XAML processor
/// stored the equivalent as a `ToothPart` of type
/// `ImplantPlanningGeneralReferenceDirection` with handles + `MatrixToWorld`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceObjectSpec {
    pub fdi: u8,
    pub role: ReferenceRole,
    pub source_mesh: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceWarning {
    pub kind: String,
    pub severity: ReferenceSeverity,
    pub fdi: Option<u8>,
    pub message: String,
}

fn is_valid_fdi(fdi: u8) -> bool {
    let q = fdi / 10;
    let n = fdi % 10;
    matches!(q, 1..=4) && (1..=8).contains(&n)
}

/// Validate a proposed reference set against the current implant planning state.
///
/// Mirrors `DefineReferenceObjectsForImplantPlanningProcessor.GetApplicableTeeth`'s
/// gate logic and adds typed warnings for the ones the original UI surfaced as
/// silent failures (FDI not in the jaw, mesh path missing, role duplicated, FDI
/// already has an implant placement, occlusal-reference set empty, …).
pub fn validate_reference_set(
    specs: &[ReferenceObjectSpec],
    state: &ImplantPlanningState,
) -> Vec<ReferenceWarning> {
    let mut warnings: Vec<ReferenceWarning> = Vec::new();

    if specs.is_empty() {
        warnings.push(ReferenceWarning {
            kind: "no-references".into(),
            severity: ReferenceSeverity::Warning,
            fdi: None,
            message: "Implant planning has no reference objects defined".into(),
        });
        return warnings;
    }

    let implant_fdis: HashSet<u8> = state.implants.iter().map(|p| p.fdi).collect();
    let mut seen_pairs: HashSet<(u8, ReferenceRole)> = HashSet::new();
    let mut occlusal_count = 0usize;

    for spec in specs {
        if !is_valid_fdi(spec.fdi) {
            warnings.push(ReferenceWarning {
                kind: "invalid-fdi".into(),
                severity: ReferenceSeverity::Error,
                fdi: Some(spec.fdi),
                message: format!("FDI {} is not a valid permanent-tooth code", spec.fdi),
            });
            continue;
        }

        if !seen_pairs.insert((spec.fdi, spec.role)) {
            warnings.push(ReferenceWarning {
                kind: "duplicate-reference".into(),
                severity: ReferenceSeverity::Warning,
                fdi: Some(spec.fdi),
                message: format!(
                    "FDI {} already has a {} reference — the previous one will be replaced",
                    spec.fdi,
                    spec.role.label()
                ),
            });
        }

        if implant_fdis.contains(&spec.fdi) {
            warnings.push(ReferenceWarning {
                kind: "reference-collides-with-implant".into(),
                severity: ReferenceSeverity::Warning,
                fdi: Some(spec.fdi),
                message: format!(
                    "FDI {} has an implant placement; using it as a {} reference may bias the axis fit",
                    spec.fdi,
                    spec.role.label()
                ),
            });
        }

        if spec.source_mesh.as_os_str().is_empty() {
            warnings.push(ReferenceWarning {
                kind: "missing-mesh".into(),
                severity: ReferenceSeverity::Error,
                fdi: Some(spec.fdi),
                message: format!(
                    "FDI {} {} reference has no source mesh path",
                    spec.fdi,
                    spec.role.label()
                ),
            });
        }

        if matches!(spec.role, ReferenceRole::OcclusalReference) {
            occlusal_count += 1;
        }
    }

    if occlusal_count == 0 {
        warnings.push(ReferenceWarning {
            kind: "missing-occlusal-reference".into(),
            severity: ReferenceSeverity::Warning,
            fdi: None,
            message: "No occlusal reference defined — implant axis fit will fall back to jaw plane".into(),
        });
    } else if occlusal_count > 2 {
        warnings.push(ReferenceWarning {
            kind: "too-many-occlusal-references".into(),
            severity: ReferenceSeverity::Info,
            fdi: None,
            message: format!(
                "{} occlusal references defined — only the first 2 will drive the plane fit",
                occlusal_count
            ),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::ImplantPlacement;

    fn spec(fdi: u8, role: ReferenceRole, path: &str) -> ReferenceObjectSpec {
        ReferenceObjectSpec {
            fdi,
            role,
            source_mesh: PathBuf::from(path),
        }
    }

    #[test]
    fn empty_spec_set_warns() {
        let state = ImplantPlanningState::default();
        let w = validate_reference_set(&[], &state);
        assert!(w.iter().any(|x| x.kind == "no-references"));
    }

    #[test]
    fn invalid_fdi_is_error() {
        let state = ImplantPlanningState::default();
        let specs = [spec(99, ReferenceRole::AnatomicLandmark, "/tmp/x.stl")];
        let w = validate_reference_set(&specs, &state);
        assert!(w.iter().any(|x| x.kind == "invalid-fdi" && x.severity == ReferenceSeverity::Error));
    }

    #[test]
    fn occlusal_missing_emits_warning() {
        let state = ImplantPlanningState::default();
        let specs = [
            spec(11, ReferenceRole::AnatomicLandmark, "/tmp/a.stl"),
            spec(21, ReferenceRole::SoftTissueLandmark, "/tmp/b.stl"),
        ];
        let w = validate_reference_set(&specs, &state);
        assert!(w.iter().any(|x| x.kind == "missing-occlusal-reference"));
    }

    #[test]
    fn duplicate_role_for_fdi_warns() {
        let state = ImplantPlanningState::default();
        let specs = [
            spec(16, ReferenceRole::AnatomicLandmark, "/tmp/a.stl"),
            spec(16, ReferenceRole::AnatomicLandmark, "/tmp/b.stl"),
            spec(26, ReferenceRole::OcclusalReference, "/tmp/c.stl"),
        ];
        let w = validate_reference_set(&specs, &state);
        assert!(w.iter().any(|x| x.kind == "duplicate-reference"));
    }

    #[test]
    fn implant_fdi_used_as_reference_warns() {
        let state = ImplantPlanningState {
            implants: vec![ImplantPlacement {
                fdi: 36,
                sku: "X".into(),
                position: [0.0; 3],
                axis: [0.0, 0.0, 1.0],
                attached_reconstructions: vec![],
            }],
            ..Default::default()
        };
        let specs = [
            spec(36, ReferenceRole::RootReference, "/tmp/r.stl"),
            spec(46, ReferenceRole::OcclusalReference, "/tmp/o.stl"),
        ];
        let w = validate_reference_set(&specs, &state);
        assert!(w.iter().any(|x| x.kind == "reference-collides-with-implant" && x.fdi == Some(36)));
    }

    #[test]
    fn missing_mesh_path_is_error() {
        let state = ImplantPlanningState::default();
        let specs = [spec(11, ReferenceRole::OcclusalReference, "")];
        let w = validate_reference_set(&specs, &state);
        assert!(w.iter().any(|x| x.kind == "missing-mesh" && x.severity == ReferenceSeverity::Error));
    }
}

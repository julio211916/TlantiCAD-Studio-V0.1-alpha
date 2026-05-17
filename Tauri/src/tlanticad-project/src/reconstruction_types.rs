//! AR-V406 — Reconstruction-type changes for a case state.
//!
//! Ported from `DentalProcessors/AlmightyChangeReconstructionType.cs`. Exocad's
//! "Almighty Change Reconstruction Type" is the unified entry-point that lets
//! a technician retype a tooth slot mid-design (Crown ↔ Bridge ↔ Onlay ↔ …).
//! Some transitions are illegal (e.g. you can't turn a Pontic into an Implant
//! because the latter requires a screw channel & abutment); others are legal
//! but require clearing parameters that are no longer applicable.
//!
//! This module exposes:
//!   * `ReconstructionType` — the 11 supported types (matches exocad's list).
//!   * `ToothCaseState` — minimal per-tooth state we need to validate the change.
//!   * `change_type` — runs the validation + clearing rules and returns a
//!      `ChangeReport`.

use serde::{Deserialize, Serialize};

/// Restoration / reconstruction type for a single tooth slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReconstructionType {
    Crown,
    Bridge,
    Inlay,
    Onlay,
    Veneer,
    Pontic,
    Implant,
    Abutment,
    Telescope,
    Bar,
    BiteSplint,
}

impl ReconstructionType {
    /// Stable string identifier (used in interop XML and command palette IDs).
    pub fn id(self) -> &'static str {
        match self {
            ReconstructionType::Crown => "crown",
            ReconstructionType::Bridge => "bridge",
            ReconstructionType::Inlay => "inlay",
            ReconstructionType::Onlay => "onlay",
            ReconstructionType::Veneer => "veneer",
            ReconstructionType::Pontic => "pontic",
            ReconstructionType::Implant => "implant",
            ReconstructionType::Abutment => "abutment",
            ReconstructionType::Telescope => "telescope",
            ReconstructionType::Bar => "bar",
            ReconstructionType::BiteSplint => "bite_splint",
        }
    }

    /// True when the type physically replaces tooth structure on the
    /// preparation (Crown / Inlay / Onlay / Veneer).
    pub fn requires_preparation_scan(self) -> bool {
        matches!(
            self,
            ReconstructionType::Crown
                | ReconstructionType::Inlay
                | ReconstructionType::Onlay
                | ReconstructionType::Veneer
        )
    }

    /// True when the type sits on an implant fixture (Implant / Abutment / Bar).
    pub fn requires_implant_fixture(self) -> bool {
        matches!(
            self,
            ReconstructionType::Implant
                | ReconstructionType::Abutment
                | ReconstructionType::Bar
        )
    }

    /// True when the type spans multiple FDI positions (Bridge / Bar).
    pub fn is_multi_unit(self) -> bool {
        matches!(self, ReconstructionType::Bridge | ReconstructionType::Bar)
    }
}

/// Per-tooth state, intentionally minimal — we keep just enough to validate
/// transitions and to know what to clear. Real tooth state lives in the
/// frontend store (`tlantidb-case-store`); this struct is the audit-trail
/// shape exocad uses internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothCaseState {
    pub fdi: u8,
    pub recon_type: ReconstructionType,
    /// Material id (e.g. "zirconia", "emax", "pmma") — cleared on type change
    /// when the new type doesn't accept the previous material category.
    pub material: Option<String>,
    /// Cement gap in micrometers — cleared if new type doesn't have a cement
    /// gap (e.g. Pontic → Bar).
    pub cement_gap_um: Option<f64>,
    /// Implant fixture id — required for Implant/Abutment/Bar; cleared when
    /// the new type doesn't need a fixture.
    pub implant_fixture_id: Option<String>,
    /// Connector ids when the tooth is part of a multi-unit (Bridge/Bar);
    /// cleared on transition to single-unit.
    pub connector_ids: Vec<String>,
    /// Margin-line completed flag.
    pub margin_completed: bool,
    /// Insertion direction completed flag.
    pub insertion_completed: bool,
}

impl ToothCaseState {
    pub fn new(fdi: u8, recon_type: ReconstructionType) -> Self {
        Self {
            fdi,
            recon_type,
            material: None,
            cement_gap_um: None,
            implant_fixture_id: None,
            connector_ids: Vec::new(),
            margin_completed: false,
            insertion_completed: false,
        }
    }
}

/// Whole-case state — wraps a list of tooth states keyed by FDI.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaseState {
    pub teeth: Vec<ToothCaseState>,
}

impl CaseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, t: ToothCaseState) {
        if let Some(slot) = self.teeth.iter_mut().find(|s| s.fdi == t.fdi) {
            *slot = t;
        } else {
            self.teeth.push(t);
        }
    }

    pub fn get(&self, fdi: u8) -> Option<&ToothCaseState> {
        self.teeth.iter().find(|s| s.fdi == fdi)
    }

    pub fn get_mut(&mut self, fdi: u8) -> Option<&mut ToothCaseState> {
        self.teeth.iter_mut().find(|s| s.fdi == fdi)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChangeError {
    #[error("Diente FDI {0} no presente en el caso")]
    ToothNotFound(u8),
    #[error("Transición ilegal: {from:?} → {to:?}")]
    IllegalTransition {
        from: ReconstructionType,
        to: ReconstructionType,
    },
    #[error("Falta fixture de implante para {to:?}")]
    MissingImplantFixture { to: ReconstructionType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReport {
    pub fdi: u8,
    pub from: ReconstructionType,
    pub to: ReconstructionType,
    pub cleared_material: bool,
    pub cleared_cement_gap: bool,
    pub cleared_implant_fixture: bool,
    pub cleared_connectors: bool,
    pub reset_margin_flag: bool,
    pub reset_insertion_flag: bool,
}

/// Decide whether `from → to` is allowed. Mirrors exocad's transition matrix:
///
/// * Anything → Anything is allowed EXCEPT:
///   * Pontic → Implant / Abutment / Bar (no fixture, no preparation).
///   * Implant / Abutment → Inlay / Onlay / Veneer (these need natural tooth structure).
///   * BiteSplint → anything else (bite splint is a full-arch device, retyping is not in scope).
fn is_legal(from: ReconstructionType, to: ReconstructionType) -> bool {
    use ReconstructionType::*;
    if from == to {
        return true;
    }
    if from == BiteSplint || to == BiteSplint {
        // Switching to/from a bite splint is structurally different (full-arch).
        return false;
    }
    match (from, to) {
        (Pontic, Implant) | (Pontic, Abutment) | (Pontic, Bar) => false,
        (Implant, Inlay) | (Implant, Onlay) | (Implant, Veneer) => false,
        (Abutment, Inlay) | (Abutment, Onlay) | (Abutment, Veneer) => false,
        _ => true,
    }
}

/// Change the reconstruction type of one tooth in `case_state`. Validates the
/// transition and clears now-irrelevant parameters per exocad rules.
///
/// On error the case is left UNTOUCHED (validation runs first).
pub fn change_type(
    case_state: &mut CaseState,
    fdi: u8,
    new_type: ReconstructionType,
) -> Result<ChangeReport, ChangeError> {
    let tooth = case_state
        .get(fdi)
        .ok_or(ChangeError::ToothNotFound(fdi))?
        .clone();
    let from = tooth.recon_type;
    if !is_legal(from, new_type) {
        return Err(ChangeError::IllegalTransition {
            from,
            to: new_type,
        });
    }
    // If the new type needs a fixture but the tooth has none AND the old type
    // didn't carry one either, this is the moment we tell the caller they
    // need to attach a fixture before the change can land. We DON'T pre-fill
    // a fictitious fixture id — that would be a stub (against project rules).
    if new_type.requires_implant_fixture() && tooth.implant_fixture_id.is_none() {
        return Err(ChangeError::MissingImplantFixture { to: new_type });
    }

    // Clear logic — driven by what the NEW type does NOT need.
    let mut report = ChangeReport {
        fdi,
        from,
        to: new_type,
        cleared_material: false,
        cleared_cement_gap: false,
        cleared_implant_fixture: false,
        cleared_connectors: false,
        reset_margin_flag: false,
        reset_insertion_flag: false,
    };

    // Now perform the mutation.
    let slot = case_state.get_mut(fdi).expect("just verified above");
    slot.recon_type = new_type;

    // Cement gap is only meaningful for crowns / inlays / onlays / veneers /
    // bridges / abutments / telescopes. Drop it otherwise.
    let needs_cement_gap = matches!(
        new_type,
        ReconstructionType::Crown
            | ReconstructionType::Inlay
            | ReconstructionType::Onlay
            | ReconstructionType::Veneer
            | ReconstructionType::Bridge
            | ReconstructionType::Abutment
            | ReconstructionType::Telescope
    );
    if !needs_cement_gap && slot.cement_gap_um.is_some() {
        slot.cement_gap_um = None;
        report.cleared_cement_gap = true;
    }

    // Implant fixture id: drop when the new type doesn't need one.
    if !new_type.requires_implant_fixture() && slot.implant_fixture_id.is_some() {
        slot.implant_fixture_id = None;
        report.cleared_implant_fixture = true;
    }

    // Connectors: drop on transition to single-unit.
    if !new_type.is_multi_unit() && !slot.connector_ids.is_empty() {
        slot.connector_ids.clear();
        report.cleared_connectors = true;
    }

    // Material: clear when transitioning between physically incompatible
    // categories. Pontic and Bridge/Bar share zirconia/PMMA universes; Implant
    // requires a different material set (titanium / cobalt-chrome). To stay
    // conservative we clear material whenever moving to/from
    // Implant/Abutment/Bar so the technician must pick again.
    let new_is_metal_zone = matches!(
        new_type,
        ReconstructionType::Implant | ReconstructionType::Abutment | ReconstructionType::Bar
    );
    let from_was_metal_zone = matches!(
        from,
        ReconstructionType::Implant | ReconstructionType::Abutment | ReconstructionType::Bar
    );
    if new_is_metal_zone != from_was_metal_zone && slot.material.is_some() {
        slot.material = None;
        report.cleared_material = true;
    }

    // Margin/insertion flags: keep when both types still need a preparation
    // scan; otherwise reset.
    if !new_type.requires_preparation_scan() {
        if slot.margin_completed {
            slot.margin_completed = false;
            report.reset_margin_flag = true;
        }
        if slot.insertion_completed {
            slot.insertion_completed = false;
            report.reset_insertion_flag = true;
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case_with(state: ToothCaseState) -> CaseState {
        let mut c = CaseState::new();
        c.upsert(state);
        c
    }

    #[test]
    fn change_crown_to_inlay_keeps_margin_clears_nothing_extra() {
        let mut tooth = ToothCaseState::new(16, ReconstructionType::Crown);
        tooth.material = Some("zirconia".into());
        tooth.cement_gap_um = Some(50.0);
        tooth.margin_completed = true;
        let mut case = case_with(tooth);

        let report = change_type(&mut case, 16, ReconstructionType::Inlay).unwrap();
        assert_eq!(report.from, ReconstructionType::Crown);
        assert_eq!(report.to, ReconstructionType::Inlay);
        assert!(!report.cleared_material);
        assert!(!report.cleared_cement_gap);
        assert!(!report.reset_margin_flag);
        let after = case.get(16).unwrap();
        assert_eq!(after.material.as_deref(), Some("zirconia"));
        assert!(after.margin_completed);
    }

    #[test]
    fn change_crown_to_pontic_clears_margin_keeps_no_fixture() {
        let mut tooth = ToothCaseState::new(16, ReconstructionType::Crown);
        tooth.cement_gap_um = Some(50.0);
        tooth.margin_completed = true;
        tooth.insertion_completed = true;
        let mut case = case_with(tooth);

        let report = change_type(&mut case, 16, ReconstructionType::Pontic).unwrap();
        assert!(report.cleared_cement_gap);
        assert!(report.reset_margin_flag);
        assert!(report.reset_insertion_flag);
        let after = case.get(16).unwrap();
        assert!(after.cement_gap_um.is_none());
        assert!(!after.margin_completed);
        assert!(!after.insertion_completed);
    }

    #[test]
    fn pontic_to_implant_is_illegal() {
        let mut case = case_with(ToothCaseState::new(16, ReconstructionType::Pontic));
        let err = change_type(&mut case, 16, ReconstructionType::Implant).unwrap_err();
        assert_eq!(
            err,
            ChangeError::IllegalTransition {
                from: ReconstructionType::Pontic,
                to: ReconstructionType::Implant
            }
        );
        // State unchanged on error.
        assert_eq!(case.get(16).unwrap().recon_type, ReconstructionType::Pontic);
    }

    #[test]
    fn change_to_implant_without_fixture_errors() {
        let mut case = case_with(ToothCaseState::new(16, ReconstructionType::Crown));
        let err = change_type(&mut case, 16, ReconstructionType::Implant).unwrap_err();
        assert_eq!(
            err,
            ChangeError::MissingImplantFixture {
                to: ReconstructionType::Implant
            }
        );
    }

    #[test]
    fn change_to_implant_with_fixture_clears_material() {
        let mut tooth = ToothCaseState::new(16, ReconstructionType::Crown);
        tooth.material = Some("zirconia".into());
        tooth.cement_gap_um = Some(50.0);
        tooth.implant_fixture_id = Some("nobel-replace-rp".into());
        tooth.margin_completed = true;
        let mut case = case_with(tooth);

        let report = change_type(&mut case, 16, ReconstructionType::Implant).unwrap();
        assert!(report.cleared_material);
        assert!(report.cleared_cement_gap);
        assert!(report.reset_margin_flag);
        let after = case.get(16).unwrap();
        assert!(after.material.is_none());
        assert!(after.cement_gap_um.is_none());
        assert!(after.implant_fixture_id.is_some());
    }

    #[test]
    fn changing_a_missing_tooth_returns_not_found() {
        let mut case = CaseState::new();
        let err = change_type(&mut case, 99, ReconstructionType::Crown).unwrap_err();
        assert_eq!(err, ChangeError::ToothNotFound(99));
    }

    #[test]
    fn bite_splint_transitions_blocked_in_both_directions() {
        let mut case = case_with(ToothCaseState::new(16, ReconstructionType::BiteSplint));
        let err = change_type(&mut case, 16, ReconstructionType::Crown).unwrap_err();
        assert!(matches!(err, ChangeError::IllegalTransition { .. }));

        let mut case2 = case_with(ToothCaseState::new(16, ReconstructionType::Crown));
        let err2 = change_type(&mut case2, 16, ReconstructionType::BiteSplint).unwrap_err();
        assert!(matches!(err2, ChangeError::IllegalTransition { .. }));
    }

    #[test]
    fn bridge_to_crown_clears_connectors() {
        let mut tooth = ToothCaseState::new(16, ReconstructionType::Bridge);
        tooth.connector_ids = vec!["c-15-16".into(), "c-16-17".into()];
        let mut case = case_with(tooth);

        let report = change_type(&mut case, 16, ReconstructionType::Crown).unwrap();
        assert!(report.cleared_connectors);
        assert!(case.get(16).unwrap().connector_ids.is_empty());
    }
}

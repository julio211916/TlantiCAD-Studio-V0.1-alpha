//! S271-S275: Removable Prosthetics
//!
//! RPD framework design, clasps, major connectors,
//! retention mesh, and surveying tools.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kennedy classification for partial edentulism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KennedyClass {
    ClassI,
    ClassII,
    ClassIII,
    ClassIV,
}

/// Major connector type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MajorConnectorType {
    // Maxillary
    PalatalStrap,
    AnteroPosteriorPalatalStrap,
    PalatalPlate,
    HorseshoeConnector,
    // Mandibular
    LingualBar,
    LingualPlate,
    DoubleLingualBar,
    LabialBar,
}

/// Clasp type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaspType {
    CircumferentialAkers,
    RoachI,        // T-bar
    RoachL,        // L-bar
    RoachY,        // Y-bar
    BackAction,
    RingClasp,
    Wrought,
    RPI,           // Rest, proximal plate, I-bar
    Combination,
}

/// A clasp element on the RPD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clasp {
    pub id: Uuid,
    pub tooth_id: String,
    pub clasp_type: ClaspType,
    pub survey_line_depth_mm: f64,
    pub tip_position: [f64; 3],
    pub retention_arm_length_mm: f64,
    pub reciprocal_arm: bool,
}

/// Rest seat specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestSeat {
    pub id: Uuid,
    pub tooth_id: String,
    pub rest_type: RestType,
    pub position_mm: [f64; 3],
    pub width_mm: f64,
    pub depth_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestType {
    OcclusalRest,
    CingulumRest,
    IncisalRest,
    OnlayRest,
}

/// RPD framework design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpdFramework {
    pub id: Uuid,
    pub patient_name: String,
    pub arch: Arch,
    pub kennedy_class: KennedyClass,
    pub modifications: u8,
    pub major_connector: MajorConnectorType,
    pub clasps: Vec<Clasp>,
    pub rests: Vec<RestSeat>,
    pub missing_teeth: Vec<String>,
    pub mesh_retention_pattern: MeshRetentionType,
    pub framework_thickness_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arch { Maxillary, Mandibular }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeshRetentionType {
    OpenMesh,
    LadderMesh,
    BeadedMesh,
    Lattice,
    None,
}

impl RpdFramework {
    pub fn new(patient: impl Into<String>, arch: Arch, class: KennedyClass) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_name: patient.into(),
            arch,
            kennedy_class: class,
            modifications: 0,
            major_connector: match arch {
                Arch::Maxillary => MajorConnectorType::PalatalStrap,
                Arch::Mandibular => MajorConnectorType::LingualBar,
            },
            clasps: Vec::new(),
            rests: Vec::new(),
            missing_teeth: Vec::new(),
            mesh_retention_pattern: MeshRetentionType::OpenMesh,
            framework_thickness_mm: 0.4,
        }
    }

    pub fn add_clasp(&mut self, clasp: Clasp) { self.clasps.push(clasp); }
    pub fn add_rest(&mut self, rest: RestSeat) { self.rests.push(rest); }
}

/// Survey analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResult {
    pub tooth_id: String,
    pub survey_line_height_mm: f64,
    pub undercut_depth_mm: f64,
    pub path_of_insertion: [f64; 3],
    pub suitable_for_clasp: bool,
}

/// Perform survey analysis on teeth geometry
pub fn survey_tooth(tooth_id: impl Into<String>, undercut: f64, path: [f64; 3]) -> SurveyResult {
    let suitable = undercut >= 0.25 && undercut <= 0.75;
    SurveyResult {
        tooth_id: tooth_id.into(),
        survey_line_height_mm: undercut * 2.0,
        undercut_depth_mm: undercut,
        path_of_insertion: path,
        suitable_for_clasp: suitable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpd_creation() {
        let rpd = RpdFramework::new("Test Patient", Arch::Mandibular, KennedyClass::ClassII);
        assert_eq!(rpd.kennedy_class, KennedyClass::ClassII);
        assert_eq!(rpd.major_connector, MajorConnectorType::LingualBar);
    }

    #[test]
    fn test_survey_suitable() {
        let s = survey_tooth("15", 0.5, [0.0, 1.0, 0.0]);
        assert!(s.suitable_for_clasp);
        assert!((s.undercut_depth_mm - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_survey_unsuitable_deep() {
        let s = survey_tooth("46", 1.0, [0.0, 1.0, 0.0]);
        assert!(!s.suitable_for_clasp);
    }

    #[test]
    fn test_rpd_add_clasp_rest() {
        let mut rpd = RpdFramework::new("P", Arch::Maxillary, KennedyClass::ClassIII);
        rpd.add_clasp(Clasp {
            id: Uuid::new_v4(), tooth_id: "15".into(), clasp_type: ClaspType::CircumferentialAkers,
            survey_line_depth_mm: 0.5, tip_position: [0.0; 3], retention_arm_length_mm: 15.0,
            reciprocal_arm: true,
        });
        rpd.add_rest(RestSeat {
            id: Uuid::new_v4(), tooth_id: "15".into(), rest_type: RestType::OcclusalRest,
            position_mm: [0.0; 3], width_mm: 2.5, depth_mm: 1.5,
        });
        assert_eq!(rpd.clasps.len(), 1);
        assert_eq!(rpd.rests.len(), 1);
    }
}

// ── S259-S262 Iteration: Extended removable prosthetics ──

/// Complete denture design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteDenture {
    pub patient_name: String,
    pub arch: Arch,
    pub base_material: String,
    pub teeth_material: String,
    pub vertical_dimension_mm: f64,
    pub occlusal_scheme: OcclusalScheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OcclusalScheme {
    Balanced,
    LingualizedOcclusion,
    MonoplaneOcclusion,
}

impl CompleteDenture {
    pub fn new(patient: impl Into<String>, arch: Arch) -> Self {
        Self {
            patient_name: patient.into(),
            arch,
            base_material: "PMMA".into(),
            teeth_material: "PMMA".into(),
            vertical_dimension_mm: 0.0,
            occlusal_scheme: OcclusalScheme::Balanced,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.vertical_dimension_mm <= 0.0 { issues.push("VDO not set".into()); }
        issues
    }
}

/// Flexible partial denture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexiblePartial {
    pub patient_name: String,
    pub arch: Arch,
    pub material: FlexMaterial,
    pub missing_teeth: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlexMaterial { Valplast, TCS, FlexStar }

impl FlexiblePartial {
    pub fn new(patient: impl Into<String>, arch: Arch, material: FlexMaterial) -> Self {
        Self { patient_name: patient.into(), arch, material, missing_teeth: Vec::new() }
    }

    pub fn add_missing(&mut self, tooth: impl Into<String>) {
        self.missing_teeth.push(tooth.into());
    }

    pub fn tooth_count(&self) -> usize { self.missing_teeth.len() }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_complete_denture() {
        let mut d = CompleteDenture::new("Patient", Arch::Maxillary);
        assert!(!d.validate().is_empty()); // VDO not set
        d.vertical_dimension_mm = 45.0;
        assert!(d.validate().is_empty());
    }

    #[test]
    fn test_flexible_partial() {
        let mut fp = FlexiblePartial::new("P", Arch::Mandibular, FlexMaterial::Valplast);
        fp.add_missing("35");
        fp.add_missing("36");
        assert_eq!(fp.tooth_count(), 2);
    }

    #[test]
    fn test_occlusal_scheme() {
        let d = CompleteDenture {
            occlusal_scheme: OcclusalScheme::LingualizedOcclusion,
            ..CompleteDenture::new("P", Arch::Mandibular)
        };
        assert_eq!(d.occlusal_scheme, OcclusalScheme::LingualizedOcclusion);
    }
}

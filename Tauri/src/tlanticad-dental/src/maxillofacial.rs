//! S281-S285: Maxillofacial Module
//!
//! Jaw segmentation, osteotomy planning, plates/screws placement,
//! soft tissue simulation, and custom implant design.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Jaw segment identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JawSegment {
    MaxillaLeft,
    MaxillaRight,
    MaxillaAnterior,
    MandibleLeft,
    MandibleRight,
    MandibleAnterior,
    MandibleRamus,
    Chin,
}

/// Osteotomy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OsteotomyType {
    LeFortI,
    LeFortII,
    LeFortIII,
    BSSO,           // Bilateral sagittal split osteotomy
    GenioplastyH,   // Horizontal
    GenioplastyV,   // Vertical
    SegmentalMaxilla,
    SegmentalMandible,
    Distraction,
}

/// An osteotomy cut definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsteotomyCut {
    pub id: Uuid,
    pub osteotomy_type: OsteotomyType,
    pub cut_plane_normal: [f64; 3],
    pub cut_plane_origin_mm: [f64; 3],
    pub affected_segment: JawSegment,
}

/// Planned segment movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMovement {
    pub segment: JawSegment,
    pub translation_mm: [f64; 3],
    pub rotation_deg: [f64; 3],
    pub description: String,
}

/// Fixation plate spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixationPlate {
    pub id: Uuid,
    pub plate_type: PlateType,
    pub position_mm: [f64; 3],
    pub hole_count: u32,
    pub thickness_mm: f64,
    pub material: PlateMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateType {
    Miniplate,
    MicroplateLocking,
    ReconstructionPlate,
    MeshPlate,
    PatientSpecific,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateMaterial {
    Titanium,
    TitaniumAlloy,
    Resorbable,
    CobaltChrome,
    PEEK,
}

/// Fixation screw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixationScrew {
    pub id: Uuid,
    pub plate_id: Uuid,
    pub position_mm: [f64; 3],
    pub axis: [f64; 3],
    pub diameter_mm: f64,
    pub length_mm: f64,
    pub screw_type: ScrewType,
    pub bicortical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScrewType {
    SelfTapping,
    SelfDrilling,
    Locking,
    Emergency,
}

/// Maxillofacial surgical plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxillofacialPlan {
    pub id: Uuid,
    pub patient_name: String,
    pub cuts: Vec<OsteotomyCut>,
    pub movements: Vec<SegmentMovement>,
    pub plates: Vec<FixationPlate>,
    pub screws: Vec<FixationScrew>,
    pub nerve_safety_margin_mm: f64,
}

impl MaxillofacialPlan {
    pub fn new(patient: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_name: patient.into(),
            cuts: Vec::new(),
            movements: Vec::new(),
            plates: Vec::new(),
            screws: Vec::new(),
            nerve_safety_margin_mm: 2.0,
        }
    }

    pub fn total_hardware(&self) -> usize {
        self.plates.len() + self.screws.len()
    }
}

/// Validate plate/screw placement against nerve canal
pub fn validate_nerve_clearance(
    screw_pos: [f64; 3],
    nerve_centerline: &[[f64; 3]],
    safety_margin_mm: f64,
) -> bool {
    for npt in nerve_centerline {
        let dx = screw_pos[0] - npt[0];
        let dy = screw_pos[1] - npt[1];
        let dz = screw_pos[2] - npt[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < safety_margin_mm {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maxfac_plan_creation() {
        let plan = MaxillofacialPlan::new("Patient");
        assert_eq!(plan.total_hardware(), 0);
        assert_eq!(plan.nerve_safety_margin_mm, 2.0);
    }

    #[test]
    fn test_nerve_clearance_safe() {
        let nerve = vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        assert!(validate_nerve_clearance([0.0, 5.0, 0.0], &nerve, 2.0));
    }

    #[test]
    fn test_nerve_clearance_violation() {
        let nerve = vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        assert!(!validate_nerve_clearance([0.0, 1.0, 0.0], &nerve, 2.0));
    }

    #[test]
    fn test_segment_movement() {
        let mv = SegmentMovement {
            segment: JawSegment::MaxillaLeft,
            translation_mm: [3.0, -2.0, 0.0],
            rotation_deg: [0.0, 0.0, 5.0],
            description: "Maxilla impaction + advance".into(),
        };
        assert_eq!(mv.segment, JawSegment::MaxillaLeft);
    }

    #[test]
    fn test_hardware_count() {
        let mut plan = MaxillofacialPlan::new("P");
        plan.plates.push(FixationPlate {
            id: Uuid::new_v4(), plate_type: PlateType::Miniplate,
            position_mm: [0.0; 3], hole_count: 4, thickness_mm: 1.0,
            material: PlateMaterial::Titanium,
        });
        plan.screws.push(FixationScrew {
            id: Uuid::new_v4(), plate_id: plan.plates[0].id,
            position_mm: [1.0, 0.0, 0.0], axis: [0.0, 1.0, 0.0],
            diameter_mm: 2.0, length_mm: 6.0, screw_type: ScrewType::SelfTapping,
            bicortical: false,
        });
        assert_eq!(plan.total_hardware(), 2);
    }
}

// ── S267-S270 Iteration: Extended maxillofacial features ──

/// Patient-specific implant (PSI) design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientSpecificImplant {
    pub id: Uuid,
    pub region: String,
    pub material: PlateMaterial,
    pub volume_mm3: f64,
    pub fixation_points: Vec<[f64; 3]>,
    pub thickness_range_mm: (f64, f64),
}

impl PatientSpecificImplant {
    pub fn new(region: impl Into<String>, material: PlateMaterial) -> Self {
        Self {
            id: Uuid::new_v4(),
            region: region.into(),
            material,
            volume_mm3: 0.0,
            fixation_points: Vec::new(),
            thickness_range_mm: (0.5, 3.0),
        }
    }

    pub fn add_fixation_point(&mut self, point: [f64; 3]) {
        self.fixation_points.push(point);
    }

    pub fn fixation_count(&self) -> usize { self.fixation_points.len() }
}

/// Virtual surgical planning report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VspReport {
    pub patient_name: String,
    pub procedure: String,
    pub segments: Vec<SegmentMovement>,
    pub total_plates: usize,
    pub total_screws: usize,
    pub total_cuts: usize,
}

pub fn generate_vsp_report(plan: &MaxillofacialPlan) -> VspReport {
    VspReport {
        patient_name: plan.patient_name.clone(),
        procedure: "Orthognathic Surgery".into(),
        segments: plan.movements.clone(),
        total_plates: plan.plates.len(),
        total_screws: plan.screws.len(),
        total_cuts: plan.cuts.len(),
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_psi_design() {
        let mut psi = PatientSpecificImplant::new("Left mandible", PlateMaterial::Titanium);
        psi.add_fixation_point([10.0, 5.0, 0.0]);
        psi.add_fixation_point([20.0, 5.0, 0.0]);
        assert_eq!(psi.fixation_count(), 2);
    }

    #[test]
    fn test_vsp_report() {
        let plan = MaxillofacialPlan::new("Patient Y");
        let report = generate_vsp_report(&plan);
        assert_eq!(report.total_plates, 0);
        assert_eq!(report.total_screws, 0);
    }

    #[test]
    fn test_psi_material() {
        let psi = PatientSpecificImplant::new("Zygomatic", PlateMaterial::PEEK);
        assert_eq!(psi.material, PlateMaterial::PEEK);
    }
}

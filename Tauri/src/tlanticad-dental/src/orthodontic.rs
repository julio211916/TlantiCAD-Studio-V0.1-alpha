//! S276-S280: Orthodontic Module
//!
//! Aligner design, tooth movement planning, staging,
//! attachment design, and treatment simulation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Orthodontic tooth movement type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementType {
    Translation,
    Rotation,
    Torque,        // root movement
    Tipping,
    Intrusion,
    Extrusion,
    Derotation,
}

/// Single tooth movement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothMovement {
    pub tooth_id: String,
    pub movement_type: MovementType,
    pub amount_mm: f64,
    pub amount_deg: f64,
    pub direction: [f64; 3],
}

/// Treatment stage (one aligner step)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentStage {
    pub stage_number: u32,
    pub movements: Vec<ToothMovement>,
    pub max_movement_mm: f64,
    pub max_rotation_deg: f64,
    pub duration_weeks: u32,
}

/// Attachment type for aligner therapy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentType {
    Conventional,
    Optimized,
    PrecisionCut,
    PowerRidge,
    BiteRamp,
    ElasticHook,
}

/// Attachment shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentShape {
    Rectangular,
    Ellipsoid,
    Trapezoidal,
    Beveled,
}

/// An attachment placed on a tooth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub tooth_id: String,
    pub attachment_type: AttachmentType,
    pub shape: AttachmentShape,
    pub position_mm: [f64; 3],
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
}

impl Attachment {
    pub fn standard(tooth_id: impl Into<String>, pos: [f64; 3]) -> Self {
        Self {
            id: Uuid::new_v4(),
            tooth_id: tooth_id.into(),
            attachment_type: AttachmentType::Conventional,
            shape: AttachmentShape::Rectangular,
            position_mm: pos,
            width_mm: 3.0,
            height_mm: 2.0,
            depth_mm: 1.0,
        }
    }
}

/// Aligner treatment plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignerPlan {
    pub id: Uuid,
    pub patient_name: String,
    pub stages: Vec<TreatmentStage>,
    pub attachments: Vec<Attachment>,
    pub total_stages: u32,
    pub estimated_months: f64,
    pub max_movement_per_stage_mm: f64,
    pub max_rotation_per_stage_deg: f64,
}

impl AlignerPlan {
    pub fn new(patient: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_name: patient.into(),
            stages: Vec::new(),
            attachments: Vec::new(),
            total_stages: 0,
            estimated_months: 0.0,
            max_movement_per_stage_mm: 0.25,
            max_rotation_per_stage_deg: 2.0,
        }
    }

    /// Auto-stage movements respecting biomechanical limits
    pub fn auto_stage(&mut self, movements: &[ToothMovement]) {
        // Split movements into stages by max per-stage limits
        let mut remaining: Vec<ToothMovement> = movements.to_vec();
        let mut stage_num = 0u32;

        while !remaining.is_empty() {
            stage_num += 1;
            let mut stage_movements = Vec::new();
            let mut next_remaining = Vec::new();

            for m in &remaining {
                let step_mm = m.amount_mm.min(self.max_movement_per_stage_mm);
                let step_deg = m.amount_deg.min(self.max_rotation_per_stage_deg);

                stage_movements.push(ToothMovement {
                    tooth_id: m.tooth_id.clone(),
                    movement_type: m.movement_type,
                    amount_mm: step_mm,
                    amount_deg: step_deg,
                    direction: m.direction,
                });

                let rem_mm = m.amount_mm - step_mm;
                let rem_deg = m.amount_deg - step_deg;
                if rem_mm > 0.01 || rem_deg > 0.01 {
                    next_remaining.push(ToothMovement {
                        tooth_id: m.tooth_id.clone(),
                        movement_type: m.movement_type,
                        amount_mm: rem_mm.max(0.0),
                        amount_deg: rem_deg.max(0.0),
                        direction: m.direction,
                    });
                }
            }

            let max_mm = stage_movements.iter().map(|m| m.amount_mm).fold(0.0f64, f64::max);
            let max_deg = stage_movements.iter().map(|m| m.amount_deg).fold(0.0f64, f64::max);

            self.stages.push(TreatmentStage {
                stage_number: stage_num,
                movements: stage_movements,
                max_movement_mm: max_mm,
                max_rotation_deg: max_deg,
                duration_weeks: 2,
            });

            remaining = next_remaining;
        }

        self.total_stages = stage_num;
        self.estimated_months = stage_num as f64 * 2.0 / 4.33;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligner_plan_auto_stage() {
        let mut plan = AlignerPlan::new("Test");
        plan.auto_stage(&[
            ToothMovement {
                tooth_id: "11".into(),
                movement_type: MovementType::Translation,
                amount_mm: 0.6,
                amount_deg: 0.0,
                direction: [1.0, 0.0, 0.0],
            },
        ]);
        // 0.6mm / 0.25mm per stage = 3 stages
        assert_eq!(plan.total_stages, 3);
        assert!(plan.estimated_months > 0.0);
    }

    #[test]
    fn test_attachment_standard() {
        let att = Attachment::standard("21", [0.0, 3.0, 1.0]);
        assert_eq!(att.width_mm, 3.0);
        assert_eq!(att.attachment_type, AttachmentType::Conventional);
    }

    #[test]
    fn test_single_stage_small_movement() {
        let mut plan = AlignerPlan::new("Small");
        plan.auto_stage(&[
            ToothMovement {
                tooth_id: "41".into(), movement_type: MovementType::Intrusion,
                amount_mm: 0.2, amount_deg: 0.0, direction: [0.0, -1.0, 0.0],
            },
        ]);
        assert_eq!(plan.total_stages, 1);
    }

    #[test]
    fn test_rotation_staging() {
        let mut plan = AlignerPlan::new("Rot");
        plan.auto_stage(&[
            ToothMovement {
                tooth_id: "33".into(), movement_type: MovementType::Derotation,
                amount_mm: 0.0, amount_deg: 5.0, direction: [0.0, 1.0, 0.0],
            },
        ]);
        // 5 deg / 2 deg per stage = 3 stages
        assert_eq!(plan.total_stages, 3);
    }
}

// ── S263-S266 Iteration: Extended orthodontic features ──

/// Cephalometric analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephAnalysis {
    pub sna_angle: f64,
    pub snb_angle: f64,
    pub anb_angle: f64,
    pub fma_angle: f64,
    pub impa_angle: f64,
    pub classification: SkelClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkelClass { ClassI, ClassII, ClassIII }

pub fn ceph_classify(anb: f64) -> SkelClass {
    if anb > 4.0 { SkelClass::ClassII }
    else if anb < 0.0 { SkelClass::ClassIII }
    else { SkelClass::ClassI }
}

/// Space analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceAnalysis {
    pub arch: String,
    pub available_space_mm: f64,
    pub required_space_mm: f64,
    pub discrepancy_mm: f64,
}

pub fn space_analysis(arch: impl Into<String>, available: f64, required: f64) -> SpaceAnalysis {
    SpaceAnalysis {
        arch: arch.into(),
        available_space_mm: available,
        required_space_mm: required,
        discrepancy_mm: available - required,
    }
}

/// Bolton analysis ratios
pub fn bolton_anterior_ratio(lower_6: f64, upper_6: f64) -> f64 {
    if upper_6 == 0.0 { 0.0 } else { lower_6 / upper_6 * 100.0 }
}

pub fn bolton_overall_ratio(lower_12: f64, upper_12: f64) -> f64 {
    if upper_12 == 0.0 { 0.0 } else { lower_12 / upper_12 * 100.0 }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_ceph_classification() {
        assert_eq!(ceph_classify(2.0), SkelClass::ClassI);
        assert_eq!(ceph_classify(6.0), SkelClass::ClassII);
        assert_eq!(ceph_classify(-2.0), SkelClass::ClassIII);
    }

    #[test]
    fn test_space_analysis() {
        let sa = space_analysis("Mandibular", 65.0, 70.0);
        assert!(sa.discrepancy_mm < 0.0); // crowding
    }

    #[test]
    fn test_bolton_anterior() {
        let ratio = bolton_anterior_ratio(38.0, 49.0);
        assert!(ratio > 70.0 && ratio < 85.0);
    }

    #[test]
    fn test_bolton_overall() {
        let ratio = bolton_overall_ratio(88.0, 97.0);
        assert!(ratio > 85.0 && ratio < 100.0);
    }
}

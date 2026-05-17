//! S261-S265: Advanced Occlusion
//!
//! Functional occlusion schemes, TMJ simulation, excursive contacts,
//! and clearance/interocclusal space analysis.

use serde::{Deserialize, Serialize};

/// Occlusal scheme type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OcclusionScheme {
    MutuallyProtected,
    GroupFunction,
    CanineGuidance,
    BalancedBilateral,
    LingualisedOcclusion,
}

/// TMJ health classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TmjStatus {
    Normal,
    Clicking,
    Crepitus,
    LimitedOpening,
    Deviation,
    Locking,
}

/// TMJ simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmjSimulation {
    pub left_status: TmjStatus,
    pub right_status: TmjStatus,
    pub max_opening_mm: f64,
    pub max_lateral_mm: f64,
    pub max_protrusion_mm: f64,
    pub deviation_on_opening_deg: f64,
}

impl Default for TmjSimulation {
    fn default() -> Self {
        Self {
            left_status: TmjStatus::Normal,
            right_status: TmjStatus::Normal,
            max_opening_mm: 45.0,
            max_lateral_mm: 10.0,
            max_protrusion_mm: 10.0,
            deviation_on_opening_deg: 0.0,
        }
    }
}

/// Excursive contact during lateral or protrusive movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcursiveContact {
    pub upper_tooth: String,
    pub lower_tooth: String,
    pub contact_type: ContactType,
    pub position_mm: [f64; 3],
    pub excursion_mm: f64,
    pub is_working_side: bool,
}

/// Type of occlusal contact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContactType {
    Centric,
    Working,
    NonWorking,
    Protrusive,
    Interference,
}

/// Clearance analysis for interocclusal space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearanceAnalysis {
    pub tooth_id: String,
    pub min_clearance_mm: f64,
    pub max_clearance_mm: f64,
    pub avg_clearance_mm: f64,
    pub sufficient: bool,
    pub required_clearance_mm: f64,
}

impl ClearanceAnalysis {
    /// Evaluate if clearance is sufficient for restoration material
    pub fn evaluate(tooth_id: impl Into<String>, clearances: &[f64], required: f64) -> Self {
        let min = clearances.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = clearances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg = if clearances.is_empty() { 0.0 }
                  else { clearances.iter().sum::<f64>() / clearances.len() as f64 };
        Self {
            tooth_id: tooth_id.into(),
            min_clearance_mm: min,
            max_clearance_mm: max,
            avg_clearance_mm: avg,
            sufficient: min >= required,
            required_clearance_mm: required,
        }
    }
}

/// Functional occlusion analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalOcclusionResult {
    pub scheme: OcclusionScheme,
    pub centric_contacts: Vec<ExcursiveContact>,
    pub working_contacts: Vec<ExcursiveContact>,
    pub non_working_contacts: Vec<ExcursiveContact>,
    pub protrusive_contacts: Vec<ExcursiveContact>,
    pub interferences: Vec<ExcursiveContact>,
    pub tmj: TmjSimulation,
    pub clearance: Vec<ClearanceAnalysis>,
    pub score: f64, // 0..100
}

impl FunctionalOcclusionResult {
    /// Overall quality assessment
    pub fn quality_label(&self) -> &'static str {
        if self.score >= 90.0 { "Excellent" }
        else if self.score >= 70.0 { "Good" }
        else if self.score >= 50.0 { "Acceptable" }
        else { "Needs Revision" }
    }

    pub fn has_interferences(&self) -> bool {
        !self.interferences.is_empty()
    }
}

/// Compute functional occlusion score from contacts analysis
pub fn compute_occlusion_score(
    scheme: OcclusionScheme,
    centric: &[ExcursiveContact],
    interferences: &[ExcursiveContact],
    clearances: &[ClearanceAnalysis],
) -> f64 {
    let mut score: f64 = 100.0;
    // Interferences are major deductions
    score -= interferences.len() as f64 * 10.0;
    // Insufficient clearance deductions
    let insufficient: usize = clearances.iter().filter(|c| !c.sufficient).count();
    score -= insufficient as f64 * 8.0;
    // Few centric contacts means poor stability
    if centric.len() < 4 {
        score -= (4 - centric.len()) as f64 * 5.0;
    }
    // Scheme-specific bonuses
    match scheme {
        OcclusionScheme::MutuallyProtected | OcclusionScheme::CanineGuidance => score += 5.0,
        _ => {}
    }
    score.clamp(0.0, 100.0)
}

/// Analysis of lateral excursion guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateralGuidance {
    pub side: LateralSide,
    pub guide_teeth: Vec<String>,
    pub disclusion_angle_deg: f64,
    pub immediate_disclusion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LateralSide { Left, Right }

impl LateralGuidance {
    pub fn evaluate(side: LateralSide, contacts: &[ExcursiveContact]) -> Self {
        let working: Vec<_> = contacts.iter()
            .filter(|c| c.is_working_side && c.contact_type == ContactType::Working)
            .collect();
        let guide_teeth: Vec<String> = working.iter().map(|c| c.upper_tooth.clone()).collect();
        let immediate = working.iter().all(|c| c.excursion_mm < 1.0);
        let angle = if working.is_empty() { 0.0 } else { 15.0 }; // simplified
        Self { side, guide_teeth, disclusion_angle_deg: angle, immediate_disclusion: immediate }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clearance_sufficient() {
        let ca = ClearanceAnalysis::evaluate("16", &[1.5, 2.0, 1.8, 2.2], 1.5);
        assert!(ca.sufficient);
        assert!((ca.min_clearance_mm - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_clearance_insufficient() {
        let ca = ClearanceAnalysis::evaluate("26", &[0.8, 1.0, 0.9], 1.5);
        assert!(!ca.sufficient);
    }

    #[test]
    fn test_tmj_defaults() {
        let tmj = TmjSimulation::default();
        assert_eq!(tmj.max_opening_mm, 45.0);
        assert_eq!(tmj.left_status, TmjStatus::Normal);
    }

    #[test]
    fn test_functional_result_scoring() {
        let result = FunctionalOcclusionResult {
            scheme: OcclusionScheme::MutuallyProtected,
            centric_contacts: vec![],
            working_contacts: vec![],
            non_working_contacts: vec![],
            protrusive_contacts: vec![],
            interferences: vec![],
            tmj: TmjSimulation::default(),
            clearance: vec![],
            score: 85.0,
        };
        assert_eq!(result.quality_label(), "Good");
        assert!(!result.has_interferences());
    }

    #[test]
    fn test_excursive_contact() {
        let contact = ExcursiveContact {
            upper_tooth: "13".into(),
            lower_tooth: "43".into(),
            contact_type: ContactType::Working,
            position_mm: [5.0, 2.0, 0.0],
            excursion_mm: 3.0,
            is_working_side: true,
        };
        assert_eq!(contact.contact_type, ContactType::Working);
        assert!(contact.is_working_side);
    }

    #[test]
    fn test_compute_occlusion_score_good() {
        let centric: Vec<ExcursiveContact> = (0..6).map(|i| ExcursiveContact {
            upper_tooth: format!("{}", 11 + i),
            lower_tooth: format!("{}", 41 + i),
            contact_type: ContactType::Centric,
            position_mm: [0.0; 3],
            excursion_mm: 0.0,
            is_working_side: true,
        }).collect();
        let score = compute_occlusion_score(OcclusionScheme::MutuallyProtected, &centric, &[], &[]);
        assert!(score > 90.0);
    }

    #[test]
    fn test_compute_occlusion_score_with_interferences() {
        let interference = ExcursiveContact {
            upper_tooth: "16".into(), lower_tooth: "46".into(),
            contact_type: ContactType::Interference,
            position_mm: [0.0; 3], excursion_mm: 2.0, is_working_side: false,
        };
        let score = compute_occlusion_score(OcclusionScheme::GroupFunction, &[], &[interference], &[]);
        assert!(score < 95.0);
    }

    #[test]
    fn test_lateral_guidance() {
        let contacts = vec![ExcursiveContact {
            upper_tooth: "13".into(), lower_tooth: "43".into(),
            contact_type: ContactType::Working,
            position_mm: [0.0; 3], excursion_mm: 0.5, is_working_side: true,
        }];
        let guidance = LateralGuidance::evaluate(LateralSide::Right, &contacts);
        assert_eq!(guidance.guide_teeth, vec!["13".to_string()]);
        assert!(guidance.immediate_disclusion);
    }
}

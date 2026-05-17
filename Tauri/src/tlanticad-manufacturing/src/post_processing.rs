//! S323-S328: Post-Processing Pipeline
//!
//! Sintering, polishing, staining, glazing, and finishing workflows.

use serde::{Deserialize, Serialize};

/// Post-processing step type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostProcessStep {
    SprueRemoval,
    Sandblasting,
    Sintering,
    Polishing,
    Glazing,
    Staining,
    CrystallizationFiring,
    UvCuring,
    WashingIPA,
    SupportRemoval,
    FitAdjustment,
    FinalInspection,
}

impl PostProcessStep {
    pub fn estimated_duration_min(&self) -> f64 {
        match self {
            Self::SprueRemoval => 5.0,
            Self::Sandblasting => 3.0,
            Self::Sintering => 480.0,  // 8 hours typical zirconia
            Self::Polishing => 15.0,
            Self::Glazing => 30.0,
            Self::Staining => 20.0,
            Self::CrystallizationFiring => 25.0,
            Self::UvCuring => 15.0,
            Self::WashingIPA => 10.0,
            Self::SupportRemoval => 10.0,
            Self::FitAdjustment => 15.0,
            Self::FinalInspection => 10.0,
        }
    }

    pub fn requires_equipment(&self) -> &'static str {
        match self {
            Self::Sintering => "Sintering furnace",
            Self::Glazing | Self::CrystallizationFiring => "Ceramic furnace",
            Self::Sandblasting => "Sandblaster",
            Self::UvCuring => "UV curing chamber",
            Self::WashingIPA => "IPA wash station",
            Self::Polishing => "Polishing motor",
            _ => "Hand tools",
        }
    }
}

/// Temperature profile for sintering/firing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiringProfile {
    pub name: String,
    pub stages: Vec<FiringStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiringStage {
    pub target_temp_c: f64,
    pub ramp_rate_c_min: f64,
    pub hold_time_min: f64,
    pub vacuum: bool,
}

impl FiringProfile {
    pub fn zirconia_sintering() -> Self {
        Self {
            name: "Zirconia Full Sintering".into(),
            stages: vec![
                FiringStage { target_temp_c: 900.0, ramp_rate_c_min: 10.0, hold_time_min: 0.0, vacuum: false },
                FiringStage { target_temp_c: 1500.0, ramp_rate_c_min: 5.0, hold_time_min: 120.0, vacuum: false },
                FiringStage { target_temp_c: 25.0, ramp_rate_c_min: -3.0, hold_time_min: 0.0, vacuum: false },
            ],
        }
    }

    pub fn emax_crystallization() -> Self {
        Self {
            name: "e.max Crystallization".into(),
            stages: vec![
                FiringStage { target_temp_c: 403.0, ramp_rate_c_min: 60.0, hold_time_min: 0.0, vacuum: true },
                FiringStage { target_temp_c: 820.0, ramp_rate_c_min: 30.0, hold_time_min: 10.0, vacuum: true },
                FiringStage { target_temp_c: 840.0, ramp_rate_c_min: 10.0, hold_time_min: 7.0, vacuum: false },
            ],
        }
    }

    pub fn total_time_min(&self) -> f64 {
        let mut total = 0.0;
        let mut current_temp = 25.0;
        for stage in &self.stages {
            let delta = (stage.target_temp_c - current_temp).abs();
            let ramp = if stage.ramp_rate_c_min.abs() > 0.0 {
                delta / stage.ramp_rate_c_min.abs()
            } else { 0.0 };
            total += ramp + stage.hold_time_min;
            current_temp = stage.target_temp_c;
        }
        total
    }

    pub fn peak_temperature(&self) -> f64 {
        self.stages.iter().map(|s| s.target_temp_c).fold(0.0_f64, f64::max)
    }
}

/// Post-processing workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessWorkflow {
    pub material: String,
    pub steps: Vec<PostProcessStep>,
    pub firing_profiles: Vec<FiringProfile>,
}

impl PostProcessWorkflow {
    pub fn for_zirconia() -> Self {
        Self {
            material: "Zirconia".into(),
            steps: vec![
                PostProcessStep::SprueRemoval,
                PostProcessStep::Sintering,
                PostProcessStep::Sandblasting,
                PostProcessStep::Staining,
                PostProcessStep::Glazing,
                PostProcessStep::Polishing,
                PostProcessStep::FinalInspection,
            ],
            firing_profiles: vec![FiringProfile::zirconia_sintering()],
        }
    }

    pub fn for_emax() -> Self {
        Self {
            material: "e.max".into(),
            steps: vec![
                PostProcessStep::SprueRemoval,
                PostProcessStep::CrystallizationFiring,
                PostProcessStep::Polishing,
                PostProcessStep::Glazing,
                PostProcessStep::FinalInspection,
            ],
            firing_profiles: vec![FiringProfile::emax_crystallization()],
        }
    }

    pub fn for_3d_print() -> Self {
        Self {
            material: "Resin".into(),
            steps: vec![
                PostProcessStep::WashingIPA,
                PostProcessStep::SupportRemoval,
                PostProcessStep::UvCuring,
                PostProcessStep::Polishing,
                PostProcessStep::FinalInspection,
            ],
            firing_profiles: Vec::new(),
        }
    }

    pub fn total_time_min(&self) -> f64 {
        let steps_time: f64 = self.steps.iter().map(|s| s.estimated_duration_min()).sum();
        let firing_time: f64 = self.firing_profiles.iter().map(|f| f.total_time_min()).sum();
        steps_time + firing_time
    }

    pub fn equipment_needed(&self) -> Vec<&'static str> {
        let mut eq: Vec<&'static str> = self.steps.iter()
            .map(|s| s.requires_equipment())
            .collect();
        eq.sort();
        eq.dedup();
        eq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zirconia_workflow() {
        let wf = PostProcessWorkflow::for_zirconia();
        assert_eq!(wf.steps.len(), 7);
        assert!(wf.total_time_min() > 400.0); // at least sintering time
    }

    #[test]
    fn test_emax_workflow() {
        let wf = PostProcessWorkflow::for_emax();
        assert!(wf.firing_profiles[0].peak_temperature() > 800.0);
    }

    #[test]
    fn test_3d_print_workflow() {
        let wf = PostProcessWorkflow::for_3d_print();
        assert!(wf.steps.contains(&PostProcessStep::UvCuring));
        assert!(wf.firing_profiles.is_empty());
    }

    #[test]
    fn test_firing_profile_time() {
        let fp = FiringProfile::zirconia_sintering();
        assert!(fp.total_time_min() > 100.0);
        assert!((fp.peak_temperature() - 1500.0).abs() < 0.1);
    }

    #[test]
    fn test_step_durations() {
        assert!(PostProcessStep::Sintering.estimated_duration_min() > PostProcessStep::Polishing.estimated_duration_min());
    }

    #[test]
    fn test_equipment_list() {
        let wf = PostProcessWorkflow::for_zirconia();
        let eq = wf.equipment_needed();
        assert!(eq.contains(&"Sintering furnace"));
    }
}

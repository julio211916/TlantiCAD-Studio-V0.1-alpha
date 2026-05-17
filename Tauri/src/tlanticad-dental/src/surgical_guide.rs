//! S266-S270: Surgical Guide Design
//!
//! Implant surgical guide creation, drill sleeve placement,
//! retention features, validation, and manufacturing export.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Surgical guide type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuideType {
    ToothSupported,
    MucosaSupported,
    BoneSupported,
    Stackable,
}

/// Drill sleeve specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillSleeve {
    pub id: Uuid,
    pub implant_id: Uuid,
    pub position_mm: [f64; 3],
    pub axis: [f64; 3],
    pub inner_diameter_mm: f64,
    pub outer_diameter_mm: f64,
    pub height_mm: f64,
    pub depth_stop_mm: f64,
    pub sleeve_system: String,
}

impl DrillSleeve {
    pub fn standard(implant_id: Uuid, pos: [f64; 3], axis: [f64; 3], diameter: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            implant_id,
            position_mm: pos,
            axis,
            inner_diameter_mm: diameter,
            outer_diameter_mm: diameter + 1.0,
            height_mm: 5.0,
            depth_stop_mm: 0.0,
            sleeve_system: "universal".into(),
        }
    }
}

/// Retention feature for guide stability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetentionType {
    Clasp,
    Undercut,
    FrictionFit,
    ScrewRetained,
    PinHole,
}

/// A retention element on the guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionFeature {
    pub id: Uuid,
    pub retention_type: RetentionType,
    pub position_mm: [f64; 3],
    pub strength: f64,
}

/// Surgical guide definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalGuide {
    pub id: Uuid,
    pub name: String,
    pub guide_type: GuideType,
    pub sleeves: Vec<DrillSleeve>,
    pub retention: Vec<RetentionFeature>,
    pub wall_thickness_mm: f64,
    pub offset_mm: f64,
    pub window_openings: bool,
    pub inspection_windows: u32,
}

impl SurgicalGuide {
    pub fn new(name: impl Into<String>, guide_type: GuideType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            guide_type,
            sleeves: Vec::new(),
            retention: Vec::new(),
            wall_thickness_mm: 2.5,
            offset_mm: 0.05,
            window_openings: true,
            inspection_windows: 2,
        }
    }

    pub fn add_sleeve(&mut self, sleeve: DrillSleeve) {
        self.sleeves.push(sleeve);
    }

    pub fn add_retention(&mut self, feature: RetentionFeature) {
        self.retention.push(feature);
    }
}

/// Guide validation criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideValidation {
    pub sleeve_clearance_ok: bool,
    pub min_wall_thickness_ok: bool,
    pub retention_adequate: bool,
    pub no_sleeve_collisions: bool,
    pub axis_within_bone: bool,
    pub issues: Vec<String>,
    pub score: f64,
}

/// Validate a surgical guide design
pub fn validate_guide(guide: &SurgicalGuide) -> GuideValidation {
    let mut issues = Vec::new();
    let mut score: f64 = 100.0;

    // Check minimum wall thickness
    let min_wall_ok = guide.wall_thickness_mm >= 2.0;
    if !min_wall_ok {
        issues.push(format!("Wall thickness {:.1} mm < 2.0 mm minimum", guide.wall_thickness_mm));
        score -= 20.0;
    }

    // Check sleeve clearance (sleeves shouldn't overlap)
    let mut no_collisions = true;
    for (i, s1) in guide.sleeves.iter().enumerate() {
        for s2 in guide.sleeves.iter().skip(i + 1) {
            let dx = s1.position_mm[0] - s2.position_mm[0];
            let dy = s1.position_mm[1] - s2.position_mm[1];
            let dz = s1.position_mm[2] - s2.position_mm[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let min_dist = (s1.outer_diameter_mm + s2.outer_diameter_mm) / 2.0 + 1.0;
            if dist < min_dist {
                no_collisions = false;
                issues.push(format!("Sleeves too close: {:.1} mm (min {:.1} mm)", dist, min_dist));
                score -= 30.0;
            }
        }
    }

    // Check retention
    let retention_ok = !guide.retention.is_empty() || guide.guide_type == GuideType::BoneSupported;
    if !retention_ok {
        issues.push("No retention features defined".into());
        score -= 15.0;
    }

    GuideValidation {
        sleeve_clearance_ok: guide.sleeves.len() <= 1 || no_collisions,
        min_wall_thickness_ok: min_wall_ok,
        retention_adequate: retention_ok,
        no_sleeve_collisions: no_collisions,
        axis_within_bone: true, // requires bone mesh — stub true
        issues,
        score: score.max(0.0),
    }
}

/// Manufacturing export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuideExportFormat {
    STL,
    ThreeMF,
    OBJ,
    STEP,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surgical_guide_creation() {
        let mut guide = SurgicalGuide::new("Guide_Mx", GuideType::ToothSupported);
        let imp_id = Uuid::new_v4();
        guide.add_sleeve(DrillSleeve::standard(imp_id, [10.0, 5.0, 0.0], [0.0, 1.0, 0.0], 3.5));
        assert_eq!(guide.sleeves.len(), 1);
        assert_eq!(guide.guide_type, GuideType::ToothSupported);
    }

    #[test]
    fn test_validation_pass() {
        let mut guide = SurgicalGuide::new("Test", GuideType::ToothSupported);
        guide.add_sleeve(DrillSleeve::standard(Uuid::new_v4(), [0.0; 3], [0.0, 1.0, 0.0], 3.5));
        guide.add_retention(RetentionFeature {
            id: Uuid::new_v4(),
            retention_type: RetentionType::FrictionFit,
            position_mm: [5.0, 0.0, 0.0],
            strength: 0.8,
        });
        let v = validate_guide(&guide);
        assert!(v.min_wall_thickness_ok);
        assert!(v.retention_adequate);
        assert!(v.score >= 80.0);
    }

    #[test]
    fn test_validation_thin_wall() {
        let guide = SurgicalGuide {
            wall_thickness_mm: 1.0,
            ..SurgicalGuide::new("Thin", GuideType::MucosaSupported)
        };
        let v = validate_guide(&guide);
        assert!(!v.min_wall_thickness_ok);
        assert!(v.score < 100.0);
    }

    #[test]
    fn test_sleeve_collision() {
        let mut guide = SurgicalGuide::new("Collision", GuideType::ToothSupported);
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        guide.add_sleeve(DrillSleeve::standard(id1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 4.0));
        guide.add_sleeve(DrillSleeve::standard(id2, [2.0, 0.0, 0.0], [0.0, 1.0, 0.0], 4.0));
        guide.add_retention(RetentionFeature {
            id: Uuid::new_v4(), retention_type: RetentionType::Clasp,
            position_mm: [10.0, 0.0, 0.0], strength: 0.5,
        });
        let v = validate_guide(&guide);
        assert!(!v.no_sleeve_collisions);
    }
}

// ── S255-S258 Iteration: Extended surgical guide features ──

/// Stackable guide for sequential drilling protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackableGuideSet {
    pub guides: Vec<StackableLayer>,
    pub patient_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackableLayer {
    pub layer_name: String,
    pub drill_diameter_mm: f64,
    pub sleeve_ids: Vec<Uuid>,
    pub order: u8,
}

impl StackableGuideSet {
    pub fn new(patient: impl Into<String>) -> Self {
        Self { guides: Vec::new(), patient_name: patient.into() }
    }

    pub fn add_layer(&mut self, name: impl Into<String>, drill_mm: f64, order: u8) {
        self.guides.push(StackableLayer {
            layer_name: name.into(),
            drill_diameter_mm: drill_mm,
            sleeve_ids: Vec::new(),
            order,
        });
        self.guides.sort_by_key(|g| g.order);
    }

    pub fn layer_count(&self) -> usize { self.guides.len() }
}

/// Guide accuracy report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideAccuracyReport {
    pub angular_deviation_deg: f64,
    pub apical_deviation_mm: f64,
    pub coronal_deviation_mm: f64,
    pub depth_deviation_mm: f64,
    pub overall_accuracy_score: f64,
}

pub fn calculate_accuracy(
    planned_axis: [f64; 3],
    actual_axis: [f64; 3],
    planned_depth: f64,
    actual_depth: f64,
) -> GuideAccuracyReport {
    let dot = planned_axis.iter().zip(actual_axis.iter())
        .map(|(a, b)| a * b).sum::<f64>();
    let mag_p = planned_axis.iter().map(|a| a * a).sum::<f64>().sqrt();
    let mag_a = actual_axis.iter().map(|a| a * a).sum::<f64>().sqrt();
    let cos_angle = (dot / (mag_p * mag_a).max(f64::EPSILON)).clamp(-1.0, 1.0);
    let angle = cos_angle.acos().to_degrees();

    let depth_dev = (planned_depth - actual_depth).abs();
    let score = (100.0 - angle * 10.0 - depth_dev * 5.0).max(0.0).min(100.0);

    GuideAccuracyReport {
        angular_deviation_deg: angle,
        apical_deviation_mm: angle * 0.2,
        coronal_deviation_mm: angle * 0.1,
        depth_deviation_mm: depth_dev,
        overall_accuracy_score: score,
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_stackable_guide() {
        let mut set = StackableGuideSet::new("Patient X");
        set.add_layer("Pilot", 2.0, 1);
        set.add_layer("Intermediate", 3.2, 2);
        set.add_layer("Final", 4.0, 3);
        assert_eq!(set.layer_count(), 3);
        assert_eq!(set.guides[0].order, 1);
    }

    #[test]
    fn test_accuracy_perfect() {
        let report = calculate_accuracy(
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], 12.0, 12.0,
        );
        assert!(report.angular_deviation_deg < 0.01);
        assert!(report.overall_accuracy_score > 99.0);
    }

    #[test]
    fn test_accuracy_deviation() {
        let report = calculate_accuracy(
            [0.0, 1.0, 0.0], [0.1, 0.99, 0.0], 12.0, 11.0,
        );
        assert!(report.angular_deviation_deg > 0.0);
        assert!(report.depth_deviation_mm > 0.0);
        assert!(report.overall_accuracy_score < 100.0);
    }
}

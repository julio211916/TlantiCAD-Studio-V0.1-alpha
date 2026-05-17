//! S256-S260: Virtual Articulator
//!
//! Bennett angle, condylar inclination, jaw movement simulation,
//! facebow transfer data, and articulator-based occlusal analysis.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────
//  Articulator settings
// ────────────────────────────────────────────────────────────────────

/// Articulator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArticulatorType {
    /// Simple hinge axis (non-adjustable)
    Hinge,
    /// Semi-adjustable (most clinical cases)
    SemiAdjustable,
    /// Fully adjustable (complex restorations)
    FullyAdjustable,
    /// Virtual / digital articulator
    Virtual,
}

/// Configuration for a virtual articulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulatorConfig {
    pub articulator_type: ArticulatorType,
    /// Sagittal condylar inclination (degrees)
    pub condylar_inclination_deg: f64,
    /// Bennett angle (degrees) — immediate side shift
    pub bennett_angle_deg: f64,
    /// Intercondylar distance (mm)
    pub intercondylar_distance_mm: f64,
    /// Incisal guidance angle (degrees)
    pub incisal_guidance_deg: f64,
    /// Lateral incisal guidance (degrees)
    pub lateral_incisal_guidance_deg: f64,
    /// Immediate side shift (mm)
    pub immediate_side_shift_mm: f64,
}

impl Default for ArticulatorConfig {
    fn default() -> Self {
        Self {
            articulator_type: ArticulatorType::SemiAdjustable,
            condylar_inclination_deg: 30.0,
            bennett_angle_deg: 15.0,
            intercondylar_distance_mm: 110.0,
            incisal_guidance_deg: 10.0,
            lateral_incisal_guidance_deg: 10.0,
            immediate_side_shift_mm: 0.0,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Jaw movements
// ────────────────────────────────────────────────────────────────────

/// Type of mandibular movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JawMovement {
    Opening,
    Closing,
    Protrusion,
    LeftLateral,
    RightLateral,
    Retrusion,
}

/// A single step in a jaw movement path
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JawPose {
    /// Translation from centric relation [anterior, vertical, lateral] in mm
    pub translation_mm: [f64; 3],
    /// Rotation angles [pitch, yaw, roll] in degrees
    pub rotation_deg: [f64; 3],
    /// Time stamp in the movement sequence (seconds)
    pub time_s: f64,
}

/// Simulated jaw movement path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JawMovementPath {
    pub movement_type: JawMovement,
    pub poses: Vec<JawPose>,
    pub max_opening_mm: f64,
    pub max_protrusion_mm: f64,
}

impl JawMovementPath {
    /// Generate a simple opening movement
    pub fn generate_opening(config: &ArticulatorConfig, max_mm: f64, steps: usize) -> Self {
        let poses: Vec<JawPose> = (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                let opening = max_mm * t;
                let pitch = config.condylar_inclination_deg * (opening / max_mm) * 0.5;
                JawPose {
                    translation_mm: [0.0, -opening, 0.0],
                    rotation_deg: [-pitch, 0.0, 0.0],
                    time_s: t * 2.0,
                }
            })
            .collect();
        Self {
            movement_type: JawMovement::Opening,
            poses,
            max_opening_mm: max_mm,
            max_protrusion_mm: 0.0,
        }
    }

    /// Generate protrusion movement
    pub fn generate_protrusion(config: &ArticulatorConfig, max_mm: f64, steps: usize) -> Self {
        let poses: Vec<JawPose> = (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                let protr = max_mm * t;
                let descend = protr * config.condylar_inclination_deg.to_radians().tan();
                JawPose {
                    translation_mm: [protr, -descend, 0.0],
                    rotation_deg: [0.0, 0.0, 0.0],
                    time_s: t * 1.5,
                }
            })
            .collect();
        Self {
            movement_type: JawMovement::Protrusion,
            poses,
            max_opening_mm: 0.0,
            max_protrusion_mm: max_mm,
        }
    }

    /// Generate lateral excursion
    pub fn generate_lateral(config: &ArticulatorConfig, side: JawMovement, max_mm: f64, steps: usize) -> Self {
        let sign = if side == JawMovement::LeftLateral { -1.0 } else { 1.0 };
        let poses: Vec<JawPose> = (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                let lateral = max_mm * t;
                let iss = config.immediate_side_shift_mm.min(lateral);
                let bennett_shift = (lateral - iss) * config.bennett_angle_deg.to_radians().tan();
                JawPose {
                    translation_mm: [bennett_shift * 0.2, 0.0, sign * (lateral + iss)],
                    rotation_deg: [0.0, sign * lateral * 0.5, 0.0],
                    time_s: t * 1.5,
                }
            })
            .collect();
        Self {
            movement_type: side,
            poses,
            max_opening_mm: 0.0,
            max_protrusion_mm: 0.0,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Facebow Transfer
// ────────────────────────────────────────────────────────────────────

/// Facebow transfer data (positions relative to hinge axis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebowTransfer {
    pub left_condyle_mm: [f64; 3],
    pub right_condyle_mm: [f64; 3],
    pub orbital_point_mm: [f64; 3],
    pub bite_fork_offset_mm: [f64; 3],
    pub intercondylar_distance_mm: f64,
}

impl FacebowTransfer {
    /// Create from measured condylar positions
    pub fn from_measurements(
        left: [f64; 3],
        right: [f64; 3],
        orbital: [f64; 3],
        fork_offset: [f64; 3],
    ) -> Self {
        let dx = right[0] - left[0];
        let dy = right[1] - left[1];
        let dz = right[2] - left[2];
        let icd = (dx * dx + dy * dy + dz * dz).sqrt();
        Self {
            left_condyle_mm: left,
            right_condyle_mm: right,
            orbital_point_mm: orbital,
            bite_fork_offset_mm: fork_offset,
            intercondylar_distance_mm: icd,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Occlusal analysis (articulator-based)
// ────────────────────────────────────────────────────────────────────

/// Contact point found during articulator simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulatorContact {
    pub upper_tooth: String,
    pub lower_tooth: String,
    pub position_mm: [f64; 3],
    pub force_n: f64,
    pub movement: JawMovement,
    pub time_s: f64,
    pub is_interference: bool,
}

/// Full articulator analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulatorAnalysis {
    pub config: ArticulatorConfig,
    pub contacts_centric: Vec<ArticulatorContact>,
    pub contacts_protrusive: Vec<ArticulatorContact>,
    pub contacts_left_lateral: Vec<ArticulatorContact>,
    pub contacts_right_lateral: Vec<ArticulatorContact>,
    pub interferences_count: usize,
}

impl ArticulatorAnalysis {
    pub fn total_contacts(&self) -> usize {
        self.contacts_centric.len()
            + self.contacts_protrusive.len()
            + self.contacts_left_lateral.len()
            + self.contacts_right_lateral.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_articulator() {
        let cfg = ArticulatorConfig::default();
        assert_eq!(cfg.condylar_inclination_deg, 30.0);
        assert_eq!(cfg.bennett_angle_deg, 15.0);
        assert_eq!(cfg.intercondylar_distance_mm, 110.0);
    }

    #[test]
    fn test_opening_path() {
        let cfg = ArticulatorConfig::default();
        let path = JawMovementPath::generate_opening(&cfg, 40.0, 10);
        assert_eq!(path.poses.len(), 11);
        assert_eq!(path.movement_type, JawMovement::Opening);
        // First pose should be at zero
        assert_eq!(path.poses[0].translation_mm[1], 0.0);
        // Last pose should be at max
        assert!((path.poses[10].translation_mm[1] - (-40.0)).abs() < 0.01);
    }

    #[test]
    fn test_protrusion_path() {
        let cfg = ArticulatorConfig::default();
        let path = JawMovementPath::generate_protrusion(&cfg, 10.0, 5);
        assert_eq!(path.poses.len(), 6);
        assert_eq!(path.movement_type, JawMovement::Protrusion);
        // Should move anteriorly
        assert!(path.poses.last().unwrap().translation_mm[0] > 0.0);
    }

    #[test]
    fn test_lateral_path() {
        let cfg = ArticulatorConfig::default();
        let path = JawMovementPath::generate_lateral(
            &cfg, JawMovement::LeftLateral, 8.0, 4,
        );
        assert_eq!(path.movement_type, JawMovement::LeftLateral);
        // Left lateral should have negative z
        assert!(path.poses.last().unwrap().translation_mm[2] < 0.0);
    }

    #[test]
    fn test_facebow_transfer() {
        let fb = FacebowTransfer::from_measurements(
            [-55.0, 0.0, 0.0],
            [55.0, 0.0, 0.0],
            [0.0, 30.0, 40.0],
            [0.0, -10.0, 20.0],
        );
        assert!((fb.intercondylar_distance_mm - 110.0).abs() < 0.1);
    }

    #[test]
    fn test_articulator_analysis() {
        let analysis = ArticulatorAnalysis {
            config: ArticulatorConfig::default(),
            contacts_centric: vec![
                ArticulatorContact {
                    upper_tooth: "16".into(), lower_tooth: "46".into(),
                    position_mm: [0.0, 0.0, 0.0], force_n: 5.0,
                    movement: JawMovement::Closing, time_s: 0.0,
                    is_interference: false,
                },
            ],
            contacts_protrusive: vec![],
            contacts_left_lateral: vec![],
            contacts_right_lateral: vec![],
            interferences_count: 0,
        };
        assert_eq!(analysis.total_contacts(), 1);
    }
}

// ── S251-S254 Iteration: Extended virtual articulator features ──

/// Dynamic occlusion map — heatmap of contact intensities across jaw movements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicOcclusionMap {
    pub contact_points: Vec<OcclusionMapPoint>,
    pub total_force_n: f64,
    pub balance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionMapPoint {
    pub position_mm: [f64; 3],
    pub force_n: f64,
    pub timestamp_s: f64,
    pub tooth_upper: String,
    pub tooth_lower: String,
}

/// Compute dynamic occlusion map from articulator contacts
pub fn compute_dynamic_occlusion_map(contacts: &[ArticulatorContact]) -> DynamicOcclusionMap {
    let total: f64 = contacts.iter().map(|c| c.force_n).sum();
    let left_force: f64 = contacts.iter()
        .filter(|c| c.position_mm[0] < 0.0).map(|c| c.force_n).sum();
    let right_force: f64 = contacts.iter()
        .filter(|c| c.position_mm[0] >= 0.0).map(|c| c.force_n).sum();
    let balance = if total > 0.0 {
        1.0 - ((left_force - right_force).abs() / total)
    } else { 0.0 };

    DynamicOcclusionMap {
        contact_points: contacts.iter().map(|c| OcclusionMapPoint {
            position_mm: c.position_mm,
            force_n: c.force_n,
            timestamp_s: c.time_s,
            tooth_upper: c.upper_tooth.clone(),
            tooth_lower: c.lower_tooth.clone(),
        }).collect(),
        total_force_n: total,
        balance_score: balance,
    }
}

/// Gothic arch tracing analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GothicArchTracing {
    pub apex_position_mm: [f64; 2],
    pub left_angle_deg: f64,
    pub right_angle_deg: f64,
    pub symmetry_score: f64,
}

pub fn analyze_gothic_arch(left_angle: f64, right_angle: f64) -> GothicArchTracing {
    let symmetry = 1.0 - (left_angle - right_angle).abs() / (left_angle + right_angle).max(1.0);
    GothicArchTracing {
        apex_position_mm: [0.0, 0.0],
        left_angle_deg: left_angle,
        right_angle_deg: right_angle,
        symmetry_score: symmetry.max(0.0).min(1.0),
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_dynamic_occlusion_map() {
        let contacts = vec![
            ArticulatorContact {
                upper_tooth: "16".into(), lower_tooth: "46".into(),
                position_mm: [-20.0, 0.0, 0.0], force_n: 5.0,
                movement: JawMovement::Closing, time_s: 0.0,
                is_interference: false,
            },
            ArticulatorContact {
                upper_tooth: "26".into(), lower_tooth: "36".into(),
                position_mm: [20.0, 0.0, 0.0], force_n: 5.0,
                movement: JawMovement::Closing, time_s: 0.0,
                is_interference: false,
            },
        ];
        let map = compute_dynamic_occlusion_map(&contacts);
        assert!((map.total_force_n - 10.0).abs() < 0.01);
        assert!((map.balance_score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gothic_arch_symmetric() {
        let g = analyze_gothic_arch(25.0, 25.0);
        assert!((g.symmetry_score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gothic_arch_asymmetric() {
        let g = analyze_gothic_arch(30.0, 20.0);
        assert!(g.symmetry_score < 1.0);
        assert!(g.symmetry_score > 0.5);
    }

    #[test]
    fn test_occlusion_map_unbalanced() {
        let contacts = vec![
            ArticulatorContact {
                upper_tooth: "16".into(), lower_tooth: "46".into(),
                position_mm: [-20.0, 0.0, 0.0], force_n: 10.0,
                movement: JawMovement::Closing, time_s: 0.0,
                is_interference: false,
            },
        ];
        let map = compute_dynamic_occlusion_map(&contacts);
        assert!(map.balance_score < 1.0);
    }
}

//! Virtual articulator simulation for mandibular movement

use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Articulator design family
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArticulatorType {
    Arcon,
    NonArcon,
    SemiAdjustable,
    Fully,
}

/// Articulator mounting record with key clinical parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountingRecord {
    /// Sagittal condylar inclination in degrees (Bennett's angle)
    pub condylar_inclination: f64,
    /// Bennett angle (lateral shift) in degrees
    pub bennett_angle: f64,
    /// Intercondylar distance in mm
    pub intercondylar_distance: f64,
    /// Incisal guide angle in degrees
    pub incisal_guide_angle: f64,
}

impl Default for MountingRecord {
    fn default() -> Self {
        Self {
            condylar_inclination: 40.0,
            bennett_angle: 15.0,
            intercondylar_distance: 110.0,
            incisal_guide_angle: 25.0,
        }
    }
}

/// Type of mandibular excursion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExcursionType {
    Protrusive,
    LeftLateral,
    RightLateral,
}

/// Simulate a mandibular excursion and return the resulting isometry of the
/// mandibular model relative to the maxillary model.
///
/// The returned `Isometry3` represents the rigid transformation of the lower
/// arch at the given excursion angle in degrees.
pub fn simulate_excursion(
    mounting: &MountingRecord,
    excursion_type: ExcursionType,
    angle_deg: f64,
) -> Isometry3<f64> {
    let t = angle_deg.to_radians();

    // Rotation axis and condylar translation based on excursion type
    let (axis, forward_mm) = match excursion_type {
        ExcursionType::Protrusive => {
            let slope = mounting.condylar_inclination.to_radians();
            let fwd = angle_deg * slope.tan();
            (Vector3::x(), fwd)
        }
        ExcursionType::LeftLateral => {
            let bennett = mounting.bennett_angle.to_radians();
            let fwd = angle_deg * bennett.tan();
            (Vector3::y(), fwd)
        }
        ExcursionType::RightLateral => {
            let bennett = mounting.bennett_angle.to_radians();
            let fwd = angle_deg * bennett.tan();
            (-Vector3::y(), fwd)
        }
    };

    let rotation = UnitQuaternion::from_axis_angle(&nalgebra::Unit::new_normalize(axis), t * 0.1);
    let translation = Translation3::new(0.0, forward_mm, 0.0);

    Isometry3::from_parts(translation, rotation)
}

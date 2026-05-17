//! Surgical guide planning and drill protocol generation

use serde::{Deserialize, Serialize};
use crate::library::ImplantDefinition;

/// Type of surgical guide support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurgicalGuideType {
    ToothSupported,
    MucosaSupported,
    BoneSupported,
}

/// Misch bone density classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoneDensity {
    /// Dense cortical bone (D1)
    D1,
    /// Thick cortical with dense cancellous (D2)
    D2,
    /// Thin cortical with dense-to-medium cancellous (D3)
    D3,
    /// Almost entirely cancellous (D4)
    D4,
}

/// A single drilling step in the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillStep {
    pub drill_name: String,
    pub diameter: f64,
    pub depth: f64,
    pub rpm: u32,
    pub torque: f64,
}

/// Complete drill sequence for an implant site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillProtocol {
    pub steps: Vec<DrillStep>,
}

/// Generate a step-by-step drill protocol for the given implant and bone density.
///
/// Follows a standard under-preparation philosophy for soft bone and
/// full-size preparation for dense bone.
pub fn generate_drill_protocol(implant: &ImplantDefinition, bone_density: BoneDensity) -> DrillProtocol {
    let length = implant.length;
    let final_dia = implant.diameter;

    // Under-preparation factor based on bone density (D4 = strongest under-prep)
    let underprepare = match bone_density {
        BoneDensity::D1 => 0.0,  // full diameter
        BoneDensity::D2 => 0.1,
        BoneDensity::D3 => 0.2,
        BoneDensity::D4 => 0.4,
    };

    let (rpm_low, rpm_high): (u32, u32) = match bone_density {
        BoneDensity::D1 => (600, 800),
        BoneDensity::D2 => (500, 700),
        BoneDensity::D3 => (400, 600),
        BoneDensity::D4 => (300, 500),
    };

    let mut steps = Vec::new();

    // Step 1: Round bur / site marking
    steps.push(DrillStep {
        drill_name: "Round Bur 1.5 mm".into(),
        diameter: 1.5,
        depth: 2.0,
        rpm: rpm_high,
        torque: 10.0,
    });

    // Step 2: Pilot drill
    steps.push(DrillStep {
        drill_name: "Pilot Drill 2.0 mm".into(),
        diameter: 2.0,
        depth: length,
        rpm: rpm_high,
        torque: 20.0,
    });

    // Intermediate drills up to final diameter
    let mut d = 2.8;
    let effective_final = final_dia * (1.0 - underprepare);
    while d < effective_final - 0.05 {
        steps.push(DrillStep {
            drill_name: format!("Twist Drill {:.1} mm", d),
            diameter: d,
            depth: length,
            rpm: rpm_low,
            torque: 25.0,
        });
        d += 0.4;
    }

    // Final drill to effective diameter
    steps.push(DrillStep {
        drill_name: format!("Final Drill {:.1} mm", effective_final),
        diameter: effective_final,
        depth: length,
        rpm: rpm_low,
        torque: 30.0,
    });

    // Countersink if needed
    if final_dia >= 4.0 {
        steps.push(DrillStep {
            drill_name: "Countersink".into(),
            diameter: final_dia + 0.8,
            depth: 1.5,
            rpm: 300,
            torque: 15.0,
        });
    }

    // Tap for D1/D2
    if matches!(bone_density, BoneDensity::D1 | BoneDensity::D2) {
        steps.push(DrillStep {
            drill_name: format!("Bone Tap {:.1} mm", final_dia),
            diameter: final_dia,
            depth: length,
            rpm: 15,
            torque: implant.torque_insertion,
        });
    }

    DrillProtocol { steps }
}

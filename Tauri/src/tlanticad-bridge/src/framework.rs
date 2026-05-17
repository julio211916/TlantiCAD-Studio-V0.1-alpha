//! Full bridge framework composed of abutment and pontic units

use serde::{Deserialize, Serialize};
use crate::connector::ConnectorParams;

/// Type of bridge unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BridgeUnitType {
    Abutment,
    Pontic,
}

/// A single unit in the bridge (abutment crown or pontic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeUnit {
    pub tooth_number: u8,
    pub unit_type: BridgeUnitType,
}

/// Complete bridge framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeFramework {
    pub units: Vec<BridgeUnit>,
    pub connectors: Vec<ConnectorParams>,
    pub material: String,
}

impl BridgeFramework {
    /// Create a new bridge framework from a list of units and a material name.
    ///
    /// Connectors are automatically initialised with default parameters between
    /// consecutive units.
    pub fn new(units: Vec<BridgeUnit>, material: impl Into<String>) -> Self {
        let connector_count = if units.len() > 1 { units.len() - 1 } else { 0 };
        let connectors = (0..connector_count)
            .map(|_| ConnectorParams::default())
            .collect();
        Self {
            units,
            connectors,
            material: material.into(),
        }
    }

    /// Validate the bridge framework and return a list of error messages.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.units.len() < 2 {
            errors.push("A bridge requires at least 2 units".into());
            return errors;
        }

        // Count abutments
        let abutment_count = self.units.iter().filter(|u| u.unit_type == BridgeUnitType::Abutment).count();
        if abutment_count < 2 {
            errors.push("A bridge requires at least 2 abutments".into());
        }

        // First and last units must be abutments
        if let Some(first) = self.units.first() {
            if first.unit_type != BridgeUnitType::Abutment {
                errors.push("The first unit must be an abutment".into());
            }
        }
        if let Some(last) = self.units.last() {
            if last.unit_type != BridgeUnitType::Abutment {
                errors.push("The last unit must be an abutment".into());
            }
        }

        // Check consecutive pontics (max 2 pontics between abutments is common limit)
        let mut consecutive_pontics = 0usize;
        for unit in &self.units {
            if unit.unit_type == BridgeUnitType::Pontic {
                consecutive_pontics += 1;
                if consecutive_pontics > 2 {
                    errors.push("More than 2 consecutive pontics may exceed material strength limits".into());
                    break;
                }
            } else {
                consecutive_pontics = 0;
            }
        }

        if self.material.is_empty() {
            errors.push("Material must not be empty".into());
        }

        errors
    }

    /// Estimate total span from first to last abutment tooth number (FDI difference × 7 mm).
    pub fn total_span_mm(&self) -> f64 {
        let abutments: Vec<&BridgeUnit> = self.units.iter()
            .filter(|u| u.unit_type == BridgeUnitType::Abutment)
            .collect();
        if abutments.len() < 2 {
            return 0.0;
        }
        let first = abutments.first().unwrap().tooth_number;
        let last = abutments.last().unwrap().tooth_number;
        (last as i32 - first as i32).unsigned_abs() as f64 * 7.0
    }
}

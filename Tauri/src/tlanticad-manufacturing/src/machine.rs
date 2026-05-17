//! S339-S342: Machine Configuration & Management
//!
//! Define CNC mills, 3D printers, and their capabilities.

use serde::{Deserialize, Serialize};
use crate::milling::MillingAxes;
use crate::printing::PrintTechnology;

/// Machine type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachineType {
    CncMill,
    Printer3D,
    Sintering,
    Scanner,
}

/// Machine status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachineStatus {
    Idle,
    Running,
    Maintenance,
    Error,
    Offline,
}

/// Tool holder / spindle spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpindleSpec {
    pub max_rpm: f64,
    pub min_rpm: f64,
    pub power_kw: f64,
    pub tool_holder: String,
    pub auto_tool_change: bool,
    pub max_tools: u8,
}

/// CNC mill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CncMillConfig {
    pub name: String,
    pub axes: MillingAxes,
    pub work_envelope_mm: [f64; 3],
    pub spindle: SpindleSpec,
    pub supports_wet_milling: bool,
    pub supports_dry_milling: bool,
    pub disc_compatible: bool,
    pub max_disc_diameter_mm: f64,
}

impl CncMillConfig {
    pub fn default_5axis() -> Self {
        Self {
            name: "5-Axis Dental Mill".into(),
            axes: MillingAxes::FiveAxis,
            work_envelope_mm: [100.0, 100.0, 40.0],
            spindle: SpindleSpec {
                max_rpm: 60000.0, min_rpm: 5000.0, power_kw: 3.0,
                tool_holder: "ER11".into(), auto_tool_change: true, max_tools: 12,
            },
            supports_wet_milling: true,
            supports_dry_milling: true,
            disc_compatible: true,
            max_disc_diameter_mm: 98.0,
        }
    }

    pub fn can_mill_disc(&self, diameter: f64) -> bool {
        self.disc_compatible && diameter <= self.max_disc_diameter_mm
    }
}

/// 3D printer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    pub name: String,
    pub technology: PrintTechnology,
    pub build_volume_mm: [f64; 3],
    pub xy_resolution_um: f64,
    pub min_layer_height_um: f64,
    pub max_layer_height_um: f64,
    pub light_source: Option<String>,
}

impl PrinterConfig {
    pub fn default_dlp() -> Self {
        Self {
            name: "DLP Dental Printer".into(),
            technology: PrintTechnology::DLP,
            build_volume_mm: [192.0, 120.0, 200.0],
            xy_resolution_um: 50.0,
            min_layer_height_um: 25.0,
            max_layer_height_um: 200.0,
            light_source: Some("405nm UV LED".into()),
        }
    }

    pub fn build_volume_liters(&self) -> f64 {
        self.build_volume_mm.iter().product::<f64>() / 1e6
    }
}

/// Machine registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineRegistry {
    pub mills: Vec<CncMillConfig>,
    pub printers: Vec<PrinterConfig>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self { mills: Vec::new(), printers: Vec::new() }
    }

    pub fn add_mill(&mut self, mill: CncMillConfig) { self.mills.push(mill); }
    pub fn add_printer(&mut self, printer: PrinterConfig) { self.printers.push(printer); }

    pub fn find_mill_for_disc(&self, diameter: f64) -> Option<&CncMillConfig> {
        self.mills.iter().find(|m| m.can_mill_disc(diameter))
    }

    pub fn total_machines(&self) -> usize { self.mills.len() + self.printers.len() }
}

impl Default for MachineRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mill() {
        let mill = CncMillConfig::default_5axis();
        assert_eq!(mill.axes, MillingAxes::FiveAxis);
        assert!(mill.can_mill_disc(98.0));
        assert!(!mill.can_mill_disc(120.0));
    }

    #[test]
    fn test_default_printer() {
        let printer = PrinterConfig::default_dlp();
        assert_eq!(printer.technology, PrintTechnology::DLP);
        assert!(printer.build_volume_liters() > 0.0);
    }

    #[test]
    fn test_machine_registry() {
        let mut reg = MachineRegistry::new();
        reg.add_mill(CncMillConfig::default_5axis());
        reg.add_printer(PrinterConfig::default_dlp());
        assert_eq!(reg.total_machines(), 2);
        assert!(reg.find_mill_for_disc(98.0).is_some());
    }

    #[test]
    fn test_spindle_spec() {
        let spec = SpindleSpec {
            max_rpm: 60000.0, min_rpm: 5000.0, power_kw: 3.0,
            tool_holder: "ER11".into(), auto_tool_change: true, max_tools: 12,
        };
        assert!(spec.auto_tool_change);
        assert!(spec.max_rpm > spec.min_rpm);
    }
}

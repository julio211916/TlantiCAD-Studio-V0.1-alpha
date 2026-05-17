//! Bar attachment types and retention forces

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Overdenture attachment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentType {
    LocatorR,
    Ball,
    Magnets,
    ERA,
    Dalbo,
}

/// A single attachment on a bar framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarAttachment {
    pub position: Point3<f64>,
    pub attachment_type: AttachmentType,
    pub retention_force_n: f64,
}

/// Get the retention force for an attachment type given a wear factor (0 = new, 1 = fully worn).
///
/// Initial retention forces are based on manufacturer specifications.
pub fn get_retention_force(attachment: &AttachmentType, wear_factor: f64) -> f64 {
    let wear = wear_factor.clamp(0.0, 1.0);
    let initial = match attachment {
        AttachmentType::LocatorR => 20.0,  // Newtons, blue insert
        AttachmentType::Ball => 10.0,
        AttachmentType::Magnets => 6.0,
        AttachmentType::ERA => 15.0,
        AttachmentType::Dalbo => 12.0,
    };
    // Linear wear model: force drops to 40% at full wear
    initial * (1.0 - wear * 0.6)
}

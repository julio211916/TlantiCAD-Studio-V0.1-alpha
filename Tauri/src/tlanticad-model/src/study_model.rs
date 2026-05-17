//! Study model type and structure

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::die::ToothDie;
use crate::base::ModelBase;

/// Type of dental study model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    FullArch,
    QuadrantUpper,
    QuadrantLower,
    Diagnostic,
    WorkingModel,
}

/// Dental arch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchType {
    Upper,
    Lower,
    Both,
}

/// A complete study model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyModel {
    pub id: Uuid,
    pub model_type: ModelType,
    pub arch: ArchType,
    pub scan_source: String,
    pub teeth: Vec<ToothDie>,
    pub base: ModelBase,
}

impl StudyModel {
    /// Create a new study model
    pub fn new(model_type: ModelType, arch: ArchType, scan_source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            model_type,
            arch,
            scan_source: scan_source.into(),
            teeth: Vec::new(),
            base: ModelBase::default(),
        }
    }

    /// Add a tooth die to the model
    pub fn add_tooth(&mut self, die: ToothDie) {
        self.teeth.push(die);
    }

    /// Find a tooth die by FDI number
    pub fn find_tooth(&self, fdi_number: u8) -> Option<&ToothDie> {
        self.teeth.iter().find(|t| t.fdi_number == fdi_number)
    }
}

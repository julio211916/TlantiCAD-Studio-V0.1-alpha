//! Tooth library management

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Tooth set (e.g., "generic", "alternative", "psarris")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothSet {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub upper_jaw: Vec<Tooth>,
    pub lower_jaw: Vec<Tooth>,
}

/// Individual tooth model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tooth {
    pub number: u8,
    pub name: String,
    pub stl_path: Option<PathBuf>,
    pub is_anterior: bool,
    pub is_molar: bool,
    pub is_wisdom: bool,
}

impl ToothSet {
    /// Load tooth set from directory
    pub async fn from_directory(path: &Path, name: &str) -> tlanticad_core::Result<Self> {
        let mut upper_jaw = Vec::new();
        let mut lower_jaw = Vec::new();

        // Upper jaw
        let upper_path = path.join("upperjaw");
        if upper_path.exists() {
            upper_jaw = Self::scan_tooth_files(&upper_path, true).await?;
        }

        // Lower jaw
        let lower_path = path.join("lowerjaw");
        if lower_path.exists() {
            lower_jaw = Self::scan_tooth_files(&lower_path, false).await?;
        }

        Ok(Self {
            id: name.to_lowercase().replace(" ", "_"),
            name: name.to_string(),
            path: path.to_path_buf(),
            upper_jaw,
            lower_jaw,
        })
    }

    async fn scan_tooth_files(path: &Path, is_upper: bool) -> tlanticad_core::Result<Vec<Tooth>> {
        let mut teeth = Vec::new();

        let mut entries = tokio::fs::read_dir(path).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))? {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if ext == "stl" || ext == "obj" {
                    if let Some(tooth) = Self::parse_tooth_file(&file_path, is_upper).await {
                        teeth.push(tooth);
                    }
                }
            }
        }

        // Sort by tooth number
        teeth.sort_by_key(|t| t.number);

        Ok(teeth)
    }

    async fn parse_tooth_file(path: &Path, _is_upper: bool) -> Option<Tooth> {
        let file_name = path.file_stem()?.to_string_lossy();
        
        // Try to extract tooth number from filename
        // Common patterns: "11.stl", "tooth_11.stl", "upper_11.stl"
        let number = Self::extract_tooth_number(&file_name)?;
        
        let name = Self::get_tooth_name(number);
        let is_anterior = (11..=22).contains(&number) || (31..=42).contains(&number);
        let is_molar = (16..=17).contains(&number) || (26..=27).contains(&number) ||
                       (36..=37).contains(&number) || (46..=47).contains(&number);
        let is_wisdom = number == 18 || number == 28 || number == 38 || number == 48;

        Some(Tooth {
            number,
            name,
            stl_path: Some(path.to_path_buf()),
            is_anterior,
            is_molar,
            is_wisdom,
        })
    }

    fn extract_tooth_number(file_name: &str) -> Option<u8> {
        // Try different patterns
        // Pattern 1: "11.stl" - just the number
        if let Ok(num) = file_name.parse::<u8>() {
            if (11..=48).contains(&num) {
                return Some(num);
            }
        }

        // Pattern 2: "tooth_11.stl" or "upper_11.stl"
        let parts: Vec<&str> = file_name.split('_').collect();
        for part in parts {
            if let Ok(num) = part.parse::<u8>() {
                if (11..=48).contains(&num) {
                    return Some(num);
                }
            }
        }

        None
    }

    fn get_tooth_name(number: u8) -> String {
        match number {
            11 | 21 => "Central Incisor".to_string(),
            12 | 22 => "Lateral Incisor".to_string(),
            13 | 23 => "Canine".to_string(),
            14 | 15 | 24 | 25 => "Premolar".to_string(),
            16 | 17 | 26 | 27 => "Molar".to_string(),
            18 | 28 => "Wisdom Tooth".to_string(),
            31 | 41 => "Central Incisor".to_string(),
            32 | 42 => "Lateral Incisor".to_string(),
            33 | 43 => "Canine".to_string(),
            34 | 35 | 44 | 45 => "Premolar".to_string(),
            36 | 37 | 46 | 47 => "Molar".to_string(),
            38 | 48 => "Wisdom Tooth".to_string(),
            _ => format!("Tooth {}", number),
        }
    }

    /// Get tooth by number
    pub fn get_tooth(&self, number: u8) -> Option<&Tooth> {
        if number >= 11 && number <= 28 {
            self.upper_jaw.iter().find(|t| t.number == number)
        } else {
            self.lower_jaw.iter().find(|t| t.number == number)
        }
    }

    /// Get tooth by position (FDI notation)
    pub fn get_tooth_by_fdi(&self, quadrant: u8, position: u8) -> Option<&Tooth> {
        let number = quadrant * 10 + position;
        self.get_tooth(number)
    }

    /// Get all teeth
    pub fn get_all_teeth(&self) -> Vec<&Tooth> {
        let mut all: Vec<&Tooth> = self.upper_jaw.iter()
            .chain(self.lower_jaw.iter())
            .collect();
        all.sort_by_key(|t| t.number);
        all
    }

    /// Get anterior teeth (incisors and canines)
    pub fn get_anterior_teeth(&self) -> Vec<&Tooth> {
        self.get_all_teeth()
            .into_iter()
            .filter(|t| t.is_anterior)
            .collect()
    }

    /// Get molars
    pub fn get_molars(&self) -> Vec<&Tooth> {
        self.get_all_teeth()
            .into_iter()
            .filter(|t| t.is_molar)
            .collect()
    }
}

/// Tooth set info (for UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothSetInfo {
    pub id: String,
    pub name: String,
    pub upper_tooth_count: usize,
    pub lower_tooth_count: usize,
    pub is_complete: bool,
}

impl From<&ToothSet> for ToothSetInfo {
    fn from(set: &ToothSet) -> Self {
        Self {
            id: set.id.clone(),
            name: set.name.clone(),
            upper_tooth_count: set.upper_jaw.len(),
            lower_tooth_count: set.lower_jaw.len(),
            is_complete: set.upper_jaw.len() >= 16 && set.lower_jaw.len() >= 16,
        }
    }
}

/// Tooth position in dental arch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToothPosition {
    UpperRight,
    UpperLeft,
    LowerRight,
    LowerLeft,
}

impl ToothPosition {
    pub fn quadrant(&self) -> u8 {
        match self {
            ToothPosition::UpperRight => 1,
            ToothPosition::UpperLeft => 2,
            ToothPosition::LowerLeft => 3,
            ToothPosition::LowerRight => 4,
        }
    }
}

//! Odontogram (Dental Chart) domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{ToothCondition, ToothSurface};

/// Odontogram entry - represents the state of a single tooth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdontogramEntry {
    pub id: Uuid,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Tooth number (FDI notation: 11-18, 21-28, 31-38, 41-48)
    pub tooth_number: i32,
    
    /// Surface conditions (map of surface to condition)
    pub surface_conditions: Vec<SurfaceCondition>,
    
    /// Overall tooth condition (primary condition)
    pub primary_condition: ToothCondition,
    
    /// Current treatment status if any
    pub treatment_status: Option<String>,
    
    /// Is this a primary (deciduous) tooth
    pub is_primary: bool,
    
    /// Mobility grade (0-3)
    pub mobility: Option<i32>,
    
    /// Notes for this tooth
    pub notes: Option<String>,
    
    /// Last updated
    pub updated_at: DateTime<Utc>,
    
    /// Last updated by
    pub updated_by: Uuid,
}

/// Surface condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceCondition {
    pub surface: ToothSurface,
    pub condition: ToothCondition,
    pub notes: Option<String>,
}

impl OdontogramEntry {
    pub fn new(patient_id: Uuid, tooth_number: i32, updated_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            tooth_number,
            surface_conditions: Vec::new(),
            primary_condition: ToothCondition::Healthy,
            treatment_status: None,
            is_primary: false,
            mobility: None,
            notes: None,
            updated_at: Utc::now(),
            updated_by,
        }
    }
    
    /// Check if tooth is in upper arch
    pub fn is_upper(&self) -> bool {
        self.tooth_number >= 11 && self.tooth_number <= 28
    }
    
    /// Check if tooth is in lower arch
    pub fn is_lower(&self) -> bool {
        self.tooth_number >= 31 && self.tooth_number <= 48
    }
    
    /// Get quadrant (1-4)
    pub fn quadrant(&self) -> i32 {
        match self.tooth_number {
            11..=18 => 1,
            21..=28 => 2,
            31..=38 => 3,
            41..=48 => 4,
            // Primary teeth (deciduous)
            51..=55 => 5,
            61..=65 => 6,
            71..=75 => 7,
            81..=85 => 8,
            _ => 0,
        }
    }
}

/// Complete patient odontogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Odontogram {
    pub patient_id: Uuid,
    pub entries: Vec<OdontogramEntry>,
    pub last_updated: DateTime<Utc>,
}

impl Odontogram {
    /// Create a new empty odontogram for a patient
    pub fn new(patient_id: Uuid) -> Self {
        Self {
            patient_id,
            entries: Vec::new(),
            last_updated: Utc::now(),
        }
    }
    
    /// Get entry for a specific tooth
    pub fn get_tooth(&self, tooth_number: i32) -> Option<&OdontogramEntry> {
        self.entries.iter().find(|e| e.tooth_number == tooth_number)
    }
    
    /// Get all upper teeth
    pub fn upper_teeth(&self) -> Vec<&OdontogramEntry> {
        self.entries.iter().filter(|e| e.is_upper()).collect()
    }
    
    /// Get all lower teeth
    pub fn lower_teeth(&self) -> Vec<&OdontogramEntry> {
        self.entries.iter().filter(|e| e.is_lower()).collect()
    }
    
    /// Get teeth by quadrant
    pub fn teeth_by_quadrant(&self, quadrant: i32) -> Vec<&OdontogramEntry> {
        self.entries.iter().filter(|e| e.quadrant() == quadrant).collect()
    }
    
    /// Count teeth with a specific condition
    pub fn count_by_condition(&self, condition: ToothCondition) -> usize {
        self.entries.iter().filter(|e| e.primary_condition == condition).count()
    }
}

/// Update odontogram entry DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOdontogramEntry {
    pub tooth_number: i32,
    pub surface_conditions: Option<Vec<SurfaceCondition>>,
    pub primary_condition: Option<ToothCondition>,
    pub treatment_status: Option<String>,
    pub is_primary: Option<bool>,
    pub mobility: Option<i32>,
    pub notes: Option<String>,
}

/// Odontogram history entry - tracks changes over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdontogramHistory {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub tooth_number: i32,
    pub previous_condition: ToothCondition,
    pub new_condition: ToothCondition,
    pub change_reason: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// Dental chart summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdontogramSummary {
    pub patient_id: Uuid,
    pub total_teeth: i32,
    pub healthy: i32,
    pub caries: i32,
    pub fillings: i32,
    pub crowns: i32,
    pub missing: i32,
    pub root_canals: i32,
    pub implants: i32,
    pub extractions_needed: i32,
    pub last_updated: DateTime<Utc>,
}

impl OdontogramSummary {
    pub fn from_odontogram(odontogram: &Odontogram) -> Self {
        Self {
            patient_id: odontogram.patient_id,
            total_teeth: odontogram.entries.len() as i32,
            healthy: odontogram.count_by_condition(ToothCondition::Healthy) as i32,
            caries: odontogram.count_by_condition(ToothCondition::Caries) as i32,
            fillings: odontogram.count_by_condition(ToothCondition::Filling) as i32,
            crowns: odontogram.count_by_condition(ToothCondition::Crown) as i32,
            missing: odontogram.count_by_condition(ToothCondition::Missing) as i32,
            root_canals: odontogram.count_by_condition(ToothCondition::RootCanal) as i32,
            implants: odontogram.count_by_condition(ToothCondition::Implant) as i32,
            extractions_needed: odontogram.count_by_condition(ToothCondition::Extraction) as i32,
            last_updated: odontogram.last_updated,
        }
    }
}

/// Tooth numbering notation system
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotationSystem {
    /// FDI World Dental Federation (ISO 3950) - International standard
    /// Quadrant (1-4 permanent, 5-8 deciduous) + Tooth (1-8)
    /// Used in most countries worldwide
    Fdi,
    /// Universal Numbering System (ADA) - Used in USA
    /// Adults: 1-32, Deciduous: A-T
    Universal,
    /// Palmer Notation - Used in UK, parts of Europe
    /// Quadrant symbol + tooth number 1-8
    Palmer,
}

/// A tooth identifier that can convert between notation systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothId {
    /// FDI number (canonical storage format)
    pub fdi: i32,
    /// Universal number equivalent
    pub universal: String,
    /// Palmer notation equivalent
    pub palmer: String,
    /// Human-readable name
    pub name: String,
    /// Quadrant (1-4 for permanent, 5-8 for deciduous)
    pub quadrant: i32,
    /// Position within quadrant (1-8)
    pub position: i32,
    /// Whether this is a deciduous (primary/baby) tooth
    pub is_deciduous: bool,
}

impl ToothId {
    /// Create a ToothId from an FDI number
    pub fn from_fdi(fdi: i32) -> Option<Self> {
        let quadrant = fdi / 10;
        let position = fdi % 10;

        if position < 1 || position > 8 {
            return None;
        }

        let is_deciduous = quadrant >= 5 && quadrant <= 8;

        if is_deciduous && position > 5 {
            return None;
        }

        if !(1..=8).contains(&quadrant) {
            return None;
        }

        Some(Self {
            fdi,
            universal: fdi_to_universal(fdi),
            palmer: fdi_to_palmer(fdi),
            name: ToothNumbers::name(fdi).to_string(),
            quadrant,
            position,
            is_deciduous,
        })
    }

    /// Create a ToothId from a Universal number (1-32 or A-T)
    pub fn from_universal(uni: &str) -> Option<Self> {
        let fdi = universal_to_fdi(uni)?;
        Self::from_fdi(fdi)
    }
}

/// Convert FDI number to Universal notation
pub fn fdi_to_universal(fdi: i32) -> String {
    match fdi {
        // Upper right (Q1) → Universal 1-8
        18 => "1".into(), 17 => "2".into(), 16 => "3".into(), 15 => "4".into(),
        14 => "5".into(), 13 => "6".into(), 12 => "7".into(), 11 => "8".into(),
        // Upper left (Q2) → Universal 9-16
        21 => "9".into(), 22 => "10".into(), 23 => "11".into(), 24 => "12".into(),
        25 => "13".into(), 26 => "14".into(), 27 => "15".into(), 28 => "16".into(),
        // Lower left (Q3) → Universal 17-24
        38 => "17".into(), 37 => "18".into(), 36 => "19".into(), 35 => "20".into(),
        34 => "21".into(), 33 => "22".into(), 32 => "23".into(), 31 => "24".into(),
        // Lower right (Q4) → Universal 25-32
        41 => "25".into(), 42 => "26".into(), 43 => "27".into(), 44 => "28".into(),
        45 => "29".into(), 46 => "30".into(), 47 => "31".into(), 48 => "32".into(),
        // Deciduous upper right (Q5) → Universal A-E
        55 => "A".into(), 54 => "B".into(), 53 => "C".into(), 52 => "D".into(), 51 => "E".into(),
        // Deciduous upper left (Q6) → Universal F-J
        61 => "F".into(), 62 => "G".into(), 63 => "H".into(), 64 => "I".into(), 65 => "J".into(),
        // Deciduous lower left (Q7) → Universal K-O
        75 => "K".into(), 74 => "L".into(), 73 => "M".into(), 72 => "N".into(), 71 => "O".into(),
        // Deciduous lower right (Q8) → Universal P-T
        81 => "P".into(), 82 => "Q".into(), 83 => "R".into(), 84 => "S".into(), 85 => "T".into(),
        _ => format!("?{}", fdi),
    }
}

/// Convert Universal notation to FDI number
pub fn universal_to_fdi(uni: &str) -> Option<i32> {
    // Try as number first (adult teeth 1-32)
    if let Ok(n) = uni.parse::<i32>() {
        return match n {
            1 => Some(18), 2 => Some(17), 3 => Some(16), 4 => Some(15),
            5 => Some(14), 6 => Some(13), 7 => Some(12), 8 => Some(11),
            9 => Some(21), 10 => Some(22), 11 => Some(23), 12 => Some(24),
            13 => Some(25), 14 => Some(26), 15 => Some(27), 16 => Some(28),
            17 => Some(38), 18 => Some(37), 19 => Some(36), 20 => Some(35),
            21 => Some(34), 22 => Some(33), 23 => Some(32), 24 => Some(31),
            25 => Some(41), 26 => Some(42), 27 => Some(43), 28 => Some(44),
            29 => Some(45), 30 => Some(46), 31 => Some(47), 32 => Some(48),
            _ => None,
        };
    }

    // Try as letter (deciduous teeth A-T)
    match uni.to_uppercase().as_str() {
        "A" => Some(55), "B" => Some(54), "C" => Some(53), "D" => Some(52), "E" => Some(51),
        "F" => Some(61), "G" => Some(62), "H" => Some(63), "I" => Some(64), "J" => Some(65),
        "K" => Some(75), "L" => Some(74), "M" => Some(73), "N" => Some(72), "O" => Some(71),
        "P" => Some(81), "Q" => Some(82), "R" => Some(83), "S" => Some(84), "T" => Some(85),
        _ => None,
    }
}

/// Convert FDI number to Palmer notation
pub fn fdi_to_palmer(fdi: i32) -> String {
    let quadrant = fdi / 10;
    let position = fdi % 10;
    let symbol = match quadrant {
        1 => "┘", // Upper Right
        2 => "└", // Upper Left
        3 => "┌", // Lower Left
        4 => "┐", // Lower Right
        5 => "┘", // Deciduous UR
        6 => "└", // Deciduous UL
        7 => "┌", // Deciduous LL
        8 => "┐", // Deciduous LR
        _ => "?",
    };

    if quadrant >= 5 {
        // Deciduous: use letters a-e
        let letter = (b'a' + (position as u8 - 1)) as char;
        format!("{}{}", letter, symbol)
    } else {
        format!("{}{}", position, symbol)
    }
}

/// Standard tooth numbers
pub struct ToothNumbers;

impl ToothNumbers {
    /// All adult permanent teeth (FDI notation)
    pub const PERMANENT: [i32; 32] = [
        // Upper right (Q1)
        18, 17, 16, 15, 14, 13, 12, 11,
        // Upper left (Q2)
        21, 22, 23, 24, 25, 26, 27, 28,
        // Lower left (Q3)
        38, 37, 36, 35, 34, 33, 32, 31,
        // Lower right (Q4)
        41, 42, 43, 44, 45, 46, 47, 48,
    ];

    /// All primary (deciduous) teeth (FDI notation)
    pub const PRIMARY: [i32; 20] = [
        // Upper right (Q5)
        55, 54, 53, 52, 51,
        // Upper left (Q6)
        61, 62, 63, 64, 65,
        // Lower left (Q7)
        75, 74, 73, 72, 71,
        // Lower right (Q8)
        81, 82, 83, 84, 85,
    ];

    /// Universal numbering: all adult teeth 1-32
    pub const UNIVERSAL_PERMANENT: [&'static str; 32] = [
        "1", "2", "3", "4", "5", "6", "7", "8",
        "9", "10", "11", "12", "13", "14", "15", "16",
        "17", "18", "19", "20", "21", "22", "23", "24",
        "25", "26", "27", "28", "29", "30", "31", "32",
    ];

    /// Universal numbering: deciduous teeth A-T
    pub const UNIVERSAL_PRIMARY: [&'static str; 20] = [
        "A", "B", "C", "D", "E",
        "F", "G", "H", "I", "J",
        "K", "L", "M", "N", "O",
        "P", "Q", "R", "S", "T",
    ];

    /// Get all teeth as ToothId objects for a given notation
    pub fn all_permanent() -> Vec<ToothId> {
        Self::PERMANENT
            .iter()
            .filter_map(|&fdi| ToothId::from_fdi(fdi))
            .collect()
    }

    /// Get all deciduous teeth as ToothId objects
    pub fn all_deciduous() -> Vec<ToothId> {
        Self::PRIMARY
            .iter()
            .filter_map(|&fdi| ToothId::from_fdi(fdi))
            .collect()
    }

    /// Get tooth name by FDI number
    pub fn name(tooth_number: i32) -> &'static str {
        match tooth_number {
            11 | 21 | 51 | 61 => "Central Incisor",
            12 | 22 | 52 | 62 => "Lateral Incisor",
            13 | 23 | 53 | 63 => "Canine",
            14 | 24 => "First Premolar",
            15 | 25 => "Second Premolar",
            16 | 26 | 54 | 64 => "First Molar",
            17 | 27 | 55 | 65 => "Second Molar",
            18 | 28 => "Third Molar (Wisdom)",
            31 | 41 | 71 | 81 => "Central Incisor",
            32 | 42 | 72 | 82 => "Lateral Incisor",
            33 | 43 | 73 | 83 => "Canine",
            34 | 44 => "First Premolar",
            35 | 45 => "Second Premolar",
            36 | 46 | 74 | 84 => "First Molar",
            37 | 47 | 75 | 85 => "Second Molar",
            38 | 48 => "Third Molar (Wisdom)",
            _ => "Unknown",
        }
    }

    /// Get tooth name in Spanish
    pub fn nombre(tooth_number: i32) -> &'static str {
        match tooth_number {
            11 | 21 | 51 | 61 => "Incisivo Central",
            12 | 22 | 52 | 62 => "Incisivo Lateral",
            13 | 23 | 53 | 63 => "Canino",
            14 | 24 => "Primer Premolar",
            15 | 25 => "Segundo Premolar",
            16 | 26 | 54 | 64 => "Primer Molar",
            17 | 27 | 55 | 65 => "Segundo Molar",
            18 | 28 => "Tercer Molar (Muela del Juicio)",
            31 | 41 | 71 | 81 => "Incisivo Central",
            32 | 42 | 72 | 82 => "Incisivo Lateral",
            33 | 43 | 73 | 83 => "Canino",
            34 | 44 => "Primer Premolar",
            35 | 45 => "Segundo Premolar",
            36 | 46 | 74 | 84 => "Primer Molar",
            37 | 47 | 75 | 85 => "Segundo Molar",
            38 | 48 => "Tercer Molar (Muela del Juicio)",
            _ => "Desconocido",
        }
    }

    /// Quadrant name in English
    pub fn quadrant_name(quadrant: i32) -> &'static str {
        match quadrant {
            1 => "Upper Right",
            2 => "Upper Left",
            3 => "Lower Left",
            4 => "Lower Right",
            5 => "Upper Right (Deciduous)",
            6 => "Upper Left (Deciduous)",
            7 => "Lower Left (Deciduous)",
            8 => "Lower Right (Deciduous)",
            _ => "Unknown",
        }
    }

    /// Quadrant name in Spanish
    pub fn nombre_cuadrante(quadrant: i32) -> &'static str {
        match quadrant {
            1 => "Superior Derecho",
            2 => "Superior Izquierdo",
            3 => "Inferior Izquierdo",
            4 => "Inferior Derecho",
            5 => "Superior Derecho (Temporal)",
            6 => "Superior Izquierdo (Temporal)",
            7 => "Inferior Izquierdo (Temporal)",
            8 => "Inferior Derecho (Temporal)",
            _ => "Desconocido",
        }
    }
}

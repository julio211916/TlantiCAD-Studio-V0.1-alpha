//! FDI Two-Digit Notation (ISO 3950)
//! Permanent teeth: 11-18, 21-28, 31-38, 41-48
//! Primary teeth: 51-55, 61-65, 71-75, 81-85

use serde::{Deserialize, Serialize};

/// FDI quadrant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FdiQuadrant {
    /// Upper Right — permanent
    UpperRight = 1,
    /// Upper Left — permanent
    UpperLeft = 2,
    /// Lower Left — permanent
    LowerLeft = 3,
    /// Lower Right — permanent
    LowerRight = 4,
    /// Upper Right — primary
    UpperRightPrimary = 5,
    /// Upper Left — primary
    UpperLeftPrimary = 6,
    /// Lower Left — primary
    LowerLeftPrimary = 7,
    /// Lower Right — primary
    LowerRightPrimary = 8,
}

impl FdiQuadrant {
    pub fn from_fdi(fdi: u8) -> Option<Self> {
        match fdi / 10 {
            1 => Some(FdiQuadrant::UpperRight),
            2 => Some(FdiQuadrant::UpperLeft),
            3 => Some(FdiQuadrant::LowerLeft),
            4 => Some(FdiQuadrant::LowerRight),
            5 => Some(FdiQuadrant::UpperRightPrimary),
            6 => Some(FdiQuadrant::UpperLeftPrimary),
            7 => Some(FdiQuadrant::LowerLeftPrimary),
            8 => Some(FdiQuadrant::LowerRightPrimary),
            _ => None,
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self,
            FdiQuadrant::UpperRightPrimary | FdiQuadrant::UpperLeftPrimary |
            FdiQuadrant::LowerLeftPrimary  | FdiQuadrant::LowerRightPrimary
        )
    }

    pub fn is_upper(&self) -> bool {
        matches!(self,
            FdiQuadrant::UpperRight | FdiQuadrant::UpperLeft |
            FdiQuadrant::UpperRightPrimary | FdiQuadrant::UpperLeftPrimary
        )
    }

    pub fn is_right(&self) -> bool {
        matches!(self,
            FdiQuadrant::UpperRight | FdiQuadrant::LowerRight |
            FdiQuadrant::UpperRightPrimary | FdiQuadrant::LowerRightPrimary
        )
    }
}

/// Validated FDI tooth number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FdiTooth(pub u8);

impl FdiTooth {
    /// Create from raw u8, validates range
    pub fn new(n: u8) -> Option<Self> {
        if is_valid_fdi(n) { Some(FdiTooth(n)) } else { None }
    }

    pub fn quadrant(&self) -> Option<FdiQuadrant> {
        FdiQuadrant::from_fdi(self.0)
    }

    pub fn tooth_position(&self) -> u8 {
        self.0 % 10
    }

    pub fn is_primary(&self) -> bool {
        self.0 / 10 >= 5
    }

    pub fn is_molar(&self) -> bool {
        (6..=8).contains(&self.tooth_position())
    }

    pub fn is_premolar(&self) -> bool {
        (4..=5).contains(&self.tooth_position())
    }

    pub fn is_canine(&self) -> bool {
        self.tooth_position() == 3
    }

    pub fn is_incisor(&self) -> bool {
        (1..=2).contains(&self.tooth_position())
    }

    pub fn name(&self) -> String {
        let q = self.0 / 10;
        let p = self.0 % 10;
        let side = if q == 1 || q == 5 || q == 4 || q == 8 { "D" } else { "I" };
        let arch = if q <= 2 || (q >= 5 && q <= 6) { "Sup" } else { "Inf" };
        let tooth_type = match p {
            1 => "Inc Central",
            2 => "Inc Lateral",
            3 => "Canino",
            4 => "1er Premolar",
            5 => "2do Premolar",
            6 => "1er Molar",
            7 => "2do Molar",
            8 => "3er Molar",
            _ => "?",
        };
        format!("{} {} {}", arch, side, tooth_type)
    }
}

impl std::fmt::Display for FdiTooth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Check if an FDI number is valid
pub fn is_valid_fdi(n: u8) -> bool {
    let q = n / 10;
    let p = n % 10;
    match q {
        1..=4 => (1..=8).contains(&p),
        5..=8 => (1..=5).contains(&p),
        _ => false,
    }
}

/// All 32 permanent FDI tooth numbers
pub fn all_permanent_fdi() -> Vec<u8> {
    let mut teeth = Vec::with_capacity(32);
    for q in 1u8..=4 {
        for p in 1u8..=8 {
            teeth.push(q * 10 + p);
        }
    }
    teeth
}

/// All 20 primary FDI tooth numbers
pub fn all_primary_fdi() -> Vec<u8> {
    let mut teeth = Vec::with_capacity(20);
    for q in 5u8..=8 {
        for p in 1u8..=5 {
            teeth.push(q * 10 + p);
        }
    }
    teeth
}

/// Get adjacent teeth (mesial, distal) in FDI notation
pub fn adjacent_teeth(fdi: u8) -> (Option<u8>, Option<u8>) {
    let q = fdi / 10;
    let p = fdi % 10;
    let max_pos = if q >= 5 { 5u8 } else { 8u8 };

    let mesial = if p > 1 { Some(q * 10 + p - 1) } else { None };
    let distal = if p < max_pos { Some(q * 10 + p + 1) } else { None };

    (mesial, distal)
}

//! S3+S101: Dental notation — FDI, Universal, Palmer systems
//!
//! Provides bidirectional conversion between international tooth notation
//! systems including FDI (ISO 3950), Universal (ADA), and Palmer.

use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotationSystem {
    Fdi,
    Universal,
    Palmer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quadrant {
    UpperRight, // FDI 1, Palmer ┘
    UpperLeft,  // FDI 2, Palmer └
    LowerLeft,  // FDI 3, Palmer ┌
    LowerRight, // FDI 4, Palmer ┐
}

impl Quadrant {
    /// FDI quadrant number (1-4).
    pub fn fdi_number(self) -> u8 {
        match self {
            Self::UpperRight => 1,
            Self::UpperLeft => 2,
            Self::LowerLeft => 3,
            Self::LowerRight => 4,
        }
    }

    /// Palmer symbol for this quadrant.
    pub fn palmer_symbol(self) -> char {
        match self {
            Self::UpperRight => '┘',
            Self::UpperLeft => '└',
            Self::LowerLeft => '┌',
            Self::LowerRight => '┐',
        }
    }

    pub fn from_fdi_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::UpperRight),
            2 => Some(Self::UpperLeft),
            3 => Some(Self::LowerLeft),
            4 => Some(Self::LowerRight),
            _ => None,
        }
    }

    pub fn from_palmer_symbol(c: char) -> Option<Self> {
        match c {
            '┘' => Some(Self::UpperRight),
            '└' => Some(Self::UpperLeft),
            '┌' => Some(Self::LowerLeft),
            '┐' => Some(Self::LowerRight),
            _ => None,
        }
    }
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpperRight => write!(f, "UR"),
            Self::UpperLeft => write!(f, "UL"),
            Self::LowerLeft => write!(f, "LL"),
            Self::LowerRight => write!(f, "LR"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToothType {
    CentralIncisor,
    LateralIncisor,
    Canine,
    FirstPremolar,
    SecondPremolar,
    FirstMolar,
    SecondMolar,
    ThirdMolar,
}

impl ToothType {
    /// Palmer/FDI tooth number within quadrant (1-8).
    pub fn number(self) -> u8 {
        match self {
            Self::CentralIncisor => 1,
            Self::LateralIncisor => 2,
            Self::Canine => 3,
            Self::FirstPremolar => 4,
            Self::SecondPremolar => 5,
            Self::FirstMolar => 6,
            Self::SecondMolar => 7,
            Self::ThirdMolar => 8,
        }
    }

    pub fn from_number(n: u8) -> Option<Self> {
        TOOTH_TYPES.get((n.wrapping_sub(1)) as usize).copied()
    }
}

impl fmt::Display for ToothType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CentralIncisor => "Central Incisor",
            Self::LateralIncisor => "Lateral Incisor",
            Self::Canine => "Canine",
            Self::FirstPremolar => "1st Premolar",
            Self::SecondPremolar => "2nd Premolar",
            Self::FirstMolar => "1st Molar",
            Self::SecondMolar => "2nd Molar",
            Self::ThirdMolar => "3rd Molar",
        };
        write!(f, "{s}")
    }
}

const QUADRANTS: [Quadrant; 4] = [
    Quadrant::UpperRight,
    Quadrant::UpperLeft,
    Quadrant::LowerLeft,
    Quadrant::LowerRight,
];

const TOOTH_TYPES: [ToothType; 8] = [
    ToothType::CentralIncisor,
    ToothType::LateralIncisor,
    ToothType::Canine,
    ToothType::FirstPremolar,
    ToothType::SecondPremolar,
    ToothType::FirstMolar,
    ToothType::SecondMolar,
    ToothType::ThirdMolar,
];

// ---------------------------------------------------------------------------
// ToothId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToothId {
    pub fdi: u8,
    pub universal: u8,
    pub quadrant: Quadrant,
    pub tooth_type: ToothType,
    pub is_primary: bool,
}

impl ToothId {
    /// Create from FDI number (e.g. 11 = upper-right central incisor).
    pub fn from_fdi(fdi: u8) -> Option<Self> {
        let q = fdi / 10;
        let t = fdi % 10;
        if !(1..=4).contains(&q) || !(1..=8).contains(&t) {
            return None;
        }
        let quadrant = QUADRANTS[(q - 1) as usize];
        let tooth_type = TOOTH_TYPES[(t - 1) as usize];
        let universal = fdi_to_universal(fdi)?;
        Some(Self { fdi, universal, quadrant, tooth_type, is_primary: false })
    }

    /// Create from Universal (ADA) number (1-32).
    pub fn from_universal(uni: u8) -> Option<Self> {
        Self::from_fdi(universal_to_fdi(uni)?)
    }

    /// Create from Palmer notation (quadrant 1-4, tooth 1-8).
    pub fn from_palmer(quadrant: u8, tooth: u8) -> Option<Self> {
        if !(1..=4).contains(&quadrant) || !(1..=8).contains(&tooth) {
            return None;
        }
        Self::from_fdi(quadrant * 10 + tooth)
    }

    /// Return Palmer representation as (quadrant_symbol, tooth_number).
    pub fn to_palmer(&self) -> (char, u8) {
        (self.quadrant.palmer_symbol(), self.tooth_type.number())
    }

    /// Format this tooth in the given notation system.
    pub fn format_as(&self, system: NotationSystem) -> String {
        match system {
            NotationSystem::Fdi => format!("{}", self.fdi),
            NotationSystem::Universal => format!("{}", self.universal),
            NotationSystem::Palmer => {
                let (sym, n) = self.to_palmer();
                format!("{n}{sym}")
            }
        }
    }
}

impl fmt::Display for ToothId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FDI {} ({} {})", self.fdi, self.quadrant, self.tooth_type)
    }
}

/// Parse from FDI string (e.g. "11"), Universal prefixed ("U1"), or Palmer ("3┘").
impl FromStr for ToothId {
    type Err = NotationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        // Try "U<number>" for universal
        if let Some(rest) = s.strip_prefix('U').or_else(|| s.strip_prefix('u')) {
            let n: u8 = rest.parse().map_err(|_| NotationParseError::InvalidFormat)?;
            return Self::from_universal(n).ok_or(NotationParseError::OutOfRange);
        }
        // Try Palmer: <digit><symbol>
        if s.len() >= 4 {
            // Palmer symbols are multi-byte UTF-8
            let chars: Vec<char> = s.chars().collect();
            if let Some(last) = chars.last() {
                if let Some(q) = Quadrant::from_palmer_symbol(*last) {
                    let num_str: String = chars[..chars.len() - 1].iter().collect();
                    if let Ok(t) = num_str.parse::<u8>() {
                        return Self::from_palmer(q.fdi_number(), t)
                            .ok_or(NotationParseError::OutOfRange);
                    }
                }
            }
        }
        // Default: FDI number
        let n: u8 = s.parse().map_err(|_| NotationParseError::InvalidFormat)?;
        Self::from_fdi(n).ok_or(NotationParseError::OutOfRange)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotationParseError {
    InvalidFormat,
    OutOfRange,
}

impl fmt::Display for NotationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid notation format"),
            Self::OutOfRange => write!(f, "tooth number out of range"),
        }
    }
}

impl std::error::Error for NotationParseError {}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

pub fn fdi_to_universal(fdi: u8) -> Option<u8> {
    let (q, t) = (fdi / 10, fdi % 10);
    if !(1..=8).contains(&t) {
        return None;
    }
    // Q1(UR): FDI 11→U8, 18→U1  | Q2(UL): FDI 21→U9, 28→U16
    // Q3(LL): FDI 31→U24, 38→U17 | Q4(LR): FDI 41→U25, 48→U32
    match q {
        1 => Some(9 - t),
        2 => Some(8 + t),
        3 => Some(25 - t),
        4 => Some(24 + t),
        _ => None,
    }
}

pub fn universal_to_fdi(uni: u8) -> Option<u8> {
    match uni {
        1..=8 => Some(10 + (9 - uni)),   // Q1: U1→FDI18, U8→FDI11
        9..=16 => Some(20 + (uni - 8)),   // Q2: U9→FDI21, U16→FDI28
        17..=24 => Some(30 + (25 - uni)), // Q3: U17→FDI38, U24→FDI31
        25..=32 => Some(40 + (uni - 24)), // Q4: U25→FDI41, U32→FDI48
        _ => None,
    }
}

/// Convert between any two notation systems.
pub fn convert(from: NotationSystem, to: NotationSystem, value: &str) -> Option<String> {
    let tooth = match from {
        NotationSystem::Fdi => {
            let n: u8 = value.parse().ok()?;
            ToothId::from_fdi(n)?
        }
        NotationSystem::Universal => {
            let n: u8 = value.parse().ok()?;
            ToothId::from_universal(n)?
        }
        NotationSystem::Palmer => {
            let chars: Vec<char> = value.chars().collect();
            let last = *chars.last()?;
            let q = Quadrant::from_palmer_symbol(last)?;
            let num_str: String = chars[..chars.len() - 1].iter().collect();
            let t: u8 = num_str.parse().ok()?;
            ToothId::from_palmer(q.fdi_number(), t)?
        }
    };
    Some(tooth.format_as(to))
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

pub fn is_valid_fdi(n: u8) -> bool {
    let q = n / 10;
    let t = n % 10;
    (1..=4).contains(&q) && (1..=8).contains(&t)
}

pub fn is_valid_universal(n: u8) -> bool {
    (1..=32).contains(&n)
}

pub fn is_valid_palmer(quadrant: u8, tooth: u8) -> bool {
    (1..=4).contains(&quadrant) && (1..=8).contains(&tooth)
}

// ---------------------------------------------------------------------------
// Collection helpers
// ---------------------------------------------------------------------------

/// All 32 permanent adult teeth.
pub fn adult_dentition() -> Vec<ToothId> {
    (1..=4u8)
        .flat_map(|q| (1..=8u8).filter_map(move |t| ToothId::from_fdi(q * 10 + t)))
        .collect()
}

/// Teeth in a specific quadrant.
pub fn teeth_by_quadrant(quadrant: Quadrant) -> Vec<ToothId> {
    let q = quadrant.fdi_number();
    (1..=8u8).filter_map(move |t| ToothId::from_fdi(q * 10 + t)).collect()
}

/// FDI → Universal lookup table.
pub fn fdi_universal_table() -> HashMap<u8, u8> {
    let mut m = HashMap::new();
    for q in 1..=4u8 {
        for t in 1..=8u8 {
            let f = q * 10 + t;
            if let Some(u) = fdi_to_universal(f) {
                m.insert(f, u);
            }
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fdi_basic() {
        let t = ToothId::from_fdi(11).unwrap();
        assert_eq!(t.quadrant, Quadrant::UpperRight);
        assert_eq!(t.tooth_type, ToothType::CentralIncisor);
        assert_eq!(t.universal, 8);
    }

    #[test]
    fn count_32() {
        assert_eq!(adult_dentition().len(), 32);
    }

    #[test]
    fn fdi_invalid() {
        assert!(ToothId::from_fdi(0).is_none());
        assert!(ToothId::from_fdi(50).is_none());
        assert!(ToothId::from_fdi(19).is_none());
        assert!(ToothId::from_fdi(10).is_none());
    }

    #[test]
    fn universal_roundtrip() {
        for uni in 1..=32u8 {
            let tooth = ToothId::from_universal(uni).unwrap();
            assert_eq!(tooth.universal, uni);
            let rt = fdi_to_universal(tooth.fdi).unwrap();
            assert_eq!(rt, uni);
        }
    }

    #[test]
    fn palmer_roundtrip() {
        for q in 1..=4u8 {
            for t in 1..=8u8 {
                let tooth = ToothId::from_palmer(q, t).unwrap();
                let (sym, num) = tooth.to_palmer();
                let q_back = Quadrant::from_palmer_symbol(sym).unwrap();
                assert_eq!(q_back.fdi_number(), q);
                assert_eq!(num, t);
            }
        }
    }

    #[test]
    fn display_format() {
        let t = ToothId::from_fdi(11).unwrap();
        assert_eq!(t.format_as(NotationSystem::Fdi), "11");
        assert_eq!(t.format_as(NotationSystem::Universal), "8");
        assert_eq!(t.format_as(NotationSystem::Palmer), "1┘");
    }

    #[test]
    fn from_str_fdi() {
        let t: ToothId = "11".parse().unwrap();
        assert_eq!(t.fdi, 11);
    }

    #[test]
    fn from_str_universal() {
        let t: ToothId = "U8".parse().unwrap();
        assert_eq!(t.fdi, 11);
    }

    #[test]
    fn from_str_palmer() {
        let t: ToothId = "1┘".parse().unwrap();
        assert_eq!(t.fdi, 11);
    }

    #[test]
    fn from_str_invalid() {
        assert!("abc".parse::<ToothId>().is_err());
        assert!("U0".parse::<ToothId>().is_err());
        assert!("U33".parse::<ToothId>().is_err());
    }

    #[test]
    fn convert_fdi_to_universal() {
        assert_eq!(convert(NotationSystem::Fdi, NotationSystem::Universal, "11"), Some("8".to_string()));
    }

    #[test]
    fn convert_palmer_to_fdi() {
        assert_eq!(convert(NotationSystem::Palmer, NotationSystem::Fdi, "1┘"), Some("11".to_string()));
    }

    #[test]
    fn validation() {
        assert!(is_valid_fdi(11));
        assert!(is_valid_fdi(48));
        assert!(!is_valid_fdi(0));
        assert!(!is_valid_fdi(50));
        assert!(is_valid_universal(1));
        assert!(is_valid_universal(32));
        assert!(!is_valid_universal(0));
        assert!(!is_valid_universal(33));
        assert!(is_valid_palmer(1, 1));
        assert!(!is_valid_palmer(5, 1));
        assert!(!is_valid_palmer(1, 9));
    }

    #[test]
    fn teeth_by_quadrant_count() {
        for q in &QUADRANTS {
            assert_eq!(teeth_by_quadrant(*q).len(), 8);
        }
    }
}

// ---------------------------------------------------------------------------
// S104: Primary (deciduous) dentition
// ---------------------------------------------------------------------------

/// Deciduous tooth type (A-E per quadrant in FDI 5x-8x system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeciduousToothType {
    CentralIncisor,  // 1 / A
    LateralIncisor,  // 2 / B
    Canine,          // 3 / C
    FirstMolar,      // 4 / D
    SecondMolar,     // 5 / E
}

impl DeciduousToothType {
    /// Number within quadrant (1-5).
    pub fn number(self) -> u8 {
        match self {
            Self::CentralIncisor => 1,
            Self::LateralIncisor => 2,
            Self::Canine => 3,
            Self::FirstMolar => 4,
            Self::SecondMolar => 5,
        }
    }

    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::CentralIncisor),
            2 => Some(Self::LateralIncisor),
            3 => Some(Self::Canine),
            4 => Some(Self::FirstMolar),
            5 => Some(Self::SecondMolar),
            _ => None,
        }
    }

    /// ADA letter (A-E).
    pub fn letter(self) -> char {
        (b'A' + self.number() - 1) as char
    }
}

/// Deciduous quadrant numbers in FDI: 5=UR, 6=UL, 7=LL, 8=LR.
pub fn deciduous_quadrant_fdi(q: Quadrant) -> u8 {
    match q {
        Quadrant::UpperRight => 5,
        Quadrant::UpperLeft => 6,
        Quadrant::LowerLeft => 7,
        Quadrant::LowerRight => 8,
    }
}

/// Compute FDI number for a deciduous tooth.
pub fn deciduous_fdi(quadrant: Quadrant, tooth: DeciduousToothType) -> u8 {
    deciduous_quadrant_fdi(quadrant) * 10 + tooth.number()
}

/// Validate a deciduous FDI number (51-55, 61-65, 71-75, 81-85).
pub fn is_valid_deciduous_fdi(fdi: u8) -> bool {
    let q = fdi / 10;
    let t = fdi % 10;
    (5..=8).contains(&q) && (1..=5).contains(&t)
}

/// Convert deciduous FDI to Universal deciduous numbering (ADA letters A-T).
/// UR: 51→A..55→E, UL: 61→F..65→J, LL: 71→K..75→O, LR: 81→P..85→T
pub fn deciduous_fdi_to_universal_letter(fdi: u8) -> Option<char> {
    if !is_valid_deciduous_fdi(fdi) {
        return None;
    }
    let q = fdi / 10;
    let t = fdi % 10;
    let offset = match q {
        5 => 0,      // A-E
        6 => 5,      // F-J
        7 => 10,     // K-O
        8 => 15,     // P-T
        _ => return None,
    };
    Some((b'A' + offset + t - 1) as char)
}

/// Convert Universal deciduous letter (A-T) back to FDI.
pub fn deciduous_universal_letter_to_fdi(letter: char) -> Option<u8> {
    let upper = letter.to_ascii_uppercase();
    if !('A'..='T').contains(&upper) {
        return None;
    }
    let idx = (upper as u8) - b'A'; // 0-19
    let (q, t) = match idx {
        0..=4 => (5, idx + 1),
        5..=9 => (6, idx - 4),
        10..=14 => (7, idx - 9),
        15..=19 => (8, idx - 14),
        _ => return None,
    };
    Some(q * 10 + t)
}

/// Check if an FDI number is deciduous.
pub fn is_deciduous(fdi: u8) -> bool {
    is_valid_deciduous_fdi(fdi)
}

/// Check if an FDI number is permanent.
pub fn is_permanent(fdi: u8) -> bool {
    is_valid_fdi(fdi) && !is_deciduous(fdi)
}

#[cfg(test)]
mod deciduous_tests {
    use super::*;

    #[test]
    fn deciduous_fdi_values() {
        assert_eq!(deciduous_fdi(Quadrant::UpperRight, DeciduousToothType::CentralIncisor), 51);
        assert_eq!(deciduous_fdi(Quadrant::LowerLeft, DeciduousToothType::SecondMolar), 75);
    }

    #[test]
    fn deciduous_fdi_validation() {
        assert!(is_valid_deciduous_fdi(51));
        assert!(is_valid_deciduous_fdi(85));
        assert!(!is_valid_deciduous_fdi(50));
        assert!(!is_valid_deciduous_fdi(86));
        assert!(!is_valid_deciduous_fdi(11));
    }

    #[test]
    fn deciduous_universal_roundtrip() {
        for fdi in [51, 55, 61, 65, 71, 75, 81, 85] {
            let letter = deciduous_fdi_to_universal_letter(fdi).unwrap();
            let back = deciduous_universal_letter_to_fdi(letter).unwrap();
            assert_eq!(fdi, back, "roundtrip failed for FDI {fdi}");
        }
    }

    #[test]
    fn deciduous_letter_range() {
        assert_eq!(deciduous_fdi_to_universal_letter(51), Some('A'));
        assert_eq!(deciduous_fdi_to_universal_letter(85), Some('T'));
    }

    #[test]
    fn permanent_vs_deciduous() {
        assert!(is_permanent(11));
        assert!(!is_deciduous(11));
        assert!(is_deciduous(51));
        assert!(!is_permanent(51));
    }
}

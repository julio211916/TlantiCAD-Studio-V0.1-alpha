//! Universal Numbering System (ADA)
//! Upper right to left: 1-16, Lower left to right: 17-32
//! Primary: A-T

use serde::{Deserialize, Serialize};

/// Universal tooth number (1-32 for permanent, or primary letter A-T)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UniversalTooth {
    Permanent(u8),
    Primary(char),
}

impl UniversalTooth {
    pub fn permanent(n: u8) -> Option<Self> {
        if (1..=32).contains(&n) { Some(UniversalTooth::Permanent(n)) } else { None }
    }

    pub fn primary(c: char) -> Option<Self> {
        if ('A'..='T').contains(&c) { Some(UniversalTooth::Primary(c)) } else { None }
    }

    pub fn is_upper(&self) -> bool {
        match self {
            UniversalTooth::Permanent(n) => (1..=16).contains(n),
            UniversalTooth::Primary(c)   => "ABCDEFGHIJ".contains(*c),
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            UniversalTooth::Permanent(n) => (1..=8).contains(n) || (25..=32).contains(n),
            UniversalTooth::Primary(c)   => "ABCDE".contains(*c) || "PQRST".contains(*c),
        }
    }
}

impl std::fmt::Display for UniversalTooth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniversalTooth::Permanent(n) => write!(f, "#{}", n),
            UniversalTooth::Primary(c)   => write!(f, "#{}", c),
        }
    }
}

/// Get all 32 universal permanent tooth numbers
pub fn all_permanent_universal() -> Vec<u8> {
    (1..=32).collect()
}

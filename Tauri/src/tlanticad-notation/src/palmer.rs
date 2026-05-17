//! Palmer Notation System
//! Uses quadrant symbols and position number: UR=⌐, UL=¬, LL=L, LR=_|

use serde::{Deserialize, Serialize};

/// Palmer quadrant symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PalmerQuadrant {
    UpperRight,
    UpperLeft,
    LowerLeft,
    LowerRight,
}

impl PalmerQuadrant {
    pub fn symbol(&self) -> &'static str {
        match self {
            PalmerQuadrant::UpperRight => "⌐",
            PalmerQuadrant::UpperLeft  => "¬",
            PalmerQuadrant::LowerLeft  => "L",
            PalmerQuadrant::LowerRight => "_|",
        }
    }

    pub fn ascii_symbol(&self) -> &'static str {
        match self {
            PalmerQuadrant::UpperRight => "UR",
            PalmerQuadrant::UpperLeft  => "UL",
            PalmerQuadrant::LowerLeft  => "LL",
            PalmerQuadrant::LowerRight => "LR",
        }
    }
}

/// Palmer tooth designation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PalmerTooth {
    pub quadrant: PalmerQuadrant,
    pub position: u8,
    pub is_primary: bool,
}

impl PalmerTooth {
    pub fn new(quadrant: PalmerQuadrant, position: u8) -> Option<Self> {
        if (1..=8).contains(&position) {
            Some(Self { quadrant, position, is_primary: false })
        } else {
            None
        }
    }

    pub fn primary(quadrant: PalmerQuadrant, position: u8) -> Option<Self> {
        if (1..=5).contains(&position) {
            Some(Self { quadrant, position, is_primary: true })
        } else {
            None
        }
    }
}

impl std::fmt::Display for PalmerTooth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_primary {
            write!(f, "{}{}", self.quadrant.ascii_symbol(), (b'A' + self.position - 1) as char)
        } else {
            write!(f, "{}{}", self.quadrant.ascii_symbol(), self.position)
        }
    }
}

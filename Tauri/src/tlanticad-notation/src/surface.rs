//! Tooth surface notation

use serde::{Deserialize, Serialize};

/// Standard tooth surfaces using international codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToothSurface {
    /// Mesial (M) — toward midline
    Mesial,
    /// Distal (D) — away from midline
    Distal,
    /// Buccal/Facial (B/F) — toward cheek/lip
    Buccal,
    /// Lingual/Palatal (L/P) — toward tongue/palate
    Lingual,
    /// Occlusal (O) — chewing surface (posterior)
    Occlusal,
    /// Incisal (I) — cutting edge (anterior)
    Incisal,
    /// Cervical (C) — near gum line
    Cervical,
    /// Apical (A) — near root tip
    Apical,
}

impl ToothSurface {
    /// Single-letter code
    pub fn code(&self) -> &'static str {
        match self {
            ToothSurface::Mesial   => "M",
            ToothSurface::Distal   => "D",
            ToothSurface::Buccal   => "B",
            ToothSurface::Lingual  => "L",
            ToothSurface::Occlusal => "O",
            ToothSurface::Incisal  => "I",
            ToothSurface::Cervical => "C",
            ToothSurface::Apical   => "A",
        }
    }

    /// Full name in Spanish (used in LatAm dental practice)
    pub fn nombre_es(&self) -> &'static str {
        match self {
            ToothSurface::Mesial   => "Mesial",
            ToothSurface::Distal   => "Distal",
            ToothSurface::Buccal   => "Bucal",
            ToothSurface::Lingual  => "Lingual/Palatino",
            ToothSurface::Occlusal => "Oclusal",
            ToothSurface::Incisal  => "Incisal",
            ToothSurface::Cervical => "Cervical",
            ToothSurface::Apical   => "Apical",
        }
    }

    pub fn from_code(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'M' => Some(ToothSurface::Mesial),
            'D' => Some(ToothSurface::Distal),
            'B' | 'F' => Some(ToothSurface::Buccal),
            'L' | 'P' => Some(ToothSurface::Lingual),
            'O' => Some(ToothSurface::Occlusal),
            'I' => Some(ToothSurface::Incisal),
            'C' => Some(ToothSurface::Cervical),
            'A' => Some(ToothSurface::Apical),
            _ => None,
        }
    }
}

/// Parse multi-surface notation string like "MOD" → surfaces
pub fn parse_surface_string(s: &str) -> Vec<ToothSurface> {
    s.chars().filter_map(ToothSurface::from_code).collect()
}

/// Format surfaces to combined code like "MOD"
pub fn format_surfaces(surfaces: &[ToothSurface]) -> String {
    surfaces.iter().map(|s| s.code()).collect()
}

/// Black's cavity classification based on surfaces
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlacksClass {
    /// Class I: Occlusal pits/fissures
    ClassI,
    /// Class II: Proximal surfaces of posterior teeth
    ClassII,
    /// Class III: Proximal surfaces of anterior teeth (no incisal)
    ClassIII,
    /// Class IV: Proximal + incisal of anterior teeth
    ClassIV,
    /// Class V: Cervical/gingival third of any tooth
    ClassV,
    /// Class VI: Incisal edge/cusp tip
    ClassVI,
}

pub fn classify_cavity(surfaces: &[ToothSurface], is_anterior: bool) -> Option<BlacksClass> {
    let has_occlusal = surfaces.contains(&ToothSurface::Occlusal);
    let has_proximal = surfaces.contains(&ToothSurface::Mesial) || surfaces.contains(&ToothSurface::Distal);
    let has_incisal  = surfaces.contains(&ToothSurface::Incisal);
    let has_cervical = surfaces.contains(&ToothSurface::Cervical);

    if has_cervical && !has_occlusal && !has_proximal { return Some(BlacksClass::ClassV); }
    if has_incisal && has_proximal && is_anterior { return Some(BlacksClass::ClassIV); }
    if has_proximal && is_anterior { return Some(BlacksClass::ClassIII); }
    if has_proximal && !is_anterior { return Some(BlacksClass::ClassII); }
    if has_occlusal { return Some(BlacksClass::ClassI); }
    None
}

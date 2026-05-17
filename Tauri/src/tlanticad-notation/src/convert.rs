//! Convert between FDI, Universal, and Palmer notation systems

use crate::palmer::{PalmerQuadrant, PalmerTooth};

/// Convert FDI permanent tooth number to Universal number
pub fn fdi_to_universal(fdi: u8) -> Option<u8> {
    let map: &[(u8, u8)] = &[
        // Upper right (Q1 → Universal 1-8 right to left)
        (18,1),(17,2),(16,3),(15,4),(14,5),(13,6),(12,7),(11,8),
        // Upper left (Q2 → Universal 9-16 left to right)
        (21,9),(22,10),(23,11),(24,12),(25,13),(26,14),(27,15),(28,16),
        // Lower left (Q3 → Universal 17-24)
        (38,17),(37,18),(36,19),(35,20),(34,21),(33,22),(32,23),(31,24),
        // Lower right (Q4 → Universal 25-32)
        (41,25),(42,26),(43,27),(44,28),(45,29),(46,30),(47,31),(48,32),
    ];
    map.iter().find(|&&(f, _)| f == fdi).map(|&(_, u)| u)
}

/// Convert Universal permanent tooth to FDI number
pub fn universal_to_fdi(universal: u8) -> Option<u8> {
    let map: &[(u8, u8)] = &[
        (1,18),(2,17),(3,16),(4,15),(5,14),(6,13),(7,12),(8,11),
        (9,21),(10,22),(11,23),(12,24),(13,25),(14,26),(15,27),(16,28),
        (17,38),(18,37),(19,36),(20,35),(21,34),(22,33),(23,32),(24,31),
        (25,41),(26,42),(27,43),(28,44),(29,45),(30,46),(31,47),(32,48),
    ];
    map.iter().find(|&&(u, _)| u == universal).map(|&(_, f)| f)
}

/// Convert FDI to Palmer tooth
pub fn fdi_to_palmer(fdi: u8) -> Option<PalmerTooth> {
    let q = fdi / 10;
    let p = fdi % 10;
    let quadrant = match q {
        1 => PalmerQuadrant::UpperRight,
        2 => PalmerQuadrant::UpperLeft,
        3 => PalmerQuadrant::LowerLeft,
        4 => PalmerQuadrant::LowerRight,
        _ => return None,
    };
    PalmerTooth::new(quadrant, p)
}

/// Convert Palmer to FDI
pub fn palmer_to_fdi(palmer: &PalmerTooth) -> Option<u8> {
    let q: u8 = match palmer.quadrant {
        PalmerQuadrant::UpperRight => 1,
        PalmerQuadrant::UpperLeft  => 2,
        PalmerQuadrant::LowerLeft  => 3,
        PalmerQuadrant::LowerRight => 4,
    };
    let fdi = q * 10 + palmer.position;
    if crate::fdi::is_valid_fdi(fdi) { Some(fdi) } else { None }
}

/// Convert a set of FDI teeth to their Universal equivalents
pub fn batch_fdi_to_universal(teeth: &[u8]) -> Vec<Option<u8>> {
    teeth.iter().map(|&t| fdi_to_universal(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdi_to_universal_roundtrip() {
        for fdi in crate::fdi::all_permanent_fdi() {
            if let Some(uni) = fdi_to_universal(fdi) {
                let back = universal_to_fdi(uni);
                assert_eq!(back, Some(fdi), "Roundtrip failed for FDI {}", fdi);
            }
        }
    }
}

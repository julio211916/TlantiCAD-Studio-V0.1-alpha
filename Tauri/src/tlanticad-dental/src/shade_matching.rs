//! S286-S290: Color & Shade Matching
//!
//! VITA shade system, LAB color space mapping,
//! translucency, fluorescence, and shade recommendation.

use serde::{Deserialize, Serialize};

/// VITA Classical shade designation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VitaClassical {
    A1, A2, A3, A3_5, A4,
    B1, B2, B3, B4,
    C1, C2, C3, C4,
    D2, D3, D4,
}

impl VitaClassical {
    /// Get the approximate CIE-Lab values (L*, a*, b*)
    pub fn to_lab(&self) -> [f64; 3] {
        match self {
            Self::A1    => [75.0, 0.5, 15.0],
            Self::A2    => [73.0, 1.0, 18.0],
            Self::A3    => [70.0, 2.0, 22.0],
            Self::A3_5  => [68.0, 2.5, 25.0],
            Self::A4    => [65.0, 3.0, 28.0],
            Self::B1    => [76.0, -1.0, 12.0],
            Self::B2    => [73.0, -0.5, 16.0],
            Self::B3    => [70.0, 0.0, 20.0],
            Self::B4    => [67.0, 0.5, 24.0],
            Self::C1    => [72.0, -1.0, 10.0],
            Self::C2    => [69.0, -0.5, 14.0],
            Self::C3    => [66.0, 0.0, 18.0],
            Self::C4    => [63.0, 0.5, 22.0],
            Self::D2    => [74.0, -2.0, 10.0],
            Self::D3    => [71.0, -1.5, 14.0],
            Self::D4    => [68.0, -1.0, 18.0],
        }
    }

    /// List all shades
    pub fn all() -> &'static [VitaClassical] {
        &[
            Self::A1, Self::A2, Self::A3, Self::A3_5, Self::A4,
            Self::B1, Self::B2, Self::B3, Self::B4,
            Self::C1, Self::C2, Self::C3, Self::C4,
            Self::D2, Self::D3, Self::D4,
        ]
    }
}

/// VITA 3D-Master shade (value, chroma, hue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vita3DMaster {
    pub value: u8,     // 1-5
    pub chroma: u8,    // 1-3 (M1, M2, M3)
    pub hue: ShadeHue, // L, M, R
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShadeHue { L, M, R }

/// CIE-Lab color value
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LabColor {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl LabColor {
    pub fn new(l: f64, a: f64, b: f64) -> Self { Self { l, a, b } }

    /// Delta E (CIE76) color distance
    pub fn delta_e(&self, other: &LabColor) -> f64 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }
}

/// Translucency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TranslucencyLevel {
    High,       // HT (anterior enamel)
    Medium,     // MT (general)
    Low,        // LT (posterior, opaque)
    SuperHigh,  // ST (very thin veneers)
}

/// Shade mapping for a restoration zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadeZone {
    pub zone_name: String,
    pub shade: VitaClassical,
    pub lab: LabColor,
    pub translucency: TranslucencyLevel,
    pub fluorescence: bool,
}

/// Shade analysis result with recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadeAnalysis {
    pub measured_lab: LabColor,
    pub recommended_shade: VitaClassical,
    pub delta_e: f64,
    pub zones: Vec<ShadeZone>,
    pub match_quality: MatchQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchQuality {
    Excellent,  // ΔE < 1.0
    Good,       // ΔE < 2.0
    Acceptable, // ΔE < 3.5
    Poor,       // ΔE >= 3.5
}

/// Find the closest VITA shade to a measured LAB value
pub fn recommend_shade(measured: LabColor) -> (VitaClassical, f64) {
    let mut best_shade = VitaClassical::A2;
    let mut best_de = f64::MAX;

    for shade in VitaClassical::all() {
        let lab = shade.to_lab();
        let ref_color = LabColor::new(lab[0], lab[1], lab[2]);
        let de = measured.delta_e(&ref_color);
        if de < best_de {
            best_de = de;
            best_shade = *shade;
        }
    }
    (best_shade, best_de)
}

/// Classify match quality from delta E
pub fn match_quality(delta_e: f64) -> MatchQuality {
    if delta_e < 1.0 { MatchQuality::Excellent }
    else if delta_e < 2.0 { MatchQuality::Good }
    else if delta_e < 3.5 { MatchQuality::Acceptable }
    else { MatchQuality::Poor }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vita_lab_roundtrip() {
        let lab = VitaClassical::A2.to_lab();
        let color = LabColor::new(lab[0], lab[1], lab[2]);
        let (shade, de) = recommend_shade(color);
        assert_eq!(shade, VitaClassical::A2);
        assert!(de < 0.001);
    }

    #[test]
    fn test_delta_e() {
        let c1 = LabColor::new(75.0, 0.0, 15.0);
        let c2 = LabColor::new(73.0, 1.0, 18.0);
        let de = c1.delta_e(&c2);
        assert!(de > 0.0);
        assert!(de < 5.0);
    }

    #[test]
    fn test_match_quality_excellent() {
        assert_eq!(match_quality(0.5), MatchQuality::Excellent);
    }

    #[test]
    fn test_match_quality_poor() {
        assert_eq!(match_quality(5.0), MatchQuality::Poor);
    }

    #[test]
    fn test_shade_recommendation() {
        let measured = LabColor::new(74.0, 0.8, 16.0);
        let (shade, de) = recommend_shade(measured);
        // Should be close to A1 or A2
        assert!(de < 5.0);
        assert!(shade == VitaClassical::A1 || shade == VitaClassical::A2 || shade == VitaClassical::B1);
    }
}

// ── S271-S274 Iteration: Extended shade matching features ──

/// Vita 3D-Master shade preset (lightness + chroma group)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Vita3DPreset {
    L1_5M1, L2M1, L2_5M1,
    L1_5M2, L2M2, L2_5M2, L3M2, L3_5M2,
    L2_5M3, L3M3, L3_5M3, L4M3,
}

impl Vita3DPreset {
    pub fn lightness_group(&self) -> u8 {
        match self {
            Self::L1_5M1 | Self::L1_5M2 => 1,
            Self::L2M1 | Self::L2M2 => 2,
            Self::L2_5M1 | Self::L2_5M2 | Self::L2_5M3 => 2,
            Self::L3M2 | Self::L3M3 => 3,
            Self::L3_5M2 | Self::L3_5M3 => 3,
            Self::L4M3 => 4,
        }
    }
}

/// Shade zone map — records shade per tooth region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadeZoneMap {
    pub tooth_id: String,
    pub cervical: VitaClassical,
    pub body: VitaClassical,
    pub incisal: VitaClassical,
    pub translucency: f64,
}

impl ShadeZoneMap {
    pub fn new(tooth: impl Into<String>, cervical: VitaClassical, body: VitaClassical, incisal: VitaClassical) -> Self {
        Self { tooth_id: tooth.into(), cervical, body, incisal, translucency: 0.5 }
    }

    pub fn is_homogeneous(&self) -> bool {
        self.cervical == self.body && self.body == self.incisal
    }
}

/// Spectrophotometer reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectroReading {
    pub device: String,
    pub tooth_id: String,
    pub lab_color: LabColor,
    pub recommended_shade: VitaClassical,
    pub confidence: f64,
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_vita_3d_lightness() {
        assert_eq!(Vita3DPreset::L1_5M1.lightness_group(), 1);
        assert_eq!(Vita3DPreset::L4M3.lightness_group(), 4);
    }

    #[test]
    fn test_zone_map_homogeneous() {
        let zm = ShadeZoneMap::new("11", VitaClassical::A2, VitaClassical::A2, VitaClassical::A2);
        assert!(zm.is_homogeneous());
    }

    #[test]
    fn test_zone_map_heterogeneous() {
        let zm = ShadeZoneMap::new("21", VitaClassical::A3, VitaClassical::A2, VitaClassical::B1);
        assert!(!zm.is_homogeneous());
    }

    #[test]
    fn test_spectro_reading() {
        let reading = SpectroReading {
            device: "VITA Easyshade V".into(),
            tooth_id: "11".into(),
            lab_color: LabColor::new(74.0, 0.5, 16.0),
            recommended_shade: VitaClassical::A2,
            confidence: 0.92,
        };
        assert!(reading.confidence > 0.9);
    }
}

//! S334-S338: Dental Material Database
//!
//! Material properties, compatibility, and selection for dental manufacturing.

use serde::{Deserialize, Serialize};

/// Material category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialCategory {
    Ceramic,
    Metal,
    Polymer,
    Composite,
    Wax,
}

/// Dental material with properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DentalMaterial {
    pub name: String,
    pub category: MaterialCategory,
    pub flexural_strength_mpa: f64,
    pub fracture_toughness_mpa_m: f64,
    pub elastic_modulus_gpa: f64,
    pub hardness_vickers: f64,
    pub translucency_pct: f64,
    pub biocompatible: bool,
    pub indications: Vec<Indication>,
    pub min_thickness_mm: f64,
    pub max_span_units: u8,
}

/// Clinical indication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Indication {
    SingleCrown,
    ThreeUnitBridge,
    LongSpanBridge,
    Inlay,
    Onlay,
    Veneer,
    ImplantAbutment,
    FullArchFramework,
    SurgicalGuide,
    DentureBase,
    OcclusalSplint,
    Temporary,
}

impl DentalMaterial {
    pub fn zirconia_ht() -> Self {
        Self {
            name: "Zirconia HT (High Translucency)".into(),
            category: MaterialCategory::Ceramic,
            flexural_strength_mpa: 1200.0,
            fracture_toughness_mpa_m: 5.0,
            elastic_modulus_gpa: 210.0,
            hardness_vickers: 1300.0,
            translucency_pct: 42.0,
            biocompatible: true,
            indications: vec![
                Indication::SingleCrown, Indication::ThreeUnitBridge,
                Indication::ImplantAbutment, Indication::FullArchFramework,
            ],
            min_thickness_mm: 0.5,
            max_span_units: 14,
        }
    }

    pub fn emax_press() -> Self {
        Self {
            name: "IPS e.max Press".into(),
            category: MaterialCategory::Ceramic,
            flexural_strength_mpa: 400.0,
            fracture_toughness_mpa_m: 2.75,
            elastic_modulus_gpa: 95.0,
            hardness_vickers: 580.0,
            translucency_pct: 70.0,
            biocompatible: true,
            indications: vec![
                Indication::SingleCrown, Indication::ThreeUnitBridge,
                Indication::Inlay, Indication::Onlay, Indication::Veneer,
            ],
            min_thickness_mm: 0.3,
            max_span_units: 3,
        }
    }

    pub fn pmma_temp() -> Self {
        Self {
            name: "PMMA Temporary".into(),
            category: MaterialCategory::Polymer,
            flexural_strength_mpa: 120.0,
            fracture_toughness_mpa_m: 1.2,
            elastic_modulus_gpa: 3.0,
            hardness_vickers: 20.0,
            translucency_pct: 55.0,
            biocompatible: true,
            indications: vec![Indication::Temporary, Indication::OcclusalSplint],
            min_thickness_mm: 0.8,
            max_span_units: 14,
        }
    }

    pub fn cocr_alloy() -> Self {
        Self {
            name: "CoCr Alloy".into(),
            category: MaterialCategory::Metal,
            flexural_strength_mpa: 800.0,
            fracture_toughness_mpa_m: 50.0,
            elastic_modulus_gpa: 230.0,
            hardness_vickers: 350.0,
            translucency_pct: 0.0,
            biocompatible: true,
            indications: vec![
                Indication::FullArchFramework, Indication::LongSpanBridge,
            ],
            min_thickness_mm: 0.3,
            max_span_units: 14,
        }
    }

    pub fn suitable_for(&self, indication: Indication) -> bool {
        self.indications.contains(&indication)
    }

    pub fn strength_index(&self) -> f64 {
        self.flexural_strength_mpa * self.fracture_toughness_mpa_m / 1000.0
    }
}

/// Material selection recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRecommendation {
    pub material: DentalMaterial,
    pub suitability_score: f64,
    pub reasons: Vec<String>,
}

/// Recommend materials for a given indication and requirements
pub fn recommend_materials(
    indication: Indication,
    span_units: u8,
    need_esthetics: bool,
) -> Vec<MaterialRecommendation> {
    let candidates = vec![
        DentalMaterial::zirconia_ht(),
        DentalMaterial::emax_press(),
        DentalMaterial::pmma_temp(),
        DentalMaterial::cocr_alloy(),
    ];

    let mut recs: Vec<MaterialRecommendation> = candidates.into_iter()
        .filter(|m| m.suitable_for(indication) && m.max_span_units >= span_units)
        .map(|m| {
            let mut score: f64 = 50.0;
            let mut reasons = Vec::new();

            if need_esthetics && m.translucency_pct > 30.0 {
                score += 20.0;
                reasons.push("Good esthetics".into());
            }
            if m.flexural_strength_mpa > 500.0 {
                score += 15.0;
                reasons.push("High strength".into());
            }
            if m.biocompatible {
                score += 10.0;
                reasons.push("Biocompatible".into());
            }

            MaterialRecommendation { material: m, suitability_score: score.min(100.0), reasons }
        })
        .collect();

    recs.sort_by(|a, b| b.suitability_score.partial_cmp(&a.suitability_score).unwrap());
    recs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zirconia_properties() {
        let z = DentalMaterial::zirconia_ht();
        assert!(z.flexural_strength_mpa > 1000.0);
        assert!(z.suitable_for(Indication::SingleCrown));
        assert!(!z.suitable_for(Indication::Veneer));
    }

    #[test]
    fn test_emax_properties() {
        let e = DentalMaterial::emax_press();
        assert!(e.translucency_pct > 50.0);
        assert!(e.suitable_for(Indication::Veneer));
    }

    #[test]
    fn test_recommend_single_crown() {
        let recs = recommend_materials(Indication::SingleCrown, 1, true);
        assert!(!recs.is_empty());
        assert!(recs[0].suitability_score > 50.0);
    }

    #[test]
    fn test_recommend_framework() {
        let recs = recommend_materials(Indication::FullArchFramework, 14, false);
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_strength_index() {
        let z = DentalMaterial::zirconia_ht();
        let e = DentalMaterial::emax_press();
        assert!(z.strength_index() > e.strength_index());
    }

    #[test]
    fn test_material_min_thickness() {
        let e = DentalMaterial::emax_press();
        assert!(e.min_thickness_mm < 0.5);
    }
}

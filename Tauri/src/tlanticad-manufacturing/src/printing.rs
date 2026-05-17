//! S311-S318: 3D Printing Support & Slicing
//!
//! SLA/DLP resin printing, support generation, slicing, and orientation optimization.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// 3D printing technology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrintTechnology {
    SLA,
    DLP,
    LCD,
    FDM,
    SLS,
    BinderJet,
    MetalSLM,
}

/// Resin/material type for printing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrintMaterial {
    SurgicalGuideResin,
    ModelResin,
    CastableResin,
    TemporaryCrownResin,
    DentureBaseResin,
    SplintResin,
    GingivalResin,
    PeekFilament,
}

impl PrintMaterial {
    pub fn layer_height_um(&self) -> f64 {
        match self {
            Self::SurgicalGuideResin => 50.0,
            Self::ModelResin => 100.0,
            Self::CastableResin => 25.0,
            Self::TemporaryCrownResin => 50.0,
            Self::DentureBaseResin => 50.0,
            Self::SplintResin => 50.0,
            Self::GingivalResin => 100.0,
            Self::PeekFilament => 200.0,
        }
    }

    pub fn cure_time_seconds(&self) -> f64 {
        match self {
            Self::SurgicalGuideResin => 6.0,
            Self::ModelResin => 8.0,
            Self::CastableResin => 4.0,
            Self::TemporaryCrownResin => 5.0,
            Self::DentureBaseResin => 7.0,
            Self::SplintResin => 6.0,
            Self::GingivalResin => 8.0,
            Self::PeekFilament => 0.0,
        }
    }

    pub fn shrinkage_pct(&self) -> f64 {
        match self {
            Self::CastableResin => 0.5,
            Self::ModelResin => 0.3,
            _ => 0.2,
        }
    }
}

/// Support structure type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportType {
    Point,
    Line,
    Tree,
    Lattice,
    Full,
}

/// Single support pillar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportPillar {
    pub base: Point3<f64>,
    pub contact: Point3<f64>,
    pub tip_diameter_mm: f64,
    pub base_diameter_mm: f64,
    pub support_type: SupportType,
}

impl SupportPillar {
    pub fn height(&self) -> f64 {
        (self.contact - self.base).norm()
    }
}

/// Support generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportStructure {
    pub pillars: Vec<SupportPillar>,
    pub total_volume_mm3: f64,
    pub total_contact_area_mm2: f64,
}

impl SupportStructure {
    pub fn pillar_count(&self) -> usize { self.pillars.len() }
}

/// Generate supports for overhang faces
pub fn generate_supports(
    overhang_points: &[Point3<f64>],
    build_plate_z: f64,
    tip_diameter: f64,
) -> SupportStructure {
    let mut pillars = Vec::new();
    let mut total_vol = 0.0;
    let mut total_area = 0.0;

    for &pt in overhang_points {
        let base = Point3::new(pt.x, pt.y, build_plate_z);
        let height = pt.z - build_plate_z;
        if height <= 0.0 { continue; }

        let base_diam = tip_diameter * 2.0;
        let vol = std::f64::consts::PI * (base_diam / 2.0).powi(2) * height * 0.33;
        let area = std::f64::consts::PI * (tip_diameter / 2.0).powi(2);

        pillars.push(SupportPillar {
            base,
            contact: pt,
            tip_diameter_mm: tip_diameter,
            base_diameter_mm: base_diam,
            support_type: SupportType::Point,
        });

        total_vol += vol;
        total_area += area;
    }

    SupportStructure { pillars, total_volume_mm3: total_vol, total_contact_area_mm2: total_area }
}

/// Orientation analysis: find optimal build orientation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationResult {
    pub rotation_x_deg: f64,
    pub rotation_y_deg: f64,
    pub rotation_z_deg: f64,
    pub support_volume_mm3: f64,
    pub print_height_mm: f64,
    pub overhang_area_mm2: f64,
    pub score: f64,
}

/// Evaluate multiple orientations and pick the best
pub fn optimize_orientation(
    bbox_dims: [f64; 3],
    candidates: &[[f64; 3]],
) -> OrientationResult {
    let mut best = OrientationResult {
        rotation_x_deg: 0.0, rotation_y_deg: 0.0, rotation_z_deg: 0.0,
        support_volume_mm3: f64::MAX, print_height_mm: bbox_dims[2],
        overhang_area_mm2: 0.0, score: 0.0,
    };

    for &[rx, ry, rz] in candidates {
        // Simplified: estimate print height based on rotation
        let height = bbox_dims[2] * rx.to_radians().cos().abs()
            + bbox_dims[0] * rx.to_radians().sin().abs();
        let support_vol = height * bbox_dims[0] * 0.1; // rough estimate
        let score = 100.0 - height * 2.0 - support_vol * 0.5;

        if score > best.score {
            best = OrientationResult {
                rotation_x_deg: rx, rotation_y_deg: ry, rotation_z_deg: rz,
                support_volume_mm3: support_vol, print_height_mm: height,
                overhang_area_mm2: support_vol * 0.3, score,
            };
        }
    }

    best
}

/// Slice model into layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceLayer {
    pub z_height_mm: f64,
    pub layer_index: usize,
    pub contour_count: usize,
    pub area_mm2: f64,
    pub is_support: bool,
}

/// Generate slice preview (layer count and heights)
pub fn compute_slicing(total_height_mm: f64, layer_height_um: f64) -> Vec<SliceLayer> {
    let lh_mm = layer_height_um / 1000.0;
    let num_layers = (total_height_mm / lh_mm).ceil() as usize;
    (0..num_layers).map(|i| {
        let z = i as f64 * lh_mm;
        SliceLayer {
            z_height_mm: z,
            layer_index: i,
            contour_count: 1,
            area_mm2: 50.0, // placeholder
            is_support: z < 2.0,
        }
    }).collect()
}

/// Print time estimation
pub fn estimate_print_time(
    total_height_mm: f64,
    material: PrintMaterial,
    technology: PrintTechnology,
) -> f64 {
    let layer_h = material.layer_height_um() / 1000.0;
    let layers = (total_height_mm / layer_h).ceil();
    let time_per_layer = match technology {
        PrintTechnology::SLA => material.cure_time_seconds() + 3.0,
        PrintTechnology::DLP | PrintTechnology::LCD => material.cure_time_seconds() + 2.0,
        PrintTechnology::FDM => 15.0,
        _ => 10.0,
    };
    (layers * time_per_layer) / 60.0 // minutes
}

/// Estimate resin usage in mL from part bounding box and fill ratio
pub fn estimate_resin_usage_ml(width: f64, depth: f64, height: f64, fill_ratio: f64) -> f64 {
    let volume_mm3 = width * depth * height * fill_ratio;
    volume_mm3 / 1000.0 // mm3 → mL
}

/// Post-print wash & cure protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashCureProtocol {
    pub material: PrintMaterial,
    pub wash_time_seconds: u32,
    pub wash_solvent: String,
    pub cure_time_seconds: u32,
    pub cure_wavelength_nm: u16,
    pub cure_temperature_c: f64,
}

pub fn wash_cure_protocol(material: PrintMaterial) -> WashCureProtocol {
    let (wash_s, cure_s, temp) = match material {
        PrintMaterial::SurgicalGuideResin => (300, 1800, 60.0),
        PrintMaterial::ModelResin => (180, 600, 40.0),
        PrintMaterial::CastableResin => (120, 900, 45.0),
        PrintMaterial::TemporaryCrownResin => (240, 1200, 55.0),
        PrintMaterial::DentureBaseResin => (300, 1800, 60.0),
        PrintMaterial::SplintResin => (240, 1500, 55.0),
        PrintMaterial::GingivalResin => (180, 600, 40.0),
        PrintMaterial::PeekFilament => (0, 0, 0.0),
    };
    WashCureProtocol {
        material,
        wash_time_seconds: wash_s,
        wash_solvent: if matches!(material, PrintMaterial::PeekFilament) { "none".into() } else { "IPA 99%".into() },
        cure_time_seconds: cure_s,
        cure_wavelength_nm: 405,
        cure_temperature_c: temp,
    }
}

/// Check if layer adhesion will be adequate
pub fn check_layer_adhesion(layer_height_um: f64, exposure_time_s: f64, _material: PrintMaterial) -> bool {
    // Basic heuristic: thicker layers need more exposure
    let min_exposure = layer_height_um / 100.0; // e.g., 50um → 0.5s min
    exposure_time_s >= min_exposure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_support_generation() {
        let overhangs = vec![
            Point3::new(5.0, 5.0, 10.0),
            Point3::new(8.0, 3.0, 12.0),
            Point3::new(2.0, 7.0, 8.0),
        ];
        let supports = generate_supports(&overhangs, 0.0, 0.4);
        assert_eq!(supports.pillar_count(), 3);
        assert!(supports.total_volume_mm3 > 0.0);
    }

    #[test]
    fn test_print_material_properties() {
        let m = PrintMaterial::SurgicalGuideResin;
        assert_eq!(m.layer_height_um(), 50.0);
        assert!(m.cure_time_seconds() > 0.0);
        assert!(m.shrinkage_pct() < 1.0);
    }

    #[test]
    fn test_orientation_optimization() {
        let candidates = vec![[0.0, 0.0, 0.0], [45.0, 0.0, 0.0], [90.0, 0.0, 0.0]];
        let result = optimize_orientation([20.0, 15.0, 10.0], &candidates);
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_slicing() {
        let layers = compute_slicing(10.0, 50.0);
        assert_eq!(layers.len(), 200);
        assert_eq!(layers[0].layer_index, 0);
    }

    #[test]
    fn test_print_time_estimation() {
        let time = estimate_print_time(10.0, PrintMaterial::ModelResin, PrintTechnology::DLP);
        assert!(time > 0.0);
    }

    #[test]
    fn test_support_pillar_height() {
        let pillar = SupportPillar {
            base: Point3::new(0.0, 0.0, 0.0),
            contact: Point3::new(0.0, 0.0, 15.0),
            tip_diameter_mm: 0.4,
            base_diameter_mm: 0.8,
            support_type: SupportType::Point,
        };
        assert!((pillar.height() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_castable_resin_shrinkage() {
        assert!(PrintMaterial::CastableResin.shrinkage_pct() > PrintMaterial::ModelResin.shrinkage_pct());
    }

    #[test]
    fn test_print_cost_estimate() {
        let cost = estimate_resin_usage_ml(5.0, 10.0, 8.0, 0.65);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_wash_cure_protocol() {
        let p = wash_cure_protocol(PrintMaterial::SurgicalGuideResin);
        assert!(p.wash_time_seconds > 0);
        assert!(p.cure_time_seconds > 0);
    }

    #[test]
    fn test_layer_adhesion_check() {
        let ok = check_layer_adhesion(50.0, 2.0, PrintMaterial::ModelResin);
        assert!(ok);
        let bad = check_layer_adhesion(200.0, 0.5, PrintMaterial::ModelResin);
        assert!(!bad);
    }
}

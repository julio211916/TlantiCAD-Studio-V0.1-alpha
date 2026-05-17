//! S304-S310: CNC Milling Strategies
//!
//! 3-axis, 4-axis, 5-axis dental milling strategies for different materials.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use crate::toolpath::Toolpath;

/// Number of axes for CNC machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MillingAxes {
    ThreeAxis,
    FourAxis,
    FiveAxis,
}

/// Milling material type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MillingMaterial {
    Zirconia,
    Emax,
    PMMA,
    Wax,
    CoCr,
    Titanium,
    Peek,
    CompositeResin,
}

impl MillingMaterial {
    /// Recommended spindle RPM
    pub fn recommended_rpm(&self) -> f64 {
        match self {
            Self::Zirconia => 18000.0,
            Self::Emax => 15000.0,
            Self::PMMA => 20000.0,
            Self::Wax => 25000.0,
            Self::CoCr => 12000.0,
            Self::Titanium => 10000.0,
            Self::Peek => 20000.0,
            Self::CompositeResin => 22000.0,
        }
    }

    /// Recommended feed rate mm/min
    pub fn recommended_feed(&self) -> f64 {
        match self {
            Self::Zirconia => 800.0,
            Self::Emax => 600.0,
            Self::PMMA => 1500.0,
            Self::Wax => 2000.0,
            Self::CoCr => 400.0,
            Self::Titanium => 300.0,
            Self::Peek => 1200.0,
            Self::CompositeResin => 1400.0,
        }
    }

    /// Shrinkage compensation factor (pre-sintered materials)
    pub fn shrinkage_factor(&self) -> f64 {
        match self {
            Self::Zirconia => 1.25, // ~20% shrinkage on sintering
            Self::Emax => 1.005,
            Self::CoCr => 1.0,
            _ => 1.0,
        }
    }

    /// Whether material needs coolant during milling
    pub fn requires_coolant(&self) -> bool {
        matches!(self, Self::CoCr | Self::Titanium | Self::Emax)
    }
}

/// Blank geometry (disc or block)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlankGeometry {
    Disc { diameter_mm: f64, height_mm: f64 },
    Block { width_mm: f64, depth_mm: f64, height_mm: f64 },
}

impl BlankGeometry {
    pub fn volume_mm3(&self) -> f64 {
        match self {
            Self::Disc { diameter_mm, height_mm } =>
                std::f64::consts::PI * (diameter_mm / 2.0).powi(2) * height_mm,
            Self::Block { width_mm, depth_mm, height_mm } =>
                width_mm * depth_mm * height_mm,
        }
    }

    pub fn fits_part(&self, part_bbox: [f64; 3]) -> bool {
        match self {
            Self::Disc { diameter_mm, height_mm } => {
                let _r = diameter_mm / 2.0;
                let diag = (part_bbox[0].powi(2) + part_bbox[1].powi(2)).sqrt();
                diag <= *diameter_mm && part_bbox[2] <= *height_mm
            }
            Self::Block { width_mm, depth_mm, height_mm } => {
                part_bbox[0] <= *width_mm && part_bbox[1] <= *depth_mm && part_bbox[2] <= *height_mm
            }
        }
    }
}

/// Milling job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MillingJob {
    pub id: String,
    pub axes: MillingAxes,
    pub material: MillingMaterial,
    pub blank: BlankGeometry,
    pub roughing_tool_mm: f64,
    pub finishing_tool_mm: f64,
    pub operations: Vec<Toolpath>,
    pub sprue_positions: Vec<Point3<f64>>,
    pub estimated_time_min: f64,
}

impl MillingJob {
    pub fn new(axes: MillingAxes, material: MillingMaterial, blank: BlankGeometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            axes,
            material,
            blank,
            roughing_tool_mm: 2.5,
            finishing_tool_mm: 1.0,
            operations: Vec::new(),
            sprue_positions: Vec::new(),
            estimated_time_min: 0.0,
        }
    }

    pub fn add_operation(&mut self, op: Toolpath) {
        self.estimated_time_min += op.estimated_time_minutes();
        self.operations.push(op);
    }

    pub fn total_operations(&self) -> usize {
        self.operations.len()
    }

    pub fn material_utilization_pct(&self, part_volume: f64) -> f64 {
        let blank_vol = self.blank.volume_mm3();
        if blank_vol > 0.0 { (part_volume / blank_vol * 100.0).min(100.0) } else { 0.0 }
    }
}

/// Auto-generate milling strategy for a dental restoration
pub fn auto_milling_strategy(
    material: MillingMaterial,
    _axes: MillingAxes,
    bbox_min: Point3<f64>,
    bbox_max: Point3<f64>,
) -> Vec<Toolpath> {
    let rpm = material.recommended_rpm();
    let feed = material.recommended_feed();
    let rough_tool = 2.5;
    let finish_tool = 1.0;

    let roughing = crate::toolpath::generate_roughing_toolpath(
        bbox_min, bbox_max, rough_tool, rough_tool * 0.5, rough_tool * 0.3, feed, rpm,
    );
    let finishing = crate::toolpath::generate_spiral_finishing(
        Point3::new(
            (bbox_min.x + bbox_max.x) / 2.0,
            (bbox_min.y + bbox_max.y) / 2.0,
            bbox_max.z,
        ),
        (bbox_max.x - bbox_min.x) / 2.0,
        bbox_max.z - bbox_min.z,
        finish_tool, feed * 0.6, rpm,
    );

    vec![roughing, finishing]
}

/// Calculate sprue positions for disc milling
pub fn calculate_sprue_positions(
    center: Point3<f64>,
    radius: f64,
    num_sprues: usize,
) -> Vec<Point3<f64>> {
    (0..num_sprues).map(|i| {
        let angle = i as f64 * std::f64::consts::TAU / num_sprues as f64;
        Point3::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
            center.z,
        )
    }).collect()
}

/// Tool wear estimation based on material and cutting distance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolWearEstimate {
    pub tool_diameter_mm: f64,
    pub total_cut_length_mm: f64,
    pub estimated_wear_um: f64,
    pub remaining_life_pct: f64,
    pub needs_replacement: bool,
}

pub fn estimate_tool_wear(
    material: MillingMaterial,
    tool_diameter_mm: f64,
    total_cut_length_mm: f64,
) -> ToolWearEstimate {
    // Wear rate depends on material hardness (um per meter of cutting)
    let wear_rate_um_per_m = match material {
        MillingMaterial::Zirconia => 2.5,
        MillingMaterial::Emax => 3.0,
        MillingMaterial::CoCr => 5.0,
        MillingMaterial::Titanium => 6.0,
        MillingMaterial::PMMA => 0.3,
        MillingMaterial::Wax => 0.1,
        MillingMaterial::Peek => 0.5,
        MillingMaterial::CompositeResin => 0.8,
    };
    let cut_m = total_cut_length_mm / 1000.0;
    let wear_um = cut_m * wear_rate_um_per_m;
    // Max allowable wear = 10% of tool diameter
    let max_wear_um = tool_diameter_mm * 1000.0 * 0.10;
    let remaining = ((max_wear_um - wear_um) / max_wear_um * 100.0).clamp(0.0, 100.0);

    ToolWearEstimate {
        tool_diameter_mm,
        total_cut_length_mm,
        estimated_wear_um: wear_um,
        remaining_life_pct: remaining,
        needs_replacement: remaining < 10.0,
    }
}

/// Coolant strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolantStrategy {
    pub coolant_type: String,
    pub flow_rate_ml_min: f64,
    pub mist_cooling: bool,
    pub flood_cooling: bool,
}

pub fn recommend_coolant(material: MillingMaterial) -> CoolantStrategy {
    match material {
        MillingMaterial::CoCr | MillingMaterial::Titanium => CoolantStrategy {
            coolant_type: "water-soluble".into(),
            flow_rate_ml_min: 500.0,
            mist_cooling: false,
            flood_cooling: true,
        },
        MillingMaterial::Emax => CoolantStrategy {
            coolant_type: "water-mist".into(),
            flow_rate_ml_min: 100.0,
            mist_cooling: true,
            flood_cooling: false,
        },
        _ => CoolantStrategy {
            coolant_type: "dry".into(),
            flow_rate_ml_min: 0.0,
            mist_cooling: false,
            flood_cooling: false,
        },
    }
}

/// Multi-unit bridge milling: partition strategy
pub fn partition_bridge_milling(
    unit_count: usize,
    total_width_mm: f64,
    disc_diameter_mm: f64,
) -> Vec<(usize, usize)> {
    // Split units into groups that fit within disc
    let unit_width = total_width_mm / unit_count as f64;
    let max_units_per_disc = (disc_diameter_mm * 0.8 / unit_width).floor() as usize;
    let max_units_per_disc = max_units_per_disc.max(1);
    let mut groups = Vec::new();
    let mut start = 0;
    while start < unit_count {
        let end = (start + max_units_per_disc).min(unit_count);
        groups.push((start, end));
        start = end;
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_properties() {
        assert!(MillingMaterial::Zirconia.recommended_rpm() > 0.0);
        assert!(MillingMaterial::Zirconia.shrinkage_factor() > 1.0);
        assert!(MillingMaterial::CoCr.requires_coolant());
        assert!(!MillingMaterial::PMMA.requires_coolant());
    }

    #[test]
    fn test_blank_geometry_disc() {
        let disc = BlankGeometry::Disc { diameter_mm: 98.0, height_mm: 20.0 };
        assert!(disc.volume_mm3() > 0.0);
        assert!(disc.fits_part([40.0, 40.0, 18.0]));
        assert!(!disc.fits_part([40.0, 40.0, 25.0]));
    }

    #[test]
    fn test_blank_geometry_block() {
        let block = BlankGeometry::Block { width_mm: 40.0, depth_mm: 20.0, height_mm: 15.0 };
        assert!(block.fits_part([35.0, 18.0, 14.0]));
        assert!(!block.fits_part([45.0, 18.0, 14.0]));
    }

    #[test]
    fn test_milling_job() {
        let job = MillingJob::new(
            MillingAxes::FiveAxis,
            MillingMaterial::Zirconia,
            BlankGeometry::Disc { diameter_mm: 98.0, height_mm: 20.0 },
        );
        assert_eq!(job.total_operations(), 0);
        assert!(job.material_utilization_pct(5000.0) > 0.0);
    }

    #[test]
    fn test_auto_milling_strategy() {
        let ops = auto_milling_strategy(
            MillingMaterial::PMMA,
            MillingAxes::ThreeAxis,
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(15.0, 12.0, 8.0),
        );
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].strategy, ToolpathStrategy::Roughing);
        assert_eq!(ops[1].strategy, ToolpathStrategy::Finishing);
    }

    #[test]
    fn test_sprue_positions() {
        let sprues = calculate_sprue_positions(Point3::origin(), 10.0, 4);
        assert_eq!(sprues.len(), 4);
        for s in &sprues {
            let dist = (s.x.powi(2) + s.y.powi(2)).sqrt();
            assert!((dist - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_material_feed_rates() {
        let mat = MillingMaterial::Titanium;
        assert!(mat.recommended_feed() < MillingMaterial::Wax.recommended_feed());
        assert!(mat.recommended_rpm() < MillingMaterial::Wax.recommended_rpm());
    }

    #[test]
    fn test_tool_wear_estimation() {
        let wear = estimate_tool_wear(MillingMaterial::Zirconia, 2.0, 50_000.0);
        assert!(wear.estimated_wear_um > 0.0);
        assert!(wear.remaining_life_pct >= 0.0 && wear.remaining_life_pct <= 100.0);
    }

    #[test]
    fn test_tool_wear_wax_minimal() {
        let wear = estimate_tool_wear(MillingMaterial::Wax, 2.0, 10_000.0);
        assert!(wear.estimated_wear_um < 5.0);
        assert!(!wear.needs_replacement);
    }

    #[test]
    fn test_coolant_recommendation() {
        let cs = recommend_coolant(MillingMaterial::CoCr);
        assert!(cs.flood_cooling);
        assert!(cs.flow_rate_ml_min > 0.0);

        let dry = recommend_coolant(MillingMaterial::PMMA);
        assert_eq!(dry.coolant_type, "dry");
    }

    #[test]
    fn test_bridge_partition() {
        let groups = partition_bridge_milling(6, 72.0, 98.0);
        assert!(!groups.is_empty());
        // All units covered
        let total: usize = groups.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total, 6);
    }
}

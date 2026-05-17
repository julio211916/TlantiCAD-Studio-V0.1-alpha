//! Default parameters - Replica defaultparameters.xml de Exocad

use serde::{Deserialize, Serialize};

/// Parámetros por defecto para diseño dental
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultParameters {
    // Crown parameters
    pub crown: CrownParameters,
    
    // Abutment parameters  
    pub abutment: AbutmentParameters,
    
    // Bridge parameters
    pub bridge: BridgeParameters,
    
    // Inlay parameters
    pub inlay: InlayParameters,
    
    // Bar parameters
    pub bar: BarParameters,
    
    // Telescope parameters
    pub telescope: TelescopeParameters,
    
    // Bite splint parameters
    pub bite_splint: BiteSplintParameters,
    
    // General parameters
    pub general: GeneralParameters,
    
    // Milling parameters
    pub milling: MillingParameters,
}

impl Default for DefaultParameters {
    fn default() -> Self {
        Self {
            crown: CrownParameters::default(),
            abutment: AbutmentParameters::default(),
            bridge: BridgeParameters::default(),
            inlay: InlayParameters::default(),
            bar: BarParameters::default(),
            telescope: TelescopeParameters::default(),
            bite_splint: BiteSplintParameters::default(),
            general: GeneralParameters::default(),
            milling: MillingParameters::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrownParameters {
    pub min_thickness: f64,
    pub cement_gap: f64,
    pub cement_gap_top: f64,
    pub extra_spacing_x: f64,
    pub extra_spacing_y: f64,
    pub extra_spacing_z: f64,
    pub margin_chamfer: f64,
    pub occlusal_reduction: f64,
}

impl Default for CrownParameters {
    fn default() -> Self {
        Self {
            min_thickness: 0.4,
            cement_gap: 0.05,
            cement_gap_top: 0.01,
            extra_spacing_x: 0.02,
            extra_spacing_y: 0.02,
            extra_spacing_z: 0.0,
            margin_chamfer: 0.2,
            occlusal_reduction: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbutmentParameters {
    pub min_thickness: f64,
    pub min_thickness_near_screw: f64,
    pub min_thickness_gingiva_top: f64,
    pub emergence_profile_height: f64,
    pub emergence_profile_cut_offset: f64,
    pub emergence_profile_max_penetration: f64,
    pub max_height: f64,
    pub shoulder_size: f64,
    pub angularity: f64,
    pub screw_channel_diameter: f64,
    pub screw_channel_angle: f64,
}

impl Default for AbutmentParameters {
    fn default() -> Self {
        Self {
            min_thickness: 0.6,
            min_thickness_near_screw: 0.2,
            min_thickness_gingiva_top: 0.2,
            emergence_profile_height: 0.2,
            emergence_profile_cut_offset: 0.0,
            emergence_profile_max_penetration: 0.1,
            max_height: 15.0,
            shoulder_size: 0.5,
            angularity: 0.5,
            screw_channel_diameter: 2.3,
            screw_channel_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeParameters {
    pub connector_area: f64,
    pub connector_width: f64,
    pub connector_height: f64,
    pub connector_below_contact: f64,
    pub min_connector_cross_section: f64,
    pub insertion_axis_divergence: f64,
}

impl Default for BridgeParameters {
    fn default() -> Self {
        Self {
            connector_area: 9.0,
            connector_width: 2.0,
            connector_height: 2.0,
            connector_below_contact: 1.4,
            min_connector_cross_section: 4.0,
            insertion_axis_divergence: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlayParameters {
    pub min_thickness: f64,
    pub cement_gap: f64,
    pub additional_spacing: f64,
    pub thickness_run_out_distance: f64,
}

impl Default for InlayParameters {
    fn default() -> Self {
        Self {
            min_thickness: 1.5,
            cement_gap: 0.05,
            additional_spacing: 0.02,
            thickness_run_out_distance: 0.75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarParameters {
    pub default_height: f64,
    pub default_width: f64,
    pub min_height: f64,
    pub min_width: f64,
    pub rounding_radius: f64,
}

impl Default for BarParameters {
    fn default() -> Self {
        Self {
            default_height: 4.0,
            default_width: 3.0,
            min_height: 2.0,
            min_width: 2.0,
            rounding_radius: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelescopeParameters {
    pub angle_mesial: f64,
    pub angle_distal: f64,
    pub angle_buccal: f64,
    pub angle_lingual: f64,
    pub friction_milling_diameter: f64,
}

impl Default for TelescopeParameters {
    fn default() -> Self {
        Self {
            angle_mesial: 0.0,
            angle_distal: 0.0,
            angle_buccal: 0.0,
            angle_lingual: 0.0,
            friction_milling_diameter: 1.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiteSplintParameters {
    pub default_thickness: f64,
    pub min_thickness: f64,
    pub relief_distance: f64,
    pub margin_offset: f64,
}

impl Default for BiteSplintParameters {
    fn default() -> Self {
        Self {
            default_thickness: 2.0,
            min_thickness: 1.0,
            relief_distance: 0.1,
            margin_offset: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralParameters {
    pub distance_to_antagonist: f64,
    pub distance_to_neighbor: f64,
    pub distance_to_gingiva: f64,
    pub freeform_brush_size: f64,
    pub freeform_strength: f64,
    pub smoothing_iterations: i32,
}

impl Default for GeneralParameters {
    fn default() -> Self {
        Self {
            distance_to_antagonist: 0.1,
            distance_to_neighbor: 0.0,
            distance_to_gingiva: 0.0,
            freeform_brush_size: 2.0,
            freeform_strength: 0.5,
            smoothing_iterations: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MillingParameters {
    pub default_tool_diameter: f64,
    pub finishing_tool_diameter: f64,
    pub roughing_tool_diameter: f64,
    pub spindle_speed: u32,
    pub feed_rate: f64,
    pub step_down: f64,
    pub stock_to_leave: f64,
}

impl Default for MillingParameters {
    fn default() -> Self {
        Self {
            default_tool_diameter: 1.2,
            finishing_tool_diameter: 0.6,
            roughing_tool_diameter: 2.0,
            spindle_speed: 30000,
            feed_rate: 1200.0,
            step_down: 0.3,
            stock_to_leave: 0.1,
        }
    }
}

impl DefaultParameters {
    /// Get parameter with min/max validation
    pub fn get_crown_min_thickness(&self) -> f64 {
        self.crown.min_thickness.clamp(0.3, 2.0)
    }

    pub fn get_abutment_min_thickness(&self) -> f64 {
        self.abutment.min_thickness.clamp(0.4, 1.2)
    }

    pub fn get_connector_area(&self) -> f64 {
        self.bridge.connector_area.clamp(4.0, 20.0)
    }
}

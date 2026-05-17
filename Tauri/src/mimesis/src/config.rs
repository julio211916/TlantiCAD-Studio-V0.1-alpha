use serde::{Deserialize, Serialize};

/// Mask generation method when no explicit mask is provided.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MaskMethod {
    /// Use alpha channel transparency.
    #[default]
    Alpha,
    /// Use brightness/luminance.
    Luminance,
    /// Use red channel.
    Red,
    /// Use green channel.
    Green,
    /// Use blue channel.
    Blue,
}

/// Processing configuration for mimesis pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimesisConfig {
    /// Polygon simplification tolerance (Ramer-Douglas-Peucker).
    /// Higher = simpler mesh, fewer vertices.
    pub simplify_tolerance: f64,

    /// Number of Chaikin smoothing iterations.
    pub smooth_iterations: u32,

    /// Extrusion depth in units.
    pub extrude_height: f64,

    /// Minimum polygon dimension in pixels to keep (filters noise).
    pub min_polygon_dimension: u32,

    /// Binary mask threshold (0–255).
    pub threshold: u8,

    /// Mask generation method when no explicit mask provided.
    pub mask_method: MaskMethod,

    /// Skip saving intermediate visualization files.
    pub skip_intermediates: bool,
}

impl Default for MimesisConfig {
    fn default() -> Self {
        Self {
            simplify_tolerance: 10.0,
            smooth_iterations: 1,
            extrude_height: 20.0,
            min_polygon_dimension: 0,
            threshold: 128,
            mask_method: MaskMethod::Alpha,
            skip_intermediates: false,
        }
    }
}

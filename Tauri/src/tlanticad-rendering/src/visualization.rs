//! S221-S225: Visualization modes — wireframe, X-ray, color maps, split view

use serde::{Deserialize, Serialize};

/// Available display modes in the viewport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayMode {
    Shaded,
    ShadedWireframe,
    Wireframe,
    XRay,
    FlatShaded,
    Matcap,
    VertexColors,
    Normal,
    Depth,
    AmbientOcclusion,
}

/// Color map type for scalar visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorMapType {
    /// Cool-warm diverging (blue → white → red)
    CoolWarm,
    /// Rainbow (HSV sweep)
    Rainbow,
    /// Viridis (perceptually uniform)
    Viridis,
    /// Magma (perceptually uniform)
    Magma,
    /// Grayscale
    Grayscale,
    /// Red-Yellow-Green (traffic light – for quality)
    RedYellowGreen,
    /// Custom two-color gradient
    TwoColor,
}

/// Scalar field visualization config (e.g., thickness heat map)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorMapConfig {
    pub map_type: ColorMapType,
    pub min_value: f64,
    pub max_value: f64,
    pub show_legend: bool,
    pub legend_label: String,
    pub legend_unit: String,
    pub steps: u32,
}

impl Default for ColorMapConfig {
    fn default() -> Self {
        Self {
            map_type: ColorMapType::CoolWarm,
            min_value: 0.0,
            max_value: 1.0,
            show_legend: true,
            legend_label: "Value".into(),
            legend_unit: "mm".into(),
            steps: 256,
        }
    }
}

impl ColorMapConfig {
    /// Map a value in [min_value, max_value] to a normalized t in [0, 1].
    pub fn normalize(&self, value: f64) -> f64 {
        let range = self.max_value - self.min_value;
        if range.abs() < 1e-12 { return 0.5; }
        ((value - self.min_value) / range).clamp(0.0, 1.0)
    }

    /// Sample the color map, returning [r, g, b] in 0..1.
    pub fn sample(&self, value: f64) -> [f32; 3] {
        let t = self.normalize(value) as f32;
        match self.map_type {
            ColorMapType::CoolWarm => {
                // Blue → White → Red
                if t < 0.5 {
                    let s = t * 2.0;
                    [s, s, 1.0]
                } else {
                    let s = (1.0 - t) * 2.0;
                    [1.0, s, s]
                }
            }
            ColorMapType::Grayscale => [t, t, t],
            ColorMapType::RedYellowGreen => {
                if t < 0.5 {
                    [1.0, t * 2.0, 0.0]
                } else {
                    [(1.0 - t) * 2.0, 1.0, 0.0]
                }
            }
            _ => {
                // Viridis approximation
                let r = (0.267 + 1.0 * t - 0.5 * t * t).clamp(0.0, 1.0);
                let g = (0.004 + 1.0 * t).clamp(0.0, 1.0);
                let b = (0.329 + 0.6 * t - 1.0 * t * t).clamp(0.0, 1.0);
                [r, g, b]
            }
        }
    }
}

/// Occlusion contact heatmap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionHeatmap {
    pub color_map: ColorMapConfig,
    pub contact_threshold_mm: f64,
    pub near_contact_threshold_mm: f64,
    pub contact_color: [f32; 3],
    pub near_contact_color: [f32; 3],
}

impl Default for OcclusionHeatmap {
    fn default() -> Self {
        Self {
            color_map: ColorMapConfig {
                map_type: ColorMapType::RedYellowGreen,
                min_value: 0.0,
                max_value: 2.0,
                legend_label: "Distance".into(),
                legend_unit: "mm".into(),
                ..Default::default()
            },
            contact_threshold_mm: 0.1,
            near_contact_threshold_mm: 0.5,
            contact_color: [1.0, 0.0, 0.0],
            near_contact_color: [1.0, 1.0, 0.0],
        }
    }
}

/// Split-view configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitViewMode {
    None,
    Horizontal,
    Vertical,
}

/// Per-viewport display override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDisplayConfig {
    pub mode: DisplayMode,
    pub background_color: [f32; 4],
    pub show_grid: bool,
    pub show_axes: bool,
    pub grid_size: f32,
    pub grid_divisions: u32,
    pub opacity_override: Option<f32>,
    pub color_map: Option<ColorMapConfig>,
    pub occlusion_heatmap: Option<OcclusionHeatmap>,
    pub split_view: SplitViewMode,
}

impl Default for ViewDisplayConfig {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Shaded,
            background_color: [0.15, 0.15, 0.17, 1.0],
            show_grid: true,
            show_axes: true,
            grid_size: 100.0,
            grid_divisions: 10,
            opacity_override: None,
            color_map: None,
            occlusion_heatmap: None,
            split_view: SplitViewMode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_map_normalize() {
        let cm = ColorMapConfig { min_value: 0.0, max_value: 10.0, ..Default::default() };
        assert!((cm.normalize(5.0) - 0.5).abs() < 0.001);
        assert!((cm.normalize(-5.0)).abs() < 0.001);
        assert!((cm.normalize(15.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color_map_sample_cool_warm() {
        let cm = ColorMapConfig {
            map_type: ColorMapType::CoolWarm,
            min_value: 0.0, max_value: 1.0,
            ..Default::default()
        };
        let blue = cm.sample(0.0);
        assert!(blue[2] > blue[0]); // blue > red at low

        let red = cm.sample(1.0);
        assert!(red[0] > red[2]); // red > blue at high
    }

    #[test]
    fn test_color_map_grayscale() {
        let cm = ColorMapConfig {
            map_type: ColorMapType::Grayscale,
            min_value: 0.0, max_value: 1.0,
            ..Default::default()
        };
        let mid = cm.sample(0.5);
        assert!((mid[0] - 0.5).abs() < 0.01);
        assert_eq!(mid[0], mid[1]);
        assert_eq!(mid[1], mid[2]);
    }

    #[test]
    fn test_display_modes() {
        let config = ViewDisplayConfig::default();
        assert_eq!(config.mode, DisplayMode::Shaded);
        assert!(config.show_grid);
        assert!(config.show_axes);
    }

    #[test]
    fn test_occlusion_heatmap() {
        let hm = OcclusionHeatmap::default();
        assert_eq!(hm.contact_threshold_mm, 0.1);
        assert_eq!(hm.near_contact_threshold_mm, 0.5);
    }
}

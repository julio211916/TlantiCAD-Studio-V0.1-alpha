//! Bar cross-section shapes and area calculations

use serde::{Deserialize, Serialize};
use crate::bar_design::BarMaterial;

/// Bar cross-section shape options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossSectionShape {
    /// Round bar with diameter d
    Round(f64),
    /// Oval bar (width × height)
    Oval { w: f64, h: f64 },
    /// Dolder bar (trapezoidal with rounded top)
    Dolder { w: f64, h: f64 },
    /// Rectangular bar (width × height)
    Rectangular { w: f64, h: f64 },
}

/// Calculate the cross-sectional area (mm²) of the given shape.
pub fn cross_section_area(shape: &CrossSectionShape) -> f64 {
    match shape {
        CrossSectionShape::Round(d) => std::f64::consts::PI * (d / 2.0).powi(2),
        CrossSectionShape::Oval { w, h } => std::f64::consts::PI * (w / 2.0) * (h / 2.0),
        CrossSectionShape::Dolder { w, h } => {
            // Approximate Dolder as trapezoid with semicircular top
            // Base ≈ 0.7w, top = w, height h; plus semicircle of radius w/2
            let trapezoid = 0.5 * (0.7 * w + w) * h;
            let semicircle = std::f64::consts::PI * (w / 2.0).powi(2) / 2.0;
            trapezoid + semicircle
        }
        CrossSectionShape::Rectangular { w, h } => w * h,
    }
}

/// Return the minimum recommended cross-section for the given material
/// to ensure structural rigidity of the bar framework.
pub fn minimum_cross_section(material: &BarMaterial) -> CrossSectionShape {
    match material {
        BarMaterial::Titanium => CrossSectionShape::Oval { w: 3.0, h: 4.0 },
        BarMaterial::CobaltChrome => CrossSectionShape::Oval { w: 2.5, h: 3.5 },
        BarMaterial::PEEK => CrossSectionShape::Rectangular { w: 4.0, h: 5.0 },
        BarMaterial::Zirconia => CrossSectionShape::Rectangular { w: 4.0, h: 5.0 },
    }
}

//! Model base plate design

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Base plate design type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    Flat,
    Horseshoe,
    SplitCast,
    Articulator,
}

/// Geometric parameters for a model base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBase {
    pub base_type: BaseType,
    /// Base height (cervico-apical) in mm
    pub height: f64,
    /// Arch width in mm
    pub width: f64,
    /// Wall thickness in mm
    pub thickness: f64,
}

impl Default for ModelBase {
    fn default() -> Self {
        Self {
            base_type: BaseType::Horseshoe,
            height: 15.0,
            width: 80.0,
            thickness: 3.0,
        }
    }
}

/// Generate the base outline polygon from an arch scan mesh.
///
/// Returns a convex-hull-like list of points forming the base perimeter.
pub fn generate_base_outline(arch_mesh: &Mesh, base: &ModelBase) -> Vec<Point3<f64>> {
    if arch_mesh.vertices.is_empty() {
        return Vec::new();
    }

    let (min, max) = arch_mesh.calculate_bounds();
    let base_z = min.z - base.height;

    let cx = (min.x + max.x) / 2.0;
    let cy = (min.y + max.y) / 2.0;
    let rx = (max.x - min.x) / 2.0 + base.thickness;
    let ry = (max.y - min.y) / 2.0 + base.thickness;

    // Generate elliptical outline (24 points)
    let n = 24usize;
    let mut outline = Vec::with_capacity(n);
    for i in 0..n {
        let angle = (i as f64) * 2.0 * std::f64::consts::PI / n as f64;
        outline.push(Point3::new(
            cx + rx * angle.cos(),
            cy + ry * angle.sin(),
            base_z,
        ));
    }

    // For horseshoe: remove the posterior third
    if base.base_type == BaseType::Horseshoe {
        let posterior_threshold = cy + ry * 0.3;
        outline.retain(|p| p.y < posterior_threshold);
        // Close the horseshoe with two straight points
        outline.push(Point3::new(cx - rx * 0.6, posterior_threshold, base_z));
        outline.push(Point3::new(cx + rx * 0.6, posterior_threshold, base_z));
    }

    outline
}

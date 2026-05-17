//! Pontic designs for bridge frameworks

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Pontic design style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PonticDesign {
    /// Ovate (egg-shaped tissue-contact)
    Ovate,
    /// Bullet (modified ovate)
    Bullet,
    /// Modified ridge lap
    Modified,
    /// Sanitary / hygienic (no tissue contact)
    Sanitary,
    /// Conical (tapers to a point)
    Conical,
}

/// Parameters defining a pontic unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonticParams {
    pub design: PonticDesign,
    /// Contact area with soft tissue in mm²
    pub tissue_contact_area: f64,
    /// Pontic height in mm
    pub height: f64,
    /// Buccal width in mm
    pub width_buccal: f64,
    /// Lingual width in mm
    pub width_lingual: f64,
}

impl Default for PonticParams {
    fn default() -> Self {
        Self {
            design: PonticDesign::Ovate,
            tissue_contact_area: 10.0,
            height: 8.0,
            width_buccal: 8.0,
            width_lingual: 7.0,
        }
    }
}

/// Generate a basic pontic mesh from parameters and ridge saddle points.
///
/// The pontic is approximated as an ellipsoid scaled to the given dimensions.
pub fn generate_pontic(params: &PonticParams, ridge_saddle: &[Point3<f64>]) -> Mesh {
    // Determine centre point — average of saddle points or origin
    let center = if ridge_saddle.is_empty() {
        Point3::origin()
    } else {
        let sum: nalgebra::Vector3<f64> = ridge_saddle.iter().map(|p| p.coords).sum();
        Point3::from(sum / ridge_saddle.len() as f64)
    };

    let rx = params.width_buccal / 2.0;
    let ry = params.width_lingual / 2.0;
    let rz = params.height / 2.0;

    // Generate a UV sphere and scale to the pontic dimensions
    let mut mesh = tlanticad_mesh::create_sphere(center, 1.0, 16, 12);
    for v in &mut mesh.vertices {
        let local = v.coords - center.coords;
        v.coords = center.coords + nalgebra::Vector3::new(local.x * rx, local.y * ry, local.z * rz);
    }

    // Flatten tissue-contact area based on design type
    match params.design {
        PonticDesign::Ovate => {
            // Push inferior vertices down to form a convex tissue contact
            for v in &mut mesh.vertices {
                if v.z < center.z {
                    v.z = center.z - (center.z - v.z).sqrt() * rz.sqrt();
                }
            }
        }
        PonticDesign::Sanitary => {
            // Lift the base away from the tissue — add 2 mm clearance
            for v in &mut mesh.vertices {
                v.z += 2.0;
            }
        }
        PonticDesign::Conical => {
            // Taper the base to a point
            for v in &mut mesh.vertices {
                if v.z < center.z {
                    let t = 1.0 - (center.z - v.z) / rz;
                    v.coords.x = center.x + (v.coords.x - center.x) * t;
                    v.coords.y = center.y + (v.coords.y - center.y) * t;
                }
            }
        }
        _ => {}
    }

    mesh.name = "pontic".into();
    mesh.calculate_normals();
    mesh
}

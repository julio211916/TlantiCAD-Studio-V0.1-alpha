//! PBR dental material system for realistic rendering

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dental material categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    Zirconia,
    MetalCeramic,
    PMMA,
    Titanium,
    Emax,
    Composite,
    Wax,
    Gingiva,
    Bone,
}

/// Physically-based rendering material for dental restorations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: Uuid,
    pub name: String,
    pub material_type: MaterialType,
    /// RGBA base color (linear, 0..1)
    pub base_color: [f32; 4],
    /// Metallic factor (0 = dielectric, 1 = metal)
    pub metallic: f32,
    /// Surface roughness (0 = mirror, 1 = diffuse)
    pub roughness: f32,
    /// Opacity (1 = fully opaque)
    pub opacity: f32,
    /// Emissive color (usually black for dental)
    pub emission: [f32; 3],
}

impl Material {
    /// Create a zirconia material preset (white, slightly translucent)
    pub fn zirconia() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Zirconia".into(),
            material_type: MaterialType::Zirconia,
            base_color: [0.96, 0.94, 0.90, 1.0],
            metallic: 0.0,
            roughness: 0.15,
            opacity: 1.0,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create a metal-ceramic (PFM) material preset
    pub fn metal_ceramic() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Metal-Ceramic".into(),
            material_type: MaterialType::MetalCeramic,
            base_color: [0.92, 0.88, 0.82, 1.0],
            metallic: 0.3,
            roughness: 0.2,
            opacity: 1.0,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create a PMMA provisional material preset
    pub fn pmma() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "PMMA".into(),
            material_type: MaterialType::PMMA,
            base_color: [0.97, 0.93, 0.84, 1.0],
            metallic: 0.0,
            roughness: 0.35,
            opacity: 1.0,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create a titanium material preset
    pub fn titanium() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Titanium".into(),
            material_type: MaterialType::Titanium,
            base_color: [0.75, 0.73, 0.70, 1.0],
            metallic: 0.9,
            roughness: 0.3,
            opacity: 1.0,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create an IPS e.max (lithium disilicate) material preset
    pub fn emax() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "IPS e.max".into(),
            material_type: MaterialType::Emax,
            base_color: [0.94, 0.91, 0.88, 0.92],
            metallic: 0.0,
            roughness: 0.1,
            opacity: 0.92,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create a diagnostic wax material preset
    pub fn wax() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Wax".into(),
            material_type: MaterialType::Wax,
            base_color: [0.98, 0.87, 0.42, 0.85],
            metallic: 0.0,
            roughness: 0.5,
            opacity: 0.85,
            emission: [0.0, 0.0, 0.0],
        }
    }

    /// Create a gingiva soft tissue material preset
    pub fn gingiva() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Gingiva".into(),
            material_type: MaterialType::Gingiva,
            base_color: [0.92, 0.60, 0.58, 1.0],
            metallic: 0.0,
            roughness: 0.65,
            opacity: 1.0,
            emission: [0.0, 0.0, 0.0],
        }
    }
}

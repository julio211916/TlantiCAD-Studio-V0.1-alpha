//! Shader and material definitions for rendering

use nalgebra::Vector3;

/// Material properties for rendering
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
    pub opacity: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".into(),
            albedo: Vector3::new(0.8, 0.8, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
            opacity: 1.0,
        }
    }
}

impl Material {
    pub fn tooth() -> Self {
        Self {
            name: "tooth".into(),
            albedo: Vector3::new(0.95, 0.92, 0.85),
            metallic: 0.0,
            roughness: 0.3,
            ..Default::default()
        }
    }

    pub fn metal() -> Self {
        Self {
            name: "metal".into(),
            albedo: Vector3::new(0.85, 0.85, 0.87),
            metallic: 1.0,
            roughness: 0.2,
            ..Default::default()
        }
    }

    pub fn gingiva() -> Self {
        Self {
            name: "gingiva".into(),
            albedo: Vector3::new(0.85, 0.55, 0.55),
            metallic: 0.0,
            roughness: 0.7,
            ..Default::default()
        }
    }

    pub fn zirconia() -> Self {
        Self {
            name: "zirconia".into(),
            albedo: Vector3::new(0.95, 0.95, 0.93),
            metallic: 0.0,
            roughness: 0.15,
            ..Default::default()
        }
    }

    pub fn transparent() -> Self {
        Self {
            name: "transparent".into(),
            albedo: Vector3::new(0.6, 0.8, 1.0),
            metallic: 0.0,
            roughness: 0.1,
            opacity: 0.3,
            ..Default::default()
        }
    }
}

/// Light source
#[derive(Debug, Clone)]
pub enum Light {
    Directional { direction: Vector3<f32>, color: Vector3<f32>, intensity: f32 },
    Point { position: Vector3<f32>, color: Vector3<f32>, intensity: f32, range: f32 },
    Ambient { color: Vector3<f32>, intensity: f32 },
}

impl Light {
    pub fn default_scene() -> Vec<Light> {
        vec![
            Light::Directional {
                direction: Vector3::new(-0.5, -1.0, -0.3).normalize(),
                color: Vector3::new(1.0, 1.0, 1.0),
                intensity: 0.8,
            },
            Light::Directional {
                direction: Vector3::new(0.5, 0.5, 0.8).normalize(),
                color: Vector3::new(0.9, 0.95, 1.0),
                intensity: 0.4,
            },
            Light::Ambient {
                color: Vector3::new(1.0, 1.0, 1.0),
                intensity: 0.2,
            },
        ]
    }
}

/// Render settings
#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub wireframe: bool,
    pub show_normals: bool,
    pub show_edges: bool,
    pub background_color: [f32; 4],
    pub msaa_samples: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            wireframe: false,
            show_normals: false,
            show_edges: false,
            background_color: [0.15, 0.15, 0.18, 1.0],
            msaa_samples: 4,
        }
    }
}

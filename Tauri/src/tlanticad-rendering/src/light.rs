//! Lighting system for dental CAD rendering

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Light source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

/// A light source in the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub id: Uuid,
    pub light_type: LightType,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
}

impl Light {
    /// Create a default directional light from above-front
    pub fn directional_default() -> Self {
        Self {
            id: Uuid::new_v4(),
            light_type: LightType::Directional,
            position: Point3::new(0.0, 10.0, 10.0),
            direction: Vector3::new(0.0, -0.7, -0.7).normalize(),
            color: [1.0, 0.98, 0.95],
            intensity: 1.0,
            cast_shadows: true,
        }
    }

    /// Create a point light at a given position
    pub fn point_at(position: Point3<f32>, intensity: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            light_type: LightType::Point,
            position,
            direction: Vector3::new(0.0, -1.0, 0.0),
            color: [1.0, 1.0, 1.0],
            intensity,
            cast_shadows: false,
        }
    }

    /// Create a professional 3-point dental studio lighting setup.
    ///
    /// Returns key light (front-top), fill light (left side), and
    /// back/rim light (rear) for optimal dental visualization.
    pub fn dental_studio_setup() -> Vec<Light> {
        vec![
            // Key light — main illumination from front-top
            Light {
                id: Uuid::new_v4(),
                light_type: LightType::Directional,
                position: Point3::new(5.0, 10.0, 8.0),
                direction: Vector3::new(-0.4, -0.7, -0.6).normalize(),
                color: [1.0, 0.98, 0.95],
                intensity: 1.2,
                cast_shadows: true,
            },
            // Fill light — softer from left to reduce harsh shadows
            Light {
                id: Uuid::new_v4(),
                light_type: LightType::Directional,
                position: Point3::new(-8.0, 5.0, 4.0),
                direction: Vector3::new(0.7, -0.5, -0.5).normalize(),
                color: [0.85, 0.90, 1.0],
                intensity: 0.5,
                cast_shadows: false,
            },
            // Rim light — rear to separate object from background
            Light {
                id: Uuid::new_v4(),
                light_type: LightType::Directional,
                position: Point3::new(0.0, 3.0, -12.0),
                direction: Vector3::new(0.0, -0.3, 1.0).normalize(),
                color: [0.9, 0.9, 1.0],
                intensity: 0.4,
                cast_shadows: false,
            },
        ]
    }
}

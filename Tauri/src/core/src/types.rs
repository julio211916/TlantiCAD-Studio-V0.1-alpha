//! Core types for TlantiStudio

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for entities
pub type EntityId = Uuid;

/// 3D Vector
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            *self
        }
    }
}

/// 4x4 Transformation Matrix
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Mat4 {
    pub data: [[f64; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// Transform component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: Vec3::one(),
        }
    }
}

/// Mesh data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub id: EntityId,
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
    pub transform: Transform,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MeshData {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            transform: Transform::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: EntityId,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            path: path.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// ML Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlModel {
    pub id: EntityId,
    pub name: String,
    pub model_type: MlModelType,
    pub path: String,
    pub input_shape: Vec<i64>,
    pub output_shape: Vec<i64>,
}

/// ML Model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MlModelType {
    Onnx,
    PyTorch,
    TensorFlow,
    Custom(String),
}

/// Processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Idle,
    Processing { progress: f32, message: String },
    Completed,
    Failed { error: String },
}

//! Scene graph for the 3D rendering system

use std::collections::HashMap;
use nalgebra::Isometry3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::camera::OrbitCamera;
use crate::light::Light;

/// A node in the scene graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: Uuid,
    pub name: String,
    pub transform: Isometry3<f64>,
    pub mesh_id: Option<Uuid>,
    pub material_id: Option<Uuid>,
    pub visible: bool,
    pub children: Vec<Uuid>,
}

impl SceneNode {
    /// Create a new scene node with identity transform
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            transform: Isometry3::identity(),
            mesh_id: None,
            material_id: None,
            visible: true,
            children: Vec::new(),
        }
    }
}

/// The 3D scene containing all renderable objects
#[derive(Debug, Clone)]
pub struct Scene {
    pub nodes: HashMap<Uuid, SceneNode>,
    pub camera: OrbitCamera,
    pub lights: Vec<Light>,
}

impl Scene {
    /// Create a new empty scene with default camera and dental studio lighting
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            camera: OrbitCamera::default(),
            lights: crate::light::Light::dental_studio_setup(),
        }
    }

    /// Add a node to the scene, returning its id
    pub fn add_node(&mut self, node: SceneNode) -> Uuid {
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node by id, returning it if found
    pub fn remove_node(&mut self, id: &Uuid) -> Option<SceneNode> {
        self.nodes.remove(id)
    }

    /// Get a mutable reference to a node by id
    pub fn find_node_mut(&mut self, id: &Uuid) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id)
    }

    /// Set visibility of a node
    pub fn set_visibility(&mut self, id: &Uuid, visible: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.visible = visible;
        }
    }

    /// Get all visible nodes
    pub fn get_visible_nodes(&self) -> Vec<&SceneNode> {
        self.nodes.values().filter(|n| n.visible).collect()
    }

    /// Update the transform of a node
    pub fn update_transform(&mut self, id: &Uuid, transform: Isometry3<f64>) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.transform = transform;
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

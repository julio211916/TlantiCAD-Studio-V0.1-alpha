use std::collections::HashMap;

use cad_core::{MeshId, Transform3D};
use uuid::Uuid;

/// Unique identifier for a scene object.
pub type SceneObjectId = Uuid;

/// A single renderable entity in the scene graph.
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub id: SceneObjectId,
    /// Which mesh to draw (must exist in the mesh store).
    pub mesh_id: MeshId,
    /// World-space transform.
    pub transform: Transform3D,
    /// Whether to include this object in rendering.
    pub visible: bool,
}

impl SceneObject {
    pub fn new(mesh_id: MeshId) -> Self {
        Self {
            id: Uuid::new_v4(),
            mesh_id,
            transform: Transform3D::identity(),
            visible: true,
        }
    }
}

/// Flat scene graph — a named collection of renderable objects.
///
/// For TlantiCAD Sprint 1 this is intentionally simple (no hierarchy).
/// Hierarchy / parent-child support can be added later.
#[derive(Debug, Default)]
pub struct SceneGraph {
    objects: HashMap<SceneObjectId, SceneObject>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an object and return its ID.
    pub fn insert(&mut self, obj: SceneObject) -> SceneObjectId {
        let id = obj.id;
        self.objects.insert(id, obj);
        id
    }

    /// Remove an object by ID. Returns `Some(obj)` if it existed.
    pub fn remove(&mut self, id: SceneObjectId) -> Option<SceneObject> {
        self.objects.remove(&id)
    }

    /// Get a reference to an object by ID.
    pub fn get(&self, id: SceneObjectId) -> Option<&SceneObject> {
        self.objects.get(&id)
    }

    /// Mutable reference to an object by ID.
    pub fn get_mut(&mut self, id: SceneObjectId) -> Option<&mut SceneObject> {
        self.objects.get_mut(&id)
    }

    /// Iterate over visible objects only.
    pub fn iter_visible(&self) -> impl Iterator<Item = &SceneObject> {
        self.objects.values().filter(|o| o.visible)
    }

    /// Total number of objects (visible or not).
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

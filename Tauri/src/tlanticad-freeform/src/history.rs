//! Undo/redo history stack for sculpting sessions

use nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;
use std::time::Instant;

/// A lightweight snapshot of mesh geometry
#[derive(Debug, Clone)]
pub struct MeshSnapshot {
    pub vertices: Vec<Point3<f64>>,
    pub normals: Vec<Vector3<f64>>,
    pub timestamp: Instant,
}

impl MeshSnapshot {
    fn from_mesh(mesh: &Mesh) -> Self {
        Self {
            vertices: mesh.vertices.clone(),
            normals: mesh.normals.clone(),
            timestamp: Instant::now(),
        }
    }

    fn apply_to(&self, mesh: &mut Mesh) {
        mesh.vertices = self.vertices.clone();
        mesh.normals = self.normals.clone();
    }
}

/// Undo/redo stack for a sculpting session
#[derive(Debug)]
pub struct HistoryStack {
    pub undo_stack: Vec<MeshSnapshot>,
    pub redo_stack: Vec<MeshSnapshot>,
    pub max_size: usize,
}

impl HistoryStack {
    /// Create a new history stack with the given capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Push the current mesh state onto the undo stack and clear the redo stack
    pub fn push(&mut self, mesh: &Mesh) {
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(MeshSnapshot::from_mesh(mesh));
        self.redo_stack.clear();
    }

    /// Undo the last operation.  Returns `true` if an undo was applied.
    pub fn undo(&mut self, mesh: &mut Mesh) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            self.redo_stack.push(MeshSnapshot::from_mesh(mesh));
            snapshot.apply_to(mesh);
            true
        } else {
            false
        }
    }

    /// Redo the last undone operation.  Returns `true` if a redo was applied.
    pub fn redo(&mut self, mesh: &mut Mesh) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(MeshSnapshot::from_mesh(mesh));
            snapshot.apply_to(mesh);
            true
        } else {
            false
        }
    }

    /// Clear all undo and redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Number of available undo steps
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of available redo steps
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

//! Reversible command pattern for undo/redo

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A reversible editing command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCommand {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: CommandKind,
    pub description: String,
}

/// Kinds of reversible editing commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandKind {
    /// Move vertices by delta
    MoveVertices {
        vertex_ids: Vec<u32>,
        delta: [f64; 3],
    },
    /// Set margin line points
    SetMarginLine {
        tooth_number: u8,
        old_points: Vec<[f64; 3]>,
        new_points: Vec<[f64; 3]>,
    },
    /// Modify restoration parameter
    SetParameter {
        path: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    /// Add mesh to scene
    AddMesh {
        mesh_id: Uuid,
        data: Vec<u8>, // serialized mesh
    },
    /// Remove mesh from scene
    RemoveMesh {
        mesh_id: Uuid,
        data: Vec<u8>,
    },
    /// Freeform sculpt stroke
    SculptStroke {
        mesh_id: Uuid,
        vertex_ids: Vec<u32>,
        old_positions: Vec<[f64; 3]>,
        new_positions: Vec<[f64; 3]>,
    },
    /// Boolean operation
    BooleanOp {
        mesh_a_id: Uuid,
        mesh_b_id: Uuid,
        result_id: Uuid,
        operation: String,
    },
    /// Compound command (groups multiple sub-commands)
    Compound {
        children: Vec<EditCommand>,
    },
}

impl EditCommand {
    pub fn new(kind: CommandKind, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind,
            description: description.into(),
        }
    }

    /// Size estimate in bytes for memory budgeting
    pub fn estimated_size(&self) -> usize {
        match &self.kind {
            CommandKind::MoveVertices { vertex_ids, .. } => {
                vertex_ids.len() * 4 + 24
            }
            CommandKind::SetMarginLine { old_points, new_points, .. } => {
                (old_points.len() + new_points.len()) * 24
            }
            CommandKind::SculptStroke { old_positions, new_positions, .. } => {
                (old_positions.len() + new_positions.len()) * 24 + 16
            }
            CommandKind::AddMesh { data, .. } | CommandKind::RemoveMesh { data, .. } => {
                data.len() + 16
            }
            CommandKind::Compound { children } => {
                children.iter().map(|c| c.estimated_size()).sum()
            }
            _ => 256,
        }
    }
}

//! WebAssembly bridge for MeshLib
//!
//! This module handles communication between Rust and MeshLib WASM.
//! MeshLib is compiled to WebAssembly using Emscripten and runs in the frontend.
//! We use IPC to send mesh data between the frontend (WASM) and backend (Rust).

use app_core::types::MeshData;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::Result;

/// Commands that can be sent to MeshLib WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MeshLibCommand {
    /// Load mesh from binary data
    LoadMesh { data: Vec<u8>, format: String },

    /// Boolean union operation
    BooleanUnion { mesh_a_id: String, mesh_b_id: String },

    /// Boolean difference operation
    BooleanDifference { mesh_a_id: String, mesh_b_id: String },

    /// Boolean intersection operation
    BooleanIntersection { mesh_a_id: String, mesh_b_id: String },

    /// Mesh decimation
    Decimate { mesh_id: String, target_ratio: f32 },

    /// Mesh subdivision
    Subdivide { mesh_id: String, iterations: u32 },

    /// Mesh smoothing
    Smooth { mesh_id: String, iterations: u32, lambda: f32 },

    /// Remesh with target edge length
    Remesh { mesh_id: String, target_edge_length: f32 },

    /// Fill holes
    FillHoles { mesh_id: String },

    /// Fix mesh (remove degenerates, fix normals, etc.)
    FixMesh { mesh_id: String },

    /// Export mesh to format
    ExportMesh { mesh_id: String, format: String },
}

/// Response from MeshLib WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MeshLibResponse {
    /// Success with mesh ID
    MeshCreated { id: String },

    /// Success with mesh data
    MeshData { 
        vertices: Vec<f32>,
        indices: Vec<u32>,
        normals: Vec<f32>,
    },

    /// Success with exported data
    ExportData { data: Vec<u8>, format: String },

    /// Operation progress
    Progress { progress: f32, message: String },

    /// Error
    Error { message: String },
}

/// Bridge handler for WASM communication
pub struct WasmBridge {
    /// Pending operations
    pending: std::collections::HashMap<String, MeshLibCommand>,
}

impl WasmBridge {
    pub fn new() -> Self {
        Self {
            pending: std::collections::HashMap::new(),
        }
    }

    /// Queue a command for MeshLib WASM
    pub fn queue_command(&mut self, id: String, command: MeshLibCommand) {
        info!("Queuing MeshLib command: {:?}", command);
        self.pending.insert(id, command);
    }

    /// Get pending commands as JSON for frontend
    pub fn get_pending_commands(&mut self) -> Vec<(String, String)> {
        let commands: Vec<_> = self
            .pending
            .drain()
            .filter_map(|(id, cmd)| {
                serde_json::to_string(&cmd)
                    .ok()
                    .map(|json| (id, json))
            })
            .collect();
        commands
    }

    /// Process response from MeshLib WASM
    pub fn process_response(&self, response: &str) -> Result<MeshLibResponse> {
        serde_json::from_str(response)
            .map_err(|e| crate::MeshError::WasmError(e.to_string()))
    }

    /// Convert MeshData to format suitable for WASM
    pub fn mesh_to_wasm_format(mesh: &MeshData) -> Vec<u8> {
        bincode::serialize(mesh).unwrap_or_default()
    }

    /// Convert WASM format to MeshData
    pub fn wasm_to_mesh_format(data: &[u8]) -> Result<MeshData> {
        bincode::deserialize(data)
            .map_err(|e| crate::MeshError::WasmError(e.to_string()))
    }
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new()
    }
}

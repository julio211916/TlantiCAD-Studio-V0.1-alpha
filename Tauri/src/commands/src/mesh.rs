//! Mesh-related Tauri commands

use app_core::types::MeshData;
use meshlib_bridge::{self};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{CommandError, CommandResult};

/// Mesh state managed by Tauri
pub struct MeshState {
    pub meshes: Arc<RwLock<HashMap<Uuid, MeshData>>>,
    pub wasm_bridge: Arc<RwLock<meshlib_bridge::WasmBridge>>,
}

/// Mesh info for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshInfo {
    pub id: String,
    pub name: String,
    pub vertex_count: usize,
    pub face_count: usize,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
}

impl From<&MeshData> for MeshInfo {
    fn from(m: &MeshData) -> Self {
        let (min, max) = meshlib_bridge::calculate_bounds(m);
        Self {
            id: m.id.to_string(),
            name: m.name.clone(),
            vertex_count: m.vertex_count(),
            face_count: m.face_count(),
            bounds_min: [min.x, min.y, min.z],
            bounds_max: [max.x, max.y, max.z],
        }
    }
}

/// Load mesh from file
#[tauri::command]
pub async fn load_mesh(
    state: State<'_, MeshState>,
    path: String,
) -> CommandResult<MeshInfo> {
    let path = PathBuf::from(&path);
    let mesh = meshlib_bridge::load_mesh(&path)?;

    let info = MeshInfo::from(&mesh);
    state.meshes.write().await.insert(mesh.id, mesh);

    Ok(info)
}

/// Save mesh to file
#[tauri::command]
pub async fn save_mesh(
    state: State<'_, MeshState>,
    id: String,
    path: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let meshes = state.meshes.read().await;
    let mesh = meshes.get(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    let path = PathBuf::from(&path);
    meshlib_bridge::save_mesh(mesh, &path)?;

    Ok(())
}

/// Get mesh info
#[tauri::command]
pub async fn get_mesh_info(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<MeshInfo> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let meshes = state.meshes.read().await;
    let mesh = meshes.get(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    Ok(MeshInfo::from(mesh))
}

/// List all loaded meshes
#[tauri::command]
pub async fn list_meshes(
    state: State<'_, MeshState>,
) -> CommandResult<Vec<MeshInfo>> {
    let meshes = state.meshes.read().await;
    Ok(meshes.values().map(MeshInfo::from).collect())
}

/// Get mesh data for rendering (vertices, indices, normals)
#[tauri::command]
pub async fn get_mesh_data(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<MeshDataPayload> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let meshes = state.meshes.read().await;
    let mesh = meshes.get(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    Ok(MeshDataPayload {
        vertices: mesh.vertices.clone(),
        indices: mesh.indices.clone(),
        normals: mesh.normals.clone(),
        uvs: mesh.uvs.clone(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDataPayload {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
}

/// Recalculate mesh normals
#[tauri::command]
pub async fn recalculate_normals(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let mut meshes = state.meshes.write().await;
    let mesh = meshes.get_mut(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    meshlib_bridge::recalculate_normals(mesh);

    Ok(())
}

/// Center mesh at origin
#[tauri::command]
pub async fn center_mesh(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let mut meshes = state.meshes.write().await;
    let mesh = meshes.get_mut(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    meshlib_bridge::center_mesh(mesh);

    Ok(())
}

/// Scale mesh
#[tauri::command]
pub async fn scale_mesh(
    state: State<'_, MeshState>,
    id: String,
    scale: f32,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let mut meshes = state.meshes.write().await;
    let mesh = meshes.get_mut(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    meshlib_bridge::scale_mesh(mesh, scale);

    Ok(())
}

/// Calculate mesh volume
#[tauri::command]
pub async fn calculate_mesh_volume(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<f64> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let meshes = state.meshes.read().await;
    let mesh = meshes.get(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    Ok(meshlib_bridge::calculate_volume(mesh))
}

/// Calculate mesh surface area
#[tauri::command]
pub async fn calculate_mesh_area(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<f64> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let meshes = state.meshes.read().await;
    let mesh = meshes.get(&uuid)
        .ok_or_else(|| CommandError {
            code: "MESH_NOT_FOUND".to_string(),
            message: format!("Mesh {} not found", id),
        })?;

    Ok(meshlib_bridge::calculate_surface_area(mesh))
}

/// Delete mesh
#[tauri::command]
pub async fn delete_mesh(
    state: State<'_, MeshState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    state.meshes.write().await.remove(&uuid);

    Ok(())
}

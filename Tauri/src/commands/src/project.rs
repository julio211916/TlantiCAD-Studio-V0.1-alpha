//! Project-related Tauri commands

use app_core::types::Project;
use database::{ProjectRepository, SqliteDb};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::{CommandError, CommandResult};

/// Project state managed by Tauri
pub struct ProjectState {
    pub db: Arc<SqliteDb>,
    pub current_project: tokio::sync::RwLock<Option<Project>>,
}

/// Project info for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Project> for ProjectInfo {
    fn from(p: Project) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            description: p.description,
            path: p.path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

/// Create a new project
#[tauri::command]
pub async fn create_project(
    state: State<'_, ProjectState>,
    name: String,
    path: String,
    description: Option<String>,
) -> CommandResult<ProjectInfo> {
    let mut project = Project::new(&name, &path);
    project.description = description;

    let repo = ProjectRepository::new((*state.db).clone());
    repo.create(&project)?;

    *state.current_project.write().await = Some(project.clone());

    Ok(project.into())
}

/// Open an existing project
#[tauri::command]
pub async fn open_project(
    state: State<'_, ProjectState>,
    id: String,
) -> CommandResult<ProjectInfo> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let repo = ProjectRepository::new((*state.db).clone());
    let project = repo.get(uuid)?;

    *state.current_project.write().await = Some(project.clone());

    Ok(project.into())
}

/// Get current project
#[tauri::command]
pub async fn get_current_project(
    state: State<'_, ProjectState>,
) -> CommandResult<Option<ProjectInfo>> {
    let project = state.current_project.read().await;
    Ok(project.as_ref().map(|p| p.clone().into()))
}

/// List all projects
#[tauri::command]
pub async fn list_projects(
    state: State<'_, ProjectState>,
) -> CommandResult<Vec<ProjectInfo>> {
    let repo = ProjectRepository::new((*state.db).clone());
    let projects = repo.list()?;
    Ok(projects.into_iter().map(|p| p.into()).collect())
}

/// Update project
#[tauri::command]
pub async fn update_project(
    state: State<'_, ProjectState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
) -> CommandResult<ProjectInfo> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let repo = ProjectRepository::new((*state.db).clone());
    let mut project = repo.get(uuid)?;

    if let Some(n) = name {
        project.name = n;
    }
    if description.is_some() {
        project.description = description;
    }

    repo.update(&project)?;

    Ok(project.into())
}

/// Delete project
#[tauri::command]
pub async fn delete_project(
    state: State<'_, ProjectState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError {
            code: "INVALID_UUID".to_string(),
            message: e.to_string(),
        })?;

    let repo = ProjectRepository::new((*state.db).clone());
    repo.delete(uuid)?;

    // Clear current project if it was deleted
    let mut current = state.current_project.write().await;
    if let Some(ref p) = *current {
        if p.id == uuid {
            *current = None;
        }
    }

    Ok(())
}

/// Close current project
#[tauri::command]
pub async fn close_project(
    state: State<'_, ProjectState>,
) -> CommandResult<()> {
    *state.current_project.write().await = None;
    Ok(())
}

//! Project persistence: save/load to disk as .gesto files

use crate::ProjectError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

/// Project file header
const MAGIC: &[u8; 6] = b"GESTO\0";
const VERSION: u16 = 1;

/// Metadata for a saved project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub patient_name: Option<String>,
    pub case_description: Option<String>,
    pub version: u16,
}

impl ProjectMeta {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: now,
            modified_at: now,
            patient_name: None,
            case_description: None,
            version: VERSION,
        }
    }
}

/// A complete project file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub meta: ProjectMeta,
    /// Serialized mesh data (all meshes in the scene)
    pub meshes: Vec<MeshEntry>,
    /// Design parameters
    pub parameters: serde_json::Value,
    /// Margin line data
    pub margin_lines: Vec<MarginLineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEntry {
    pub id: Uuid,
    pub name: String,
    pub role: String, // "preparation", "antagonist", "restoration", "scan"
    pub vertices: Vec<[f64; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub visible: bool,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginLineEntry {
    pub tooth_number: u8,
    pub points: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub closed: bool,
}

impl ProjectFile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            meta: ProjectMeta::new(name),
            meshes: Vec::new(),
            parameters: serde_json::json!({}),
            margin_lines: Vec::new(),
        }
    }

    /// Save project to .gesto file
    pub fn save(&mut self, path: &Path) -> Result<PathBuf, ProjectError> {
        self.meta.modified_at = Utc::now();

        let json = serde_json::to_vec(self)
            .map_err(|e| ProjectError::Serialization(e.to_string()))?;

        let mut buf = Vec::with_capacity(8 + json.len());
        buf.extend_from_slice(MAGIC);
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&json);

        let file_path = if path.extension().is_some() {
            path.to_path_buf()
        } else {
            path.with_extension("gesto")
        };

        std::fs::write(&file_path, &buf)?;
        Ok(file_path)
    }

    /// Load project from .gesto file
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let data = std::fs::read(path)?;

        if data.len() < 8 {
            return Err(ProjectError::Serialization("File too small".into()));
        }
        if &data[0..6] != MAGIC {
            return Err(ProjectError::Serialization("Invalid file magic".into()));
        }
        let _version = u16::from_le_bytes([data[6], data[7]]);

        let project: ProjectFile = serde_json::from_slice(&data[8..])
            .map_err(|e| ProjectError::Serialization(e.to_string()))?;
        Ok(project)
    }

    /// Get recent projects from a directory
    pub fn list_recent(dir: &Path, max: usize) -> Result<Vec<ProjectMeta>, ProjectError> {
        let mut projects = Vec::new();

        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("gesto") {
                    if let Ok(project) = Self::load(&path) {
                        projects.push(project.meta);
                    }
                }
            }
        }

        projects.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        projects.truncate(max);
        Ok(projects)
    }
}

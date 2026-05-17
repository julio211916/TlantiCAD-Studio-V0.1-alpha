//! TlantiCAD Library Manager
//!
//! Gestiona el acceso a las librerías dentales:
//! - Implantes (implant/)
//! - Dientes (teeth/)
//! - Barras (bar/)
//! - Pónticos (pontics/)
//! - Retenciones (retentions/)
//! - Efectos de renderizado (rendereffects/)
//! - etc.

pub mod implant;
pub mod teeth;
pub mod library_manager;

pub use implant::*;
pub use teeth::*;
pub use library_manager::*;

use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// Library root path
#[derive(Debug, Clone)]
pub struct LibraryPath {
    pub root: PathBuf,
}

impl LibraryPath {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn implant_path(&self) -> PathBuf {
        self.root.join("implant")
    }

    pub fn teeth_path(&self) -> PathBuf {
        self.root.join("teeth")
    }

    pub fn bar_path(&self) -> PathBuf {
        self.root.join("bar")
    }

    pub fn pontics_path(&self) -> PathBuf {
        self.root.join("pontics")
    }

    pub fn attachments_path(&self) -> PathBuf {
        self.root.join("attachments")
    }

    pub fn retentions_path(&self) -> PathBuf {
        self.root.join("retentions")
    }

    pub fn articulator_path(&self) -> PathBuf {
        self.root.join("articulator")
    }

    pub fn visualizers_path(&self) -> PathBuf {
        self.root.join("visualizers")
    }

    pub fn render_effects_path(&self) -> PathBuf {
        self.root.join("rendereffects")
    }

    pub fn prosthetic_tooth_sets_path(&self) -> PathBuf {
        self.root.join("prosthetictoothsets")
    }

    pub fn prosthetic_tooth_presets_path(&self) -> PathBuf {
        self.root.join("prosthetictoothpresets")
    }

    pub fn metadata_path(&self) -> PathBuf {
        self.root.join("metadata")
    }
}

/// Library manager
#[derive(Debug)]
pub struct Library {
    path: LibraryPath,
    cache: RwLock<LibraryCache>,
}

#[derive(Debug, Default)]
struct LibraryCache {
    implants: Option<Vec<ImplantLibrary>>,
    tooth_sets: Option<Vec<ToothSet>>,
}

impl Library {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: LibraryPath::new(path),
            cache: RwLock::new(LibraryCache::default()),
        }
    }

    /// Get library path
    pub fn path(&self) -> &LibraryPath {
        &self.path
    }

    /// Initialize library (scan and index)
    pub async fn initialize(&self) -> tlanticad_core::Result<()> {
        tracing::info!("Initializing library at: {:?}", self.path.root);

        // Index implant libraries
        let implants = self.scan_implant_libraries().await?;
        let mut cache = self.cache.write().await;
        cache.implants = Some(implants);

        // Index tooth sets
        let tooth_sets = self.scan_tooth_sets().await?;
        cache.tooth_sets = Some(tooth_sets);

        tracing::info!("Library initialized successfully");
        Ok(())
    }

    /// Scan implant libraries
    async fn scan_implant_libraries(&self) -> tlanticad_core::Result<Vec<ImplantLibrary>> {
        let mut libraries = Vec::new();
        let implant_path = self.path.implant_path();

        if !implant_path.exists() {
            tracing::warn!("Implant library path does not exist: {:?}", implant_path);
            return Ok(libraries);
        }

        let mut entries = tokio::fs::read_dir(&implant_path).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))? {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(library) = ImplantLibrary::from_directory(&path, &name).await.ok() {
                    libraries.push(library);
                }
            }
        }

        tracing::info!("Found {} implant libraries", libraries.len());
        Ok(libraries)
    }

    /// Scan tooth sets
    async fn scan_tooth_sets(&self) -> tlanticad_core::Result<Vec<ToothSet>> {
        let mut sets = Vec::new();
        let teeth_path = self.path.teeth_path();

        if !teeth_path.exists() {
            tracing::warn!("Teeth library path does not exist: {:?}", teeth_path);
            return Ok(sets);
        }

        let mut entries = tokio::fs::read_dir(&teeth_path).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))? {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(set) = ToothSet::from_directory(&path, &name).await.ok() {
                    sets.push(set);
                }
            }
        }

        tracing::info!("Found {} tooth sets", sets.len());
        Ok(sets)
    }

    /// Get all implant libraries
    pub async fn get_implant_libraries(&self) -> tlanticad_core::Result<Vec<ImplantLibrary>> {
        let cache = self.cache.read().await;
        if let Some(ref implants) = cache.implants {
            return Ok(implants.clone());
        }
        drop(cache);
        self.scan_implant_libraries().await
    }

    /// Get implant library by ID
    pub async fn get_implant_library(&self, id: &str) -> tlanticad_core::Result<Option<ImplantLibrary>> {
        let libraries = self.get_implant_libraries().await?;
        Ok(libraries.into_iter().find(|l| l.id == id))
    }

    /// Get all tooth sets
    pub async fn get_tooth_sets(&self) -> tlanticad_core::Result<Vec<ToothSet>> {
        let cache = self.cache.read().await;
        if let Some(ref sets) = cache.tooth_sets {
            return Ok(sets.clone());
        }
        drop(cache);
        self.scan_tooth_sets().await
    }

    /// Get tooth set by ID
    pub async fn get_tooth_set(&self, id: &str) -> tlanticad_core::Result<Option<ToothSet>> {
        let sets = self.get_tooth_sets().await?;
        Ok(sets.into_iter().find(|s| s.id == id))
    }

    /// Search libraries
    pub async fn search(&self, query: &str) -> LibrarySearchResults {
        let mut results = LibrarySearchResults::default();
        let query = query.to_lowercase();

        if let Ok(libraries) = self.get_implant_libraries().await {
            results.implants = libraries
                .into_iter()
                .filter(|l| {
                    l.name.to_lowercase().contains(&query) ||
                    l.manufacturer.to_lowercase().contains(&query)
                })
                .collect();
        }

        if let Ok(sets) = self.get_tooth_sets().await {
            results.tooth_sets = sets
                .into_iter()
                .filter(|s| s.name.to_lowercase().contains(&query))
                .collect();
        }

        results
    }
}

/// Search results
#[derive(Debug, Default)]
pub struct LibrarySearchResults {
    pub implants: Vec<ImplantLibrary>,
    pub tooth_sets: Vec<ToothSet>,
}

/// Get library resource path (works on both dev and production)
pub fn get_library_resource_path() -> PathBuf {
    // In development, use local resources folder
    // In production, use bundled resources
    if cfg!(debug_assertions) {
        PathBuf::from("resources/library")
    } else {
        // In production, resources are in the app bundle
        // This will be resolved at runtime by Tauri
        PathBuf::from("library")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_library_path() {
        let path = LibraryPath::new("/tmp/library");
        assert_eq!(path.implant_path(), PathBuf::from("/tmp/library/implant"));
        assert_eq!(path.teeth_path(), PathBuf::from("/tmp/library/teeth"));
    }
}

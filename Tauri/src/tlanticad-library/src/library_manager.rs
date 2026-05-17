//! Library Manager - High-level API for library operations

use crate::{Library, ImplantLibrary, ToothSet, ImplantLibraryInfo, ToothSetInfo};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global library manager
#[derive(Debug, Clone)]
pub struct LibraryManager {
    library: Arc<RwLock<Library>>,
}

impl LibraryManager {
    /// Create new library manager
    pub fn new(library_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            library: Arc::new(RwLock::new(Library::new(library_path))),
        }
    }

    /// Initialize the library (scan and index)
    pub async fn initialize(&self) -> tlanticad_core::Result<()> {
        let library = self.library.read().await;
        library.initialize().await
    }

    /// Get all implant libraries (for UI)
    pub async fn get_implant_libraries(&self) -> tlanticad_core::Result<Vec<ImplantLibraryInfo>> {
        let library = self.library.read().await;
        let libraries = library.get_implant_libraries().await?;
        Ok(libraries.iter().map(ImplantLibraryInfo::from).collect())
    }

    /// Get full implant library with all implants
    pub async fn get_implant_library(&self, id: &str) -> tlanticad_core::Result<Option<ImplantLibrary>> {
        let library = self.library.read().await;
        library.get_implant_library(id).await
    }

    /// Get all tooth sets (for UI)
    pub async fn get_tooth_sets(&self) -> tlanticad_core::Result<Vec<ToothSetInfo>> {
        let library = self.library.read().await;
        let sets = library.get_tooth_sets().await?;
        Ok(sets.iter().map(ToothSetInfo::from).collect())
    }

    /// Get full tooth set
    pub async fn get_tooth_set(&self, id: &str) -> tlanticad_core::Result<Option<ToothSet>> {
        let library = self.library.read().await;
        library.get_tooth_set(id).await
    }

    /// Search across all libraries
    pub async fn search(&self, query: &str) -> crate::LibrarySearchResults {
        let library = self.library.read().await;
        library.search(query).await
    }

    /// Get implant by library ID and implant ID
    pub async fn get_implant(
        &self,
        library_id: &str,
        implant_id: &str,
    ) -> tlanticad_core::Result<Option<crate::implant::Implant>> {
        if let Some(library) = self.get_implant_library(library_id).await? {
            Ok(library.get_implant(implant_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get tooth by set ID and tooth number
    pub async fn get_tooth(
        &self,
        set_id: &str,
        tooth_number: u8,
    ) -> tlanticad_core::Result<Option<crate::teeth::Tooth>> {
        if let Some(set) = self.get_tooth_set(set_id).await? {
            Ok(set.get_tooth(tooth_number).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get available diameters for an implant library
    pub async fn get_implant_diameters(&self, library_id: &str) -> tlanticad_core::Result<Vec<f64>> {
        if let Some(library) = self.get_implant_library(library_id).await? {
            Ok(library.get_diameters())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get available lengths for a diameter
    pub async fn get_implant_lengths(
        &self,
        library_id: &str,
        diameter: f64,
    ) -> tlanticad_core::Result<Vec<f64>> {
        if let Some(library) = self.get_implant_library(library_id).await? {
            Ok(library.get_lengths_for_diameter(diameter))
        } else {
            Ok(Vec::new())
        }
    }

    /// Get implants matching criteria
    pub async fn find_implants(
        &self,
        library_id: &str,
        diameter: Option<f64>,
        length: Option<f64>,
    ) -> tlanticad_core::Result<Vec<crate::implant::Implant>> {
        if let Some(library) = self.get_implant_library(library_id).await? {
            let mut implants = library.implants.clone();

            if let Some(d) = diameter {
                implants.retain(|i| (i.diameter - d).abs() < 0.1);
            }

            if let Some(l) = length {
                implants.retain(|i| (i.length - l).abs() < 0.1);
            }

            Ok(implants)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Library selection state (for UI)
#[derive(Debug, Clone, Default)]
pub struct LibrarySelection {
    pub selected_implant_library: Option<String>,
    pub selected_implant: Option<String>,
    pub selected_tooth_set: Option<String>,
    pub selected_tooth: Option<u8>,
}

impl LibrarySelection {
    pub fn clear_implant(&mut self) {
        self.selected_implant_library = None;
        self.selected_implant = None;
    }

    pub fn clear_tooth(&mut self) {
        self.selected_tooth_set = None;
        self.selected_tooth = None;
    }

    pub fn is_implant_selected(&self) -> bool {
        self.selected_implant_library.is_some() && self.selected_implant.is_some()
    }

    pub fn is_tooth_selected(&self) -> bool {
        self.selected_tooth_set.is_some() && self.selected_tooth.is_some()
    }
}

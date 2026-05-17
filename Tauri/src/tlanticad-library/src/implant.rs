//! Implant library management

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Implant library (e.g., "exocad_Demo_Implant_plan_fda")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantLibrary {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub connection_type: String,
    pub path: PathBuf,
    pub implants: Vec<Implant>,
}

/// Individual implant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implant {
    pub id: String,
    pub name: String,
    pub diameter: f64,
    pub length: f64,
    pub platform: String,
    pub stl_path: Option<PathBuf>,
    pub sdfa_path: Option<PathBuf>,
}

impl ImplantLibrary {
    /// Load library from directory
    pub async fn from_directory(path: &Path, name: &str) -> tlanticad_core::Result<Self> {
        let mut implants = Vec::new();

        // Read config.xml if exists
        let config_path = path.join("config.xml");
        let (manufacturer, connection_type) = if config_path.exists() {
            Self::parse_config(&config_path).await.unwrap_or_default()
        } else {
            (name.to_string(), "Standard".to_string())
        };

        // Scan for implant files
        let mut entries = tokio::fs::read_dir(path).await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| tlanticad_core::TlantiError::Io(e))? {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if ext == "stl" || ext == "sdfa" {
                    if let Some(stem) = file_path.file_stem() {
                        let stem_str = stem.to_string_lossy();
                        // Skip "_dummy" files (simplified meshes)
                        if !stem_str.ends_with("_dummy") {
                            if let Some(implant) = Self::parse_implant_file(&file_path).await {
                                // Check if we already have this implant
                                if !implants.iter().any(|i: &Implant| i.name == implant.name) {
                                    implants.push(implant);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort implants by diameter and length
        implants.sort_by(|a, b| {
            a.diameter.partial_cmp(&b.diameter)
                .unwrap()
                .then_with(|| a.length.partial_cmp(&b.length).unwrap())
        });

        Ok(Self {
            id: name.to_lowercase().replace(" ", "_"),
            name: name.to_string(),
            manufacturer,
            connection_type,
            path: path.to_path_buf(),
            implants,
        })
    }

    async fn parse_config(path: &Path) -> Option<(String, String)> {
        let content = tokio::fs::read_to_string(path).await.ok()?;
        // Simple XML parsing - extract manufacturer and connection type
        let manufacturer = Self::extract_xml_value(&content, "manufacturer")
            .unwrap_or_else(|| "Unknown".to_string());
        let connection_type = Self::extract_xml_value(&content, "connectionType")
            .unwrap_or_else(|| "Standard".to_string());
        Some((manufacturer, connection_type))
    }

    fn extract_xml_value(content: &str, tag: &str) -> Option<String> {
        let start = content.find(&format!("<{}>", tag))? + tag.len() + 2;
        let end = content.find(&format!("</{}>", tag))?;
        Some(content[start..end].to_string())
    }

    async fn parse_implant_file(path: &Path) -> Option<Implant> {
        let file_name = path.file_stem()?.to_string_lossy();
        let ext = path.extension()?.to_string_lossy();

        // Parse name like "3_3x10_rotM" or "4x12_rotM"
        let name = file_name.to_string();
        
        // Try to extract diameter and length from filename
        let (diameter, length) = Self::extract_dimensions(&name)?;

        let id = name.to_lowercase().replace(" ", "_");

        let mut stl_path = None;
        let mut sdfa_path = None;

        if ext == "stl" {
            stl_path = Some(path.to_path_buf());
            // Check for corresponding SDFA file
            let sdfa_file = path.with_extension("sdfa");
            if sdfa_file.exists() {
                sdfa_path = Some(sdfa_file);
            }
        } else if ext == "sdfa" {
            sdfa_path = Some(path.to_path_buf());
            // Check for corresponding STL file
            let stl_file = path.with_extension("stl");
            if stl_file.exists() {
                stl_path = Some(stl_file);
            }
        }

        Some(Implant {
            id,
            name,
            diameter,
            length,
            platform: Self::extract_platform(&file_name),
            stl_path,
            sdfa_path,
        })
    }

    fn extract_dimensions(name: &str) -> Option<(f64, f64)> {
        // Pattern: "3_3x10" -> diameter 3.3, length 10
        // Pattern: "4x12" -> diameter 4.0, length 12
        
        // Remove suffixes like "_rotM", "_dummy"
        let base = name.split('_').next()?;
        
        // Find 'x' separator
        let parts: Vec<&str> = base.split('x').collect();
        if parts.len() != 2 {
            return None;
        }

        let diameter_str = parts[0].replace('_', ".");
        let length_str = parts[1];

        let diameter = diameter_str.parse::<f64>().ok()?;
        let length = length_str.parse::<f64>().ok()?;

        Some((diameter, length))
    }

    fn extract_platform(name: &str) -> String {
        if name.contains("rotM") {
            "Rotational Morse".to_string()
        } else if name.contains("ext") {
            "External".to_string()
        } else if name.contains("int") {
            "Internal".to_string()
        } else {
            "Standard".to_string()
        }
    }

    /// Get implant by ID
    pub fn get_implant(&self, id: &str) -> Option<&Implant> {
        self.implants.iter().find(|i| i.id == id)
    }

    /// Get implants by diameter
    pub fn get_implants_by_diameter(&self, diameter: f64) -> Vec<&Implant> {
        self.implants.iter()
            .filter(|i| (i.diameter - diameter).abs() < 0.1)
            .collect()
    }

    /// Get available diameters
    pub fn get_diameters(&self) -> Vec<f64> {
        let mut diameters: Vec<f64> = self.implants.iter()
            .map(|i| i.diameter)
            .collect();
        diameters.sort_by(|a, b| a.partial_cmp(b).unwrap());
        diameters.dedup_by(|a, b| (*a - *b).abs() < 0.1);
        diameters
    }

    /// Get available lengths for a diameter
    pub fn get_lengths_for_diameter(&self, diameter: f64) -> Vec<f64> {
        let mut lengths: Vec<f64> = self.implants.iter()
            .filter(|i| (i.diameter - diameter).abs() < 0.1)
            .map(|i| i.length)
            .collect();
        lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lengths.dedup_by(|a, b| (*a - *b).abs() < 0.1);
        lengths
    }
}

/// Library selection info (for UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantLibraryInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub connection_type: String,
    pub implant_count: usize,
    pub available_diameters: Vec<f64>,
}

impl From<&ImplantLibrary> for ImplantLibraryInfo {
    fn from(lib: &ImplantLibrary) -> Self {
        Self {
            id: lib.id.clone(),
            name: lib.name.clone(),
            manufacturer: lib.manufacturer.clone(),
            connection_type: lib.connection_type.clone(),
            implant_count: lib.implants.len(),
            available_diameters: lib.get_diameters(),
        }
    }
}

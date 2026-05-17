//! Pre-built tooth anatomy templates and morphology parameters

use serde::{Deserialize, Serialize};

/// Morphology parameters for generating a tooth shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologyParams {
    /// Clinical crown length in mm
    pub length: f64,
    /// Buccal-lingual width in mm
    pub width_buccal: f64,
    /// Lingual width in mm
    pub width_lingual: f64,
    /// Cusp height above cervical line in mm
    pub cusp_height: f64,
}

/// A serializable anatomy template (mesh stored as raw bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyTemplate {
    pub name: String,
    pub tooth_group: String,
    /// Serialised mesh data (placeholder — empty in library stubs)
    pub mesh_data: Vec<u8>,
}

/// Return the names of available anatomy templates
pub fn get_template_names() -> Vec<&'static str> {
    vec![
        "Upper Central Incisor",
        "Upper Lateral Incisor",
        "Upper Canine",
        "Upper First Premolar",
        "Upper Second Premolar",
        "Upper First Molar",
        "Upper Second Molar",
        "Lower Central Incisor",
        "Lower Lateral Incisor",
        "Lower Canine",
        "Lower First Premolar",
        "Lower Second Premolar",
        "Lower First Molar",
        "Lower Second Molar",
    ]
}

/// Return morphology parameters for an FDI tooth number (11–48).
///
/// Values are population-average dimensions from dental morphology literature.
pub fn get_morphology_params(tooth_number: u8) -> MorphologyParams {
    match tooth_number {
        // Upper right
        11 => MorphologyParams { length: 10.5, width_buccal: 8.5, width_lingual: 7.0, cusp_height: 0.0 },
        12 => MorphologyParams { length: 9.0, width_buccal: 6.5, width_lingual: 5.5, cusp_height: 0.0 },
        13 => MorphologyParams { length: 10.0, width_buccal: 7.5, width_lingual: 7.0, cusp_height: 5.5 },
        14 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 9.0, cusp_height: 5.0 },
        15 => MorphologyParams { length: 8.0, width_buccal: 6.5, width_lingual: 8.5, cusp_height: 4.5 },
        16 => MorphologyParams { length: 7.5, width_buccal: 10.0, width_lingual: 11.0, cusp_height: 5.5 },
        17 => MorphologyParams { length: 7.0, width_buccal: 9.5, width_lingual: 10.5, cusp_height: 5.0 },
        18 => MorphologyParams { length: 6.5, width_buccal: 8.5, width_lingual: 9.5, cusp_height: 4.0 },
        // Upper left (mirror of right)
        21 => MorphologyParams { length: 10.5, width_buccal: 8.5, width_lingual: 7.0, cusp_height: 0.0 },
        22 => MorphologyParams { length: 9.0, width_buccal: 6.5, width_lingual: 5.5, cusp_height: 0.0 },
        23 => MorphologyParams { length: 10.0, width_buccal: 7.5, width_lingual: 7.0, cusp_height: 5.5 },
        24 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 9.0, cusp_height: 5.0 },
        25 => MorphologyParams { length: 8.0, width_buccal: 6.5, width_lingual: 8.5, cusp_height: 4.5 },
        26 => MorphologyParams { length: 7.5, width_buccal: 10.0, width_lingual: 11.0, cusp_height: 5.5 },
        27 => MorphologyParams { length: 7.0, width_buccal: 9.5, width_lingual: 10.5, cusp_height: 5.0 },
        28 => MorphologyParams { length: 6.5, width_buccal: 8.5, width_lingual: 9.5, cusp_height: 4.0 },
        // Lower right
        41 => MorphologyParams { length: 8.8, width_buccal: 5.4, width_lingual: 5.8, cusp_height: 0.0 },
        42 => MorphologyParams { length: 9.2, width_buccal: 5.7, width_lingual: 6.0, cusp_height: 0.0 },
        43 => MorphologyParams { length: 10.0, width_buccal: 6.8, width_lingual: 7.0, cusp_height: 5.0 },
        44 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 8.0, cusp_height: 4.5 },
        45 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 8.5, cusp_height: 4.5 },
        46 => MorphologyParams { length: 7.5, width_buccal: 11.0, width_lingual: 10.5, cusp_height: 5.0 },
        47 => MorphologyParams { length: 7.0, width_buccal: 10.5, width_lingual: 10.0, cusp_height: 4.5 },
        48 => MorphologyParams { length: 6.5, width_buccal: 9.5, width_lingual: 9.0, cusp_height: 3.5 },
        // Lower left (mirror of right)
        31 => MorphologyParams { length: 8.8, width_buccal: 5.4, width_lingual: 5.8, cusp_height: 0.0 },
        32 => MorphologyParams { length: 9.2, width_buccal: 5.7, width_lingual: 6.0, cusp_height: 0.0 },
        33 => MorphologyParams { length: 10.0, width_buccal: 6.8, width_lingual: 7.0, cusp_height: 5.0 },
        34 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 8.0, cusp_height: 4.5 },
        35 => MorphologyParams { length: 8.5, width_buccal: 7.0, width_lingual: 8.5, cusp_height: 4.5 },
        36 => MorphologyParams { length: 7.5, width_buccal: 11.0, width_lingual: 10.5, cusp_height: 5.0 },
        37 => MorphologyParams { length: 7.0, width_buccal: 10.5, width_lingual: 10.0, cusp_height: 4.5 },
        38 => MorphologyParams { length: 6.5, width_buccal: 9.5, width_lingual: 9.0, cusp_height: 3.5 },
        _ => MorphologyParams { length: 8.0, width_buccal: 8.0, width_lingual: 8.0, cusp_height: 4.0 },
    }
}

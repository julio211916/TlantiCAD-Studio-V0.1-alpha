//! Color configuration - Replica colors.xml y defaultcolors.xml de Exocad

use serde::{Deserialize, Serialize};

/// Configuración de colores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    // UI Colors
    pub menu_background_selected: String,
    pub menu_background_unselected: String,
    pub menu_text_selected: String,
    pub menu_text_unselected: String,
    pub menu_headline: String,
    pub menu_info_text: String,
    pub menu_separator: String,
    
    // 3D Viewer Colors
    pub viewer_background_top: String,
    pub viewer_background_bottom: String,
    pub selection_highlight: String,
    pub hover_highlight: String,
    
    // Dental-specific Colors
    pub margin_line: String,
    pub preparation_scan: String,
    pub antagonist_scan: String,
    pub gingiva_scan: String,
    pub bite_scan: String,
    
    // Design Colors
    pub crown_default: String,
    pub abutment_default: String,
    pub bar_default: String,
    pub telescope_default: String,
    
    // Tool Colors
    pub measuring_line: String,
    pub section_plane: String,
    
    // Status Colors
    pub status_ok: String,
    pub status_warning: String,
    pub status_error: String,
    
    // Icon Colors
    pub icon_background_colors: Vec<String>,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            // UI Colors - Dark theme similar to Exocad
            menu_background_selected: "#584f7c".to_string(),
            menu_background_unselected: "#2c2444".to_string(),
            menu_text_selected: "#FFFFFF".to_string(),
            menu_text_unselected: "#FFFFFF".to_string(),
            menu_headline: "#ef895f".to_string(),
            menu_info_text: "#6d6199".to_string(),
            menu_separator: "#584f7c".to_string(),
            
            // 3D Viewer
            viewer_background_top: "#473a6d".to_string(),
            viewer_background_bottom: "#473a6d".to_string(),
            selection_highlight: "#00ff00".to_string(),
            hover_highlight: "#ffff00".to_string(),
            
            // Dental-specific
            margin_line: "#ff0000".to_string(),
            preparation_scan: "#e8d4c4".to_string(),
            antagonist_scan: "#90EE90".to_string(),
            gingiva_scan: "#FFB6C1".to_string(),
            bite_scan: "#ADD8E6".to_string(),
            
            // Design
            crown_default: "#FFFFFF".to_string(),
            abutment_default: "#C0C0C0".to_string(),
            bar_default: "#FFD700".to_string(),
            telescope_default: "#C0C0C0".to_string(),
            
            // Tools
            measuring_line: "#FFFF00".to_string(),
            section_plane: "#00FFFF".to_string(),
            
            // Status
            status_ok: "#4CAF50".to_string(),
            status_warning: "#FF9800".to_string(),
            status_error: "#F44336".to_string(),
            
            // Icon backgrounds
            icon_background_colors: vec![
                "#5b39b5".to_string(),
                "#7b4fc4".to_string(),
                "#9b65d3".to_string(),
            ],
        }
    }
}

impl ColorConfig {
    /// Get color for specific element type
    pub fn get_element_color(&self, element_type: ElementType) -> &str {
        match element_type {
            ElementType::MarginLine => &self.margin_line,
            ElementType::Preparation => &self.preparation_scan,
            ElementType::Antagonist => &self.antagonist_scan,
            ElementType::Gingiva => &self.gingiva_scan,
            ElementType::Crown => &self.crown_default,
            ElementType::Abutment => &self.abutment_default,
            ElementType::Bar => &self.bar_default,
        }
    }
    
    /// Convert hex color to RGB array
    pub fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        
        Some([r, g, b])
    }
    
    /// Convert RGB to hex
    pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    MarginLine,
    Preparation,
    Antagonist,
    Gingiva,
    Crown,
    Abutment,
    Bar,
}

/// Material color with shades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialColor {
    pub name: String,
    pub base_color: String,
    pub shades: Vec<Shade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shade {
    pub name: String,
    pub color: String,
    pub translucency: f64,
}

/// Standard dental shades (VITA Classical)
pub fn standard_shades() -> Vec<Shade> {
    vec![
        Shade { name: "A1".to_string(), color: "#F5E4C8".to_string(), translucency: 0.4 },
        Shade { name: "A2".to_string(), color: "#F0D5A8".to_string(), translucency: 0.4 },
        Shade { name: "A3".to_string(), color: "#E8C598".to_string(), translucency: 0.4 },
        Shade { name: "A3.5".to_string(), color: "#E0B888".to_string(), translucency: 0.4 },
        Shade { name: "A4".to_string(), color: "#D4A070".to_string(), translucency: 0.4 },
        Shade { name: "B1".to_string(), color: "#F5EAD8".to_string(), translucency: 0.4 },
        Shade { name: "B2".to_string(), color: "#F0E0C0".to_string(), translucency: 0.4 },
        Shade { name: "B3".to_string(), color: "#E8D0A8".to_string(), translucency: 0.4 },
        Shade { name: "B4".to_string(), color: "#D8B890".to_string(), translucency: 0.4 },
        Shade { name: "C1".to_string(), color: "#F5F0E0".to_string(), translucency: 0.4 },
        Shade { name: "C2".to_string(), color: "#E8E0C8".to_string(), translucency: 0.4 },
        Shade { name: "C3".to_string(), color: "#D8D0B0".to_string(), translucency: 0.4 },
        Shade { name: "C4".to_string(), color: "#C8C098".to_string(), translucency: 0.4 },
        Shade { name: "D2".to_string(), color: "#F0E8D8".to_string(), translucency: 0.4 },
        Shade { name: "D3".to_string(), color: "#E8D8C0".to_string(), translucency: 0.4 },
        Shade { name: "D4".to_string(), color: "#D8C8A8".to_string(), translucency: 0.4 },
    ]
}

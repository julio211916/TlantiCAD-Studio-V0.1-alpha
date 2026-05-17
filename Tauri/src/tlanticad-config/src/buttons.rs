//! Button/Toolbar configuration - Replica buttons.xml de Exocad

use serde::{Deserialize, Serialize};
use tlanticad_core::ProcessorType;

/// Configuración de botones/herramientas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonConfig {
    pub buttons: Vec<ButtonDefinition>,
    pub toolbar_layout: ToolbarLayout,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            buttons: default_buttons(),
            toolbar_layout: ToolbarLayout::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDefinition {
    pub processor: ProcessorType,
    pub caption: String,
    pub tooltip: String,
    pub icon: String,
    pub category: ButtonCategory,
    pub shortcut: Option<String>,
    pub order: i32,
    pub is_advanced: bool,
    pub requires_selection: bool,
    pub available_in_modes: Vec<AppMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonCategory {
    Import,
    Export,
    Margin,
    Abutment,
    Anatomy,
    Crown,
    Bridge,
    Bar,
    Telescope,
    Inlay,
    BiteSplint,
    WaxUp,
    Gingiva,
    Model,
    Freeform,
    Tools,
    View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode {
    Crown,
    Bridge,
    Abutment,
    Bar,
    Telescope,
    BiteSplint,
    Model,
    WaxUp,
    Freeform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarLayout {
    pub primary_toolbar: Vec<ProcessorType>,
    pub secondary_toolbar: Vec<ProcessorType>,
    pub context_menu: Vec<ProcessorType>,
}

impl Default for ToolbarLayout {
    fn default() -> Self {
        use ProcessorType::*;
        
        Self {
            primary_toolbar: vec![
                PreparationMargin,
                PlaceModelTooth,
                AdaptToothmodel,
                CrownBottom,
                Connector,
                Freeform,
            ],
            secondary_toolbar: vec![
                ImportScan,
                ExportStl,
                MeasuringPoints,
                SectionView,
            ],
            context_menu: vec![
                CorrectPreparationMargin,
                CopyAndPasteTooth,
                DeleteReconstructions,
            ],
        }
    }
}

fn default_buttons() -> Vec<ButtonDefinition> {
    use ProcessorType::*;
    
    vec![
        // Import/Export
        ButtonDefinition {
            processor: ImportScan,
            caption: "Import Scan".to_string(),
            tooltip: "Import scan data from file".to_string(),
            icon: "import_scan".to_string(),
            category: ButtonCategory::Import,
            shortcut: Some("Ctrl+I".to_string()),
            order: 10,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![
                AppMode::Crown, AppMode::Bridge, AppMode::Abutment, 
                AppMode::Bar, AppMode::Telescope, AppMode::BiteSplint,
                AppMode::Model, AppMode::WaxUp
            ],
        },
        ButtonDefinition {
            processor: ExportStl,
            caption: "Export STL".to_string(),
            tooltip: "Export design for manufacturing".to_string(),
            icon: "export_stl".to_string(),
            category: ButtonCategory::Export,
            shortcut: Some("Ctrl+E".to_string()),
            order: 11,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![
                AppMode::Crown, AppMode::Bridge, AppMode::Abutment, 
                AppMode::Bar, AppMode::Telescope, AppMode::BiteSplint,
                AppMode::Model, AppMode::WaxUp
            ],
        },
        
        // Margin
        ButtonDefinition {
            processor: PreparationMargin,
            caption: "Preparation Margin".to_string(),
            tooltip: "Define preparation margin line".to_string(),
            icon: "margin_prep".to_string(),
            category: ButtonCategory::Margin,
            shortcut: Some("M".to_string()),
            order: 20,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge, AppMode::Telescope],
        },
        ButtonDefinition {
            processor: CorrectPreparationMargin,
            caption: "Correct Margin".to_string(),
            tooltip: "Manually correct margin line".to_string(),
            icon: "margin_correct".to_string(),
            category: ButtonCategory::Margin,
            shortcut: None,
            order: 21,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge, AppMode::Telescope],
        },
        ButtonDefinition {
            processor: AutoDetectMargin,
            caption: "Auto Detect Margin".to_string(),
            tooltip: "Automatically detect margin using AI".to_string(),
            icon: "margin_auto".to_string(),
            category: ButtonCategory::Margin,
            shortcut: None,
            order: 22,
            is_advanced: true,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge],
        },
        
        // Abutment
        ButtonDefinition {
            processor: SelectImplantType,
            caption: "Select Implant".to_string(),
            tooltip: "Select implant type from library".to_string(),
            icon: "implant_select".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 30,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![AppMode::Abutment, AppMode::Bar],
        },
        ButtonDefinition {
            processor: AbutmentMarker,
            caption: "Abutment Marker".to_string(),
            tooltip: "Place scan marker".to_string(),
            icon: "abutment_marker".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 31,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![AppMode::Abutment],
        },
        ButtonDefinition {
            processor: EmergenceProfile,
            caption: "Emergence Profile".to_string(),
            tooltip: "Define emergence profile".to_string(),
            icon: "emergence_profile".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 32,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Abutment, AppMode::Bar],
        },
        ButtonDefinition {
            processor: AbutmentBottom,
            caption: "Abutment Bottom".to_string(),
            tooltip: "Generate abutment bottom".to_string(),
            icon: "abutment_bottom".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 33,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Abutment],
        },
        ButtonDefinition {
            processor: InsertionDirection,
            caption: "Insertion Direction".to_string(),
            tooltip: "Set insertion axis".to_string(),
            icon: "insertion_dir".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 34,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Abutment, AppMode::Telescope],
        },
        ButtonDefinition {
            processor: AbutmentEdit,
            caption: "Edit Abutment".to_string(),
            tooltip: "Customize abutment design".to_string(),
            icon: "abutment_edit".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 35,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Abutment],
        },
        ButtonDefinition {
            processor: SetScrewChannel,
            caption: "Screw Channel".to_string(),
            tooltip: "Define screw channel".to_string(),
            icon: "screw_channel".to_string(),
            category: ButtonCategory::Abutment,
            shortcut: None,
            order: 36,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Abutment],
        },
        
        // Anatomy
        ButtonDefinition {
            processor: PlaceModelTooth,
            caption: "Place Model Tooth".to_string(),
            tooltip: "Select tooth from library".to_string(),
            icon: "tooth_place".to_string(),
            category: ButtonCategory::Anatomy,
            shortcut: Some("T".to_string()),
            order: 40,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge],
        },
        ButtonDefinition {
            processor: AdaptToothmodel,
            caption: "Adapt Tooth".to_string(),
            tooltip: "Adapt to antagonist".to_string(),
            icon: "tooth_adapt".to_string(),
            category: ButtonCategory::Anatomy,
            shortcut: None,
            order: 41,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge],
        },
        ButtonDefinition {
            processor: CopyAndPasteTooth,
            caption: "Copy/Mirror".to_string(),
            tooltip: "Copy or mirror tooth".to_string(),
            icon: "tooth_copy".to_string(),
            category: ButtonCategory::Anatomy,
            shortcut: Some("Ctrl+C".to_string()),
            order: 42,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge],
        },
        
        // Crown/Bridge
        ButtonDefinition {
            processor: CrownBottom,
            caption: "Crown Bottom".to_string(),
            tooltip: "Generate crown bottom".to_string(),
            icon: "crown_bottom".to_string(),
            category: ButtonCategory::Crown,
            shortcut: None,
            order: 50,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Crown, AppMode::Bridge],
        },
        ButtonDefinition {
            processor: Connector,
            caption: "Connector".to_string(),
            tooltip: "Design bridge connector".to_string(),
            icon: "connector".to_string(),
            category: ButtonCategory::Bridge,
            shortcut: None,
            order: 51,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Bridge],
        },
        
        // Bar
        ButtonDefinition {
            processor: Bar,
            caption: "Bar Design".to_string(),
            tooltip: "Design bar structure".to_string(),
            icon: "bar_design".to_string(),
            category: ButtonCategory::Bar,
            shortcut: None,
            order: 60,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![AppMode::Bar],
        },
        
        // Telescope
        ButtonDefinition {
            processor: PrimaryTelescope,
            caption: "Telescope".to_string(),
            tooltip: "Create telescope crown".to_string(),
            icon: "telescope".to_string(),
            category: ButtonCategory::Telescope,
            shortcut: None,
            order: 70,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![AppMode::Telescope],
        },
        
        // Freeform
        ButtonDefinition {
            processor: Freeform,
            caption: "Freeform".to_string(),
            tooltip: "Freeform sculpting".to_string(),
            icon: "freeform".to_string(),
            category: ButtonCategory::Freeform,
            shortcut: Some("F".to_string()),
            order: 100,
            is_advanced: false,
            requires_selection: true,
            available_in_modes: vec![
                AppMode::Crown, AppMode::Bridge, AppMode::Abutment,
                AppMode::Bar, AppMode::Telescope, AppMode::BiteSplint
            ],
        },
        
        // Tools
        ButtonDefinition {
            processor: MeasuringPoints,
            caption: "Measure".to_string(),
            tooltip: "Measure distances".to_string(),
            icon: "measure".to_string(),
            category: ButtonCategory::Tools,
            shortcut: Some("Ctrl+R".to_string()),
            order: 120,
            is_advanced: false,
            requires_selection: false,
            available_in_modes: vec![
                AppMode::Crown, AppMode::Bridge, AppMode::Abutment,
                AppMode::Bar, AppMode::Telescope, AppMode::BiteSplint
            ],
        },
    ]
}

impl ButtonConfig {
    /// Get buttons by category
    pub fn get_by_category(&self, category: ButtonCategory) -> Vec<&ButtonDefinition> {
        self.buttons
            .iter()
            .filter(|b| b.category == category)
            .collect()
    }

    /// Get buttons available in mode
    pub fn get_for_mode(&self, mode: AppMode) -> Vec<&ButtonDefinition> {
        self.buttons
            .iter()
            .filter(|b| b.available_in_modes.contains(&mode))
            .collect()
    }

    /// Get button for processor
    pub fn get(&self, processor: ProcessorType) -> Option<&ButtonDefinition> {
        self.buttons.iter().find(|b| b.processor == processor)
    }
}

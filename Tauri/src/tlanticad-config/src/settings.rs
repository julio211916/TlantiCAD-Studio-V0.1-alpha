//! User settings - Replica defaultsettings.xml de Exocad

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Application settings
    pub application: ApplicationSettings,
    
    // Viewport settings
    pub viewport: ViewportSettings,
    
    // UI settings
    pub ui: UiSettings,
    
    // Tool settings
    pub tools: ToolSettings,
    
    // Manufacturing settings
    pub manufacturing: ManufacturingSettings,
    
    // Advanced settings
    pub advanced: AdvancedSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            application: ApplicationSettings::default(),
            viewport: ViewportSettings::default(),
            ui: UiSettings::default(),
            tools: ToolSettings::default(),
            manufacturing: ManufacturingSettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub language: String,
    pub measurement_unit: MeasurementUnit,
    pub auto_save_interval_minutes: u32,
    pub max_recent_projects: usize,
    pub default_project_path: String,
    pub backup_before_save: bool,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            language: "es".to_string(),
            measurement_unit: MeasurementUnit::Millimeters,
            auto_save_interval_minutes: 5,
            max_recent_projects: 20,
            default_project_path: "~/Documents/TlantiCAD/Projects".to_string(),
            backup_before_save: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementUnit {
    Millimeters,
    Inches,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSettings {
    pub background_color_top: String,
    pub background_color_bottom: String,
    pub default_view: ViewType,
    pub show_grid: bool,
    pub grid_size: f64,
    pub show_axes: bool,
    pub default_shading: ShadingMode,
    pub camera_fov: f64,
    pub near_plane: f64,
    pub far_plane: f64,
    pub selection_color: String,
    pub hover_color: String,
    pub margin_line_color: String,
    pub preparation_color: String,
    pub antagonist_color: String,
    pub gingiva_color: String,
}

impl Default for ViewportSettings {
    fn default() -> Self {
        Self {
            background_color_top: "#473a6d".to_string(),
            background_color_bottom: "#473a6d".to_string(),
            default_view: ViewType::Perspective,
            show_grid: true,
            grid_size: 10.0,
            show_axes: true,
            default_shading: ShadingMode::Smooth,
            camera_fov: 45.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            selection_color: "#00ff00".to_string(),
            hover_color: "#ffff00".to_string(),
            margin_line_color: "#ff0000".to_string(),
            preparation_color: "#e8d4c4".to_string(),
            antagonist_color: "#90EE90".to_string(),
            gingiva_color: "#FFB6C1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    Perspective,
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadingMode {
    Smooth,
    Flat,
    Wireframe,
    XRay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: Theme,
    pub font_size: u32,
    pub icon_size: u32,
    pub show_tooltips: bool,
    pub show_status_bar: bool,
    pub sidebar_width: u32,
    pub panel_opacity: f64,
    pub animation_enabled: bool,
    pub language: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            font_size: 12,
            icon_size: 24,
            show_tooltips: true,
            show_status_bar: true,
            sidebar_width: 280,
            panel_opacity: 0.95,
            animation_enabled: true,
            language: "es".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSettings {
    pub default_select_tool: String,
    pub snap_to_grid: bool,
    pub snap_to_vertex: bool,
    pub snap_distance: f64,
    pub brush_size: f64,
    pub brush_strength: f64,
    pub measurement_decimals: u32,
    pub auto_smooth: bool,
}

impl Default for ToolSettings {
    fn default() -> Self {
        Self {
            default_select_tool: "select".to_string(),
            snap_to_grid: false,
            snap_to_vertex: true,
            snap_distance: 0.5,
            brush_size: 2.0,
            brush_strength: 0.5,
            measurement_decimals: 2,
            auto_smooth: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingSettings {
    pub default_milling_machine: String,
    pub default_printer: String,
    pub auto_nest: bool,
    pub add_supports: bool,
    pub default_blank_size: [f64; 3],
    pub output_directory: String,
    pub filename_template: String,
}

impl Default for ManufacturingSettings {
    fn default() -> Self {
        Self {
            default_milling_machine: "Roland DWX-52D".to_string(),
            default_printer: "Formlabs Form 3B".to_string(),
            auto_nest: true,
            add_supports: true,
            default_blank_size: [98.0, 16.0, 20.0],
            output_directory: "~/Documents/TlantiCAD/Output".to_string(),
            filename_template: "{case_number}_{tooth_number}_{design_type}".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub enable_gpu_acceleration: bool,
    pub max_undo_steps: usize,
    pub mesh_decimation_target: f64,
    pub auto_save_meshes: bool,
    pub debug_mode: bool,
    pub log_level: LogLevel,
    pub experimental_features: Vec<String>,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            enable_gpu_acceleration: true,
            max_undo_steps: 50,
            mesh_decimation_target: 0.1,
            auto_save_meshes: true,
            debug_mode: false,
            log_level: LogLevel::Info,
            experimental_features: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

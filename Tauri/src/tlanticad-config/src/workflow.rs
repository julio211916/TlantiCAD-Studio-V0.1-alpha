//! Workflow configuration - Replica wizard.xml de Exocad

use serde::{Deserialize, Serialize};
use tlanticad_core::{ProcessorType, WorkType};

/// Configuración de workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub workflows: Vec<WorkflowDefinition>,
    pub default_workflow: WorkType,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            workflows: default_workflows(),
            default_workflow: WorkType::CrownAnatomic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub work_type: WorkType,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub required_scans: Vec<ScanRequirement>,
    pub available_materials: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub processor: ProcessorType,
    pub priority: i32,
    pub required: bool,
    pub skippable: bool,
    pub dependencies: Vec<String>,
    pub ui_config: StepUiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepUiConfig {
    pub show_in_wizard: bool,
    pub show_in_toolbar: bool,
    pub panel_width: u32,
    pub help_text: String,
}

impl Default for StepUiConfig {
    fn default() -> Self {
        Self {
            show_in_wizard: true,
            show_in_toolbar: true,
            panel_width: 280,
            help_text: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRequirement {
    pub scan_type: ScanType,
    pub required: bool,
    pub multiple_allowed: bool,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Preparation,
    Antagonist,
    Bite,
    Gingiva,
    ScanBody,
    FullArchUpper,
    FullArchLower,
}

fn default_workflows() -> Vec<WorkflowDefinition> {
    use ProcessorType::*;
    use WorkType::*;
    
    vec![
        // Crown Anatomic Workflow
        WorkflowDefinition {
            work_type: CrownAnatomic,
            name: "Anatomic Crown".to_string(),
            description: "Design anatomic crown with full anatomy".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "import".to_string(),
                    name: "Import Scan".to_string(),
                    description: "Import preparation and antagonist scans".to_string(),
                    processor: ImportScan,
                    priority: 10,
                    required: true,
                    skippable: false,
                    dependencies: vec![],
                    ui_config: StepUiConfig {
                        help_text: "Import STL files from your scanner".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "margin".to_string(),
                    name: "Define Margin".to_string(),
                    description: "Draw preparation margin line".to_string(),
                    processor: PreparationMargin,
                    priority: 20,
                    required: true,
                    skippable: false,
                    dependencies: vec!["import".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Click points along the preparation margin".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "tooth".to_string(),
                    name: "Place Tooth".to_string(),
                    description: "Select and position model tooth".to_string(),
                    processor: PlaceModelTooth,
                    priority: 30,
                    required: true,
                    skippable: false,
                    dependencies: vec!["margin".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Choose tooth from library and position it".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "adapt".to_string(),
                    name: "Adapt Tooth".to_string(),
                    description: "Adapt to antagonist and margin".to_string(),
                    processor: AdaptToothmodel,
                    priority: 40,
                    required: true,
                    skippable: false,
                    dependencies: vec!["tooth".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Adjust tooth position and adaptation".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "bottom".to_string(),
                    name: "Crown Bottom".to_string(),
                    description: "Generate crown bottom geometry".to_string(),
                    processor: CrownBottom,
                    priority: 50,
                    required: true,
                    skippable: false,
                    dependencies: vec!["adapt".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Review and adjust crown bottom".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "export".to_string(),
                    name: "Export".to_string(),
                    description: "Export for manufacturing".to_string(),
                    processor: ExportStl,
                    priority: 100,
                    required: true,
                    skippable: false,
                    dependencies: vec!["bottom".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Export STL file for milling".to_string(),
                        ..Default::default()
                    },
                },
            ],
            required_scans: vec![
                ScanRequirement {
                    scan_type: ScanType::Preparation,
                    required: true,
                    multiple_allowed: false,
                    description: "Prepared tooth scan".to_string(),
                },
                ScanRequirement {
                    scan_type: ScanType::Antagonist,
                    required: true,
                    multiple_allowed: false,
                    description: "Antagonist teeth scan".to_string(),
                },
            ],
            available_materials: vec![
                "zirconia".to_string(),
                "lithium_disilicate".to_string(),
                "pmma".to_string(),
            ],
        },
        
        // Custom Abutment Workflow
        WorkflowDefinition {
            work_type: AbutmentCustom,
            name: "Custom Abutment".to_string(),
            description: "Design custom implant abutment".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "import".to_string(),
                    name: "Import Scan".to_string(),
                    description: "Import gingiva and scan body".to_string(),
                    processor: ImportScan,
                    priority: 10,
                    required: true,
                    skippable: false,
                    dependencies: vec![],
                    ui_config: StepUiConfig {
                        help_text: "Import gingiva and scan body STL".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "implant".to_string(),
                    name: "Select Implant".to_string(),
                    description: "Choose implant system and size".to_string(),
                    processor: SelectImplantType,
                    priority: 20,
                    required: true,
                    skippable: false,
                    dependencies: vec!["import".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Select implant from library".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "marker".to_string(),
                    name: "Align Marker".to_string(),
                    description: "Align scan marker with library".to_string(),
                    processor: AbutmentMarker,
                    priority: 30,
                    required: true,
                    skippable: false,
                    dependencies: vec!["implant".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Position marker to match scan".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "emergence".to_string(),
                    name: "Emergence Profile".to_string(),
                    description: "Define emergence profile".to_string(),
                    processor: EmergenceProfile,
                    priority: 40,
                    required: true,
                    skippable: false,
                    dependencies: vec!["marker".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Adjust emergence profile shape".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "bottom".to_string(),
                    name: "Abutment Bottom".to_string(),
                    description: "Generate abutment bottom".to_string(),
                    processor: AbutmentBottom,
                    priority: 50,
                    required: true,
                    skippable: false,
                    dependencies: vec!["emergence".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Review abutment bottom".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "insertion".to_string(),
                    name: "Insertion Direction".to_string(),
                    description: "Set insertion axis".to_string(),
                    processor: InsertionDirection,
                    priority: 60,
                    required: true,
                    skippable: false,
                    dependencies: vec!["bottom".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Define insertion path".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "edit".to_string(),
                    name: "Edit Abutment".to_string(),
                    description: "Customize abutment design".to_string(),
                    processor: AbutmentEdit,
                    priority: 70,
                    required: true,
                    skippable: false,
                    dependencies: vec!["insertion".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Fine-tune abutment parameters".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "screw".to_string(),
                    name: "Screw Channel".to_string(),
                    description: "Define screw channel".to_string(),
                    processor: SetScrewChannel,
                    priority: 80,
                    required: true,
                    skippable: false,
                    dependencies: vec!["edit".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Position screw channel".to_string(),
                        ..Default::default()
                    },
                },
                WorkflowStep {
                    id: "export".to_string(),
                    name: "Export".to_string(),
                    description: "Export for manufacturing".to_string(),
                    processor: ExportStl,
                    priority: 100,
                    required: true,
                    skippable: false,
                    dependencies: vec!["screw".to_string()],
                    ui_config: StepUiConfig {
                        help_text: "Export abutment STL".to_string(),
                        ..Default::default()
                    },
                },
            ],
            required_scans: vec![
                ScanRequirement {
                    scan_type: ScanType::Gingiva,
                    required: true,
                    multiple_allowed: false,
                    description: "Gingiva scan".to_string(),
                },
                ScanRequirement {
                    scan_type: ScanType::ScanBody,
                    required: true,
                    multiple_allowed: false,
                    description: "Scan body position".to_string(),
                },
            ],
            available_materials: vec![
                "titanium".to_string(),
                "peek".to_string(),
                "zirconia".to_string(),
            ],
        },
    ]
}

impl WorkflowConfig {
    /// Get workflow for work type
    pub fn get(&self, work_type: WorkType) -> Option<&WorkflowDefinition> {
        self.workflows.iter().find(|w| w.work_type == work_type)
    }
    
    /// List all available workflows
    pub fn list(&self) -> &[WorkflowDefinition] {
        &self.workflows
    }
}

impl WorkflowDefinition {
    /// Get step by ID
    pub fn get_step(&self, step_id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }
    
    /// Get next step
    pub fn next_step(&self, current_step_id: &str) -> Option<&WorkflowStep> {
        let current_idx = self.steps.iter().position(|s| s.id == current_step_id)?;
        self.steps.get(current_idx + 1)
    }
    
    /// Get previous step
    pub fn previous_step(&self, current_step_id: &str) -> Option<&WorkflowStep> {
        let current_idx = self.steps.iter().position(|s| s.id == current_step_id)?;
        if current_idx > 0 {
            self.steps.get(current_idx - 1)
        } else {
            None
        }
    }
}

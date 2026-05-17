//! Workflow/Wizard system - Replica wizard.xml de Exocad

use serde::{Deserialize, Serialize};
use crate::{ProcessorType, WorkType};

/// Paso del wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub processor: ProcessorType,
    pub priority: i32, // Orden de ejecución
    pub required: bool,
    pub skippable: bool,
    pub dependencies: Vec<String>, // IDs de pasos que deben completarse antes
}

/// Wizard completo para un tipo de trabajo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub work_type: WorkType,
    pub steps: Vec<WizardStep>,
    pub current_step: usize,
    pub completed_steps: Vec<String>,
}

impl Workflow {
    pub fn for_work_type(work_type: WorkType) -> Self {
        let steps = generate_steps_for_work_type(work_type);
        Self {
            work_type,
            steps,
            current_step: 0,
            completed_steps: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<&WizardStep> {
        self.steps.get(self.current_step)
    }

    pub fn next(&mut self) -> Option<&WizardStep> {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            self.steps.get(self.current_step)
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<&WizardStep> {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.steps.get(self.current_step)
        } else {
            None
        }
    }

    pub fn complete_current(&mut self) {
        if let Some(step) = self.current() {
            if !self.completed_steps.contains(&step.id) {
                self.completed_steps.push(step.id.clone());
            }
        }
    }

    pub fn can_proceed(&self) -> bool {
        if let Some(step) = self.current() {
            if step.required {
                return self.completed_steps.contains(&step.id);
            }
        }
        true
    }

    pub fn progress_percent(&self) -> u8 {
        if self.steps.is_empty() {
            return 0;
        }
        let total = self.steps.len();
        let completed = self.completed_steps.len();
        ((completed as f64 / total as f64) * 100.0) as u8
    }
}

fn generate_steps_for_work_type(work_type: WorkType) -> Vec<WizardStep> {
    use ProcessorType::*;
    
    match work_type {
        WorkType::CrownAnatomic => vec![
            WizardStep {
                id: "import".to_string(),
                name: "Import Scan".to_string(),
                description: "Import preparation and antagonist scans".to_string(),
                processor: ImportScan,
                priority: 10,
                required: true,
                skippable: false,
                dependencies: vec![],
            },
            WizardStep {
                id: "margin".to_string(),
                name: "Define Margin".to_string(),
                description: "Draw preparation margin line".to_string(),
                processor: PreparationMargin,
                priority: 20,
                required: true,
                skippable: false,
                dependencies: vec!["import".to_string()],
            },
            WizardStep {
                id: "tooth".to_string(),
                name: "Place Tooth".to_string(),
                description: "Select and place model tooth".to_string(),
                processor: PlaceModelTooth,
                priority: 30,
                required: true,
                skippable: false,
                dependencies: vec!["margin".to_string()],
            },
            WizardStep {
                id: "adapt".to_string(),
                name: "Adapt Tooth".to_string(),
                description: "Adapt to antagonist and margin".to_string(),
                processor: AdaptToothmodel,
                priority: 40,
                required: true,
                skippable: false,
                dependencies: vec!["tooth".to_string()],
            },
            WizardStep {
                id: "bottom".to_string(),
                name: "Crown Bottom".to_string(),
                description: "Generate crown bottom".to_string(),
                processor: CrownBottom,
                priority: 50,
                required: true,
                skippable: false,
                dependencies: vec!["adapt".to_string()],
            },
            WizardStep {
                id: "export".to_string(),
                name: "Export".to_string(),
                description: "Export for manufacturing".to_string(),
                processor: ExportStl,
                priority: 100,
                required: true,
                skippable: false,
                dependencies: vec!["bottom".to_string()],
            },
        ],
        WorkType::AbutmentCustom => vec![
            WizardStep {
                id: "import".to_string(),
                name: "Import Scan".to_string(),
                description: "Import gingiva and scan body".to_string(),
                processor: ImportScan,
                priority: 10,
                required: true,
                skippable: false,
                dependencies: vec![],
            },
            WizardStep {
                id: "implant".to_string(),
                name: "Select Implant".to_string(),
                description: "Select implant type and size".to_string(),
                processor: SelectImplantType,
                priority: 20,
                required: true,
                skippable: false,
                dependencies: vec!["import".to_string()],
            },
            WizardStep {
                id: "marker".to_string(),
                name: "Place Marker".to_string(),
                description: "Align scan marker with library".to_string(),
                processor: AbutmentMarker,
                priority: 30,
                required: true,
                skippable: false,
                dependencies: vec!["implant".to_string()],
            },
            WizardStep {
                id: "emergence".to_string(),
                name: "Emergence Profile".to_string(),
                description: "Define emergence profile".to_string(),
                processor: EmergenceProfile,
                priority: 40,
                required: true,
                skippable: false,
                dependencies: vec!["marker".to_string()],
            },
            WizardStep {
                id: "bottom".to_string(),
                name: "Abutment Bottom".to_string(),
                description: "Generate abutment bottom".to_string(),
                processor: AbutmentBottom,
                priority: 50,
                required: true,
                skippable: false,
                dependencies: vec!["emergence".to_string()],
            },
            WizardStep {
                id: "insertion".to_string(),
                name: "Insertion Direction".to_string(),
                description: "Set insertion axis".to_string(),
                processor: InsertionDirection,
                priority: 60,
                required: true,
                skippable: false,
                dependencies: vec!["bottom".to_string()],
            },
            WizardStep {
                id: "edit".to_string(),
                name: "Edit Abutment".to_string(),
                description: "Customize abutment design".to_string(),
                processor: AbutmentEdit,
                priority: 70,
                required: true,
                skippable: false,
                dependencies: vec!["insertion".to_string()],
            },
            WizardStep {
                id: "screw".to_string(),
                name: "Screw Channel".to_string(),
                description: "Define screw channel".to_string(),
                processor: SetScrewChannel,
                priority: 80,
                required: true,
                skippable: false,
                dependencies: vec!["edit".to_string()],
            },
            WizardStep {
                id: "export".to_string(),
                name: "Export".to_string(),
                description: "Export for manufacturing".to_string(),
                processor: ExportStl,
                priority: 100,
                required: true,
                skippable: false,
                dependencies: vec!["screw".to_string()],
            },
        ],
        WorkType::Bridge => vec![
            WizardStep {
                id: "import".to_string(),
                name: "Import Scan".to_string(),
                description: "Import preparation and antagonist".to_string(),
                processor: ImportScan,
                priority: 10,
                required: true,
                skippable: false,
                dependencies: vec![],
            },
            WizardStep {
                id: "margins".to_string(),
                name: "Define Margins".to_string(),
                description: "Draw margin lines for all abutments".to_string(),
                processor: PreparationMargin,
                priority: 20,
                required: true,
                skippable: false,
                dependencies: vec!["import".to_string()],
            },
            WizardStep {
                id: "teeth".to_string(),
                name: "Place Teeth".to_string(),
                description: "Select and place model teeth".to_string(),
                processor: PlaceModelTooth,
                priority: 30,
                required: true,
                skippable: false,
                dependencies: vec!["margins".to_string()],
            },
            WizardStep {
                id: "connectors".to_string(),
                name: "Add Connectors".to_string(),
                description: "Design bridge connectors".to_string(),
                processor: Connector,
                priority: 50,
                required: true,
                skippable: false,
                dependencies: vec!["teeth".to_string()],
            },
            WizardStep {
                id: "bottom".to_string(),
                name: "Bridge Bottom".to_string(),
                description: "Generate bridge bottom".to_string(),
                processor: CrownBottom,
                priority: 60,
                required: true,
                skippable: false,
                dependencies: vec!["connectors".to_string()],
            },
            WizardStep {
                id: "export".to_string(),
                name: "Export".to_string(),
                description: "Export for manufacturing".to_string(),
                processor: ExportStl,
                priority: 100,
                required: true,
                skippable: false,
                dependencies: vec!["bottom".to_string()],
            },
        ],
        // Más tipos de trabajo...
        _ => vec![
            WizardStep {
                id: "import".to_string(),
                name: "Import Scan".to_string(),
                description: "Import scan data".to_string(),
                processor: ImportScan,
                priority: 10,
                required: true,
                skippable: false,
                dependencies: vec![],
            },
            WizardStep {
                id: "freeform".to_string(),
                name: "Freeform Design".to_string(),
                description: "Design using freeform tools".to_string(),
                processor: Freeform,
                priority: 50,
                required: true,
                skippable: false,
                dependencies: vec!["import".to_string()],
            },
            WizardStep {
                id: "export".to_string(),
                name: "Export".to_string(),
                description: "Export for manufacturing".to_string(),
                processor: ExportStl,
                priority: 100,
                required: true,
                skippable: false,
                dependencies: vec!["freeform".to_string()],
            },
        ],
    }
}

/// Contexto del workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub work_type: WorkType,
    pub current_step_id: String,
    pub completed_steps: Vec<String>,
    pub parameters: serde_json::Value,
}

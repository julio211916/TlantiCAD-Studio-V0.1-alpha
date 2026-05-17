//! Command system for TlantiCAD
//! 
//! Replicas los "processors" de Exocad como commands

use serde::{Deserialize, Serialize};
use crate::{Id, ProcessorType, Result};

/// Comando para ejecutar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum Command {
    // Proyecto
    NewProject { patient_name: String, work_type: String },
    OpenProject { project_id: Id },
    SaveProject,
    CloseProject,
    
    // Import/Export
    ImportScan { file_path: String, scan_type: String },
    ExportStl { design_id: Id, file_path: String },
    
    // Procesadores de diseño
    RunProcessor { processor: ProcessorType, params: serde_json::Value },
    CancelProcessor,
    
    // Margen
    StartMargin { tooth_number: u8 },
    AddMarginPoint { tooth_number: u8, x: f64, y: f64, z: f64 },
    CloseMargin { tooth_number: u8 },
    AutoDetectMargin { tooth_number: u8 },
    
    // Selección
    SelectTooth { tooth_number: u8 },
    SelectDesign { design_id: Id },
    ClearSelection,
    
    // Vista
    SetViewMode { mode: String },
    ResetView,
    FocusSelection,
    
    // Wizard
    WizardNext,
    WizardPrevious,
    WizardGoto { step: String },
}

/// Resultado de un comando
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "snake_case")]
pub enum CommandResult {
    Success { data: serde_json::Value },
    Error { message: String, code: String },
    Progress { percent: u8, message: String },
}

/// Handler de comandos
pub trait CommandHandler: Send + Sync {
    fn can_execute(&self, command: &Command) -> bool;
    fn execute<'a>(&'a self, command: Command) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CommandResult>> + Send + 'a>>;
}

/// Registro de comandos
pub struct CommandRegistry {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn CommandHandler>) {
        self.handlers.push(handler);
    }

    pub async fn execute(&self, command: Command) -> Result<CommandResult> {
        for handler in &self.handlers {
            if handler.can_execute(&command) {
                return handler.execute(command).await;
            }
        }
        Err(crate::TlantiError::Workflow(format!(
            "No handler for command: {:?}",
            command
        )))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

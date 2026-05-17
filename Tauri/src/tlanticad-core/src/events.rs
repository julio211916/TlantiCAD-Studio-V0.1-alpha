//! S7: Event system for TlantiCAD — Full pub/sub with tokio::broadcast

use serde::{Deserialize, Serialize};
use crate::types::{Id, ProcessorType};

/// Eventos del sistema CAD
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum CadEvent {
    // Proyecto
    ProjectCreated { project_id: Id },
    ProjectOpened { project_id: Id },
    ProjectSaved { project_id: Id },
    ProjectClosed { project_id: Id },
    ProjectAutoSaved { project_id: Id },

    // Scans
    ScanImported { scan_id: Id, file_name: String },
    ScanRemoved { scan_id: Id },
    ScanVisibilityChanged { scan_id: Id, visible: bool },
    ScanOpacityChanged { scan_id: Id, opacity: f64 },
    ScanAligned { scan_id: Id },

    // Diseño
    DesignStarted { design_id: Id, processor: ProcessorType },
    DesignCompleted { design_id: Id },
    DesignModified { design_id: Id },
    DesignDeleted { design_id: Id },
    DesignValidated { design_id: Id, valid: bool, issues: Vec<String> },

    // Mesh
    MeshUpdated { mesh_id: Id, vertex_count: usize, triangle_count: usize },
    MeshBooleanCompleted { result_id: Id, op: String },
    MeshDecimated { mesh_id: Id, original_tris: usize, result_tris: usize },

    // Línea de margen
    MarginStarted { tooth_number: u8 },
    MarginPointAdded { tooth_number: u8, point_index: usize },
    MarginCompleted { tooth_number: u8 },
    MarginEdited { tooth_number: u8 },
    MarginAutoDetected { tooth_number: u8, confidence: f64 },

    // Vista
    ViewModeChanged { mode: String },
    CameraMoved { position: [f64; 3], target: [f64; 3] },
    SelectionChanged { selected_ids: Vec<Id> },

    // Wizard
    WizardStepChanged { step: String, previous_step: String },
    WizardStepCompleted { step: String },
    WizardStepSkipped { step: String },

    // Export
    ExportStarted { file_path: String, format: String },
    ExportCompleted { file_path: String },
    ExportFailed { file_path: String, error: String },

    // Undo/Redo
    UndoPerformed { action: String },
    RedoPerformed { action: String },
    HistoryCleared,

    // Plugin
    PluginLoaded { name: String },
    PluginUnloaded { name: String },

    // Errores / Info
    Error { message: String, code: String },
    Warning { message: String },
    Info { message: String },
}

/// S7: Event bus con tokio::broadcast — pub/sub desacoplado
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<CadEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    pub fn emit(&self, event: CadEvent) {
        tracing::debug!("CAD Event: {:?}", event);
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CadEvent> {
        self.tx.subscribe()
    }

    pub fn sender(&self) -> tokio::sync::broadcast::Sender<CadEvent> {
        self.tx.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}
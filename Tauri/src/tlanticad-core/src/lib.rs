//! TlantiCAD Core Module
//!
//! Núcleo del sistema TlantiCAD — OPEN SOURCE, sin restricciones.
//! Event bus, plugin system, command registry, types, errors, workflow.

pub mod types;
pub mod errors;
pub mod events;
pub mod commands;
pub mod workflow;
pub mod plugin;

pub use types::*;
pub use errors::*;
pub use events::*;
pub use commands::*;
pub use workflow::*;
pub use plugin::*;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Versión del sistema
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Nombre del sistema
pub const NAME: &str = "TlantiCAD";

/// Sistema de módulos — TODOS DISPONIBLES (OPEN SOURCE)
/// S6: Expandido con todos los módulos exocad
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Module {
    // Core
    Core,
    Mesh,
    Geometry,
    Io,
    Db,
    Rendering,
    // Dental CAD
    Implant,
    Abutment,
    Crown,
    Bridge,
    Bar,
    Telescope,
    BiteSplint,
    Model,
    WaxUp,
    Freeform,
    Ai,
    // S6: exocad modules
    SmileDesign,
    Articulator,
    GuideCreator,
    PartialCAD,
    BarModule,
    TelescopeModule,
    BiteSplintModule,
    ImplantPlanning,
    ManufacturingExport,
    DicomViewer,
    ChairsideCAD,
    JawMotionImport,
    DentureDesign,
    InlayOnlay,
    Veneer,
    PostCore,
    Endocrown,
    Orthodontic,
}

impl Module {
    /// Todos los módulos — siempre disponibles
    pub fn all() -> Vec<Module> {
        vec![
            Module::Core, Module::Mesh, Module::Geometry, Module::Io, Module::Db,
            Module::Rendering, Module::Implant, Module::Abutment, Module::Crown,
            Module::Bridge, Module::Bar, Module::Telescope, Module::BiteSplint,
            Module::Model, Module::WaxUp, Module::Freeform, Module::Ai,
            Module::SmileDesign, Module::Articulator, Module::GuideCreator,
            Module::PartialCAD, Module::BarModule, Module::TelescopeModule,
            Module::BiteSplintModule, Module::ImplantPlanning,
            Module::ManufacturingExport, Module::DicomViewer, Module::ChairsideCAD,
            Module::JawMotionImport, Module::DentureDesign, Module::InlayOnlay,
            Module::Veneer, Module::PostCore, Module::Endocrown, Module::Orthodontic,
        ]
    }

    /// Nombre del módulo
    pub fn name(&self) -> &'static str {
        match self {
            Module::Core => "Core",
            Module::Mesh => "Mesh Processing",
            Module::Geometry => "Geometry Engine",
            Module::Io => "Import/Export",
            Module::Db => "Database",
            Module::Rendering => "Rendering",
            Module::Implant => "Implant Library",
            Module::Abutment => "Abutment Design",
            Module::Crown => "Crown Design",
            Module::Bridge => "Bridge Design",
            Module::Bar => "Bar Design",
            Module::Telescope => "Telescope Design",
            Module::BiteSplint => "Bite Splint",
            Module::Model => "Model Creator",
            Module::WaxUp => "Wax-Up",
            Module::Freeform => "Freeform Sculpting",
            Module::Ai => "AI/ML Pipeline",
            Module::SmileDesign => "Smile Design",
            Module::Articulator => "Virtual Articulator",
            Module::GuideCreator => "Guide Creator",
            Module::PartialCAD => "Partial CAD",
            Module::BarModule => "Bar Module",
            Module::TelescopeModule => "Telescope Module",
            Module::BiteSplintModule => "Bite Splint Module",
            Module::ImplantPlanning => "Implant Planning",
            Module::ManufacturingExport => "Manufacturing Export",
            Module::DicomViewer => "DICOM Viewer",
            Module::ChairsideCAD => "Chairside CAD",
            Module::JawMotionImport => "Jaw Motion Import",
            Module::DentureDesign => "Denture Design",
            Module::InlayOnlay => "Inlay/Onlay",
            Module::Veneer => "Veneer",
            Module::PostCore => "Post & Core",
            Module::Endocrown => "Endocrown",
            Module::Orthodontic => "Orthodontic",
        }
    }
}

/// S19: Estado global de la aplicación con broadcast
#[derive(Debug)]
pub struct AppState {
    pub project: Arc<RwLock<Option<Project>>>,
    pub modules: Vec<Module>,
    pub version: String,
    pub events: tokio::sync::broadcast::Sender<CadEvent>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(256);
        Self {
            project: Arc::new(RwLock::new(None)),
            modules: Module::all(),
            version: VERSION.to_string(),
            events: tx,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_module_active(&self, module: Module) -> bool {
        self.modules.contains(&module)
    }

    /// Emit event to all subscribers
    pub fn emit(&self, event: CadEvent) {
        let _ = self.events.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CadEvent> {
        self.events.subscribe()
    }
}

/// Información del sistema
pub fn system_info() -> serde_json::Value {
    serde_json::json!({
        "name": NAME,
        "version": VERSION,
        "license": "MIT - Open Source",
        "modules": Module::all().iter().map(|m| {
            serde_json::json!({
                "id": format!("{:?}", m),
                "name": m.name(),
                "available": true,
            })
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "TlantiCAD");
    }

    #[test]
    fn test_module_all_count() {
        let all = Module::all();
        assert!(all.len() > 20);
    }

    #[test]
    fn test_module_name() {
        assert_eq!(Module::Core.name(), "Core");
        assert_eq!(Module::Crown.name(), "Crown Design");
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.modules.is_empty());
        assert!(state.is_module_active(Module::Core));
        assert!(state.is_module_active(Module::Ai));
    }

    #[test]
    fn test_system_info() {
        let info = system_info();
        assert_eq!(info["name"], "TlantiCAD");
        assert!(info["modules"].as_array().unwrap().len() > 10);
    }

    #[test]
    fn test_work_type_display() {
        assert!(!WorkType::CrownAnatomic.display_name().is_empty());
        assert!(!WorkType::Bridge.short_code().is_empty());
    }

    #[test]
    fn test_processor_type_category() {
        let cat = ProcessorType::ImportScan.category();
        assert_eq!(cat, ProcessorCategory::Import);
    }

    #[test]
    fn test_event_bus_default() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe();
        bus.emit(CadEvent::HistoryCleared);
        let ev = rx.try_recv();
        assert!(ev.is_ok());
    }

    #[test]
    fn test_tlanti_error_display() {
        let e = TlantiError::InvalidParameter("test".into());
        let msg = format!("{}", e);
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_project_status_variants() {
        let _s = ProjectStatus::New;
        let _s2 = ProjectStatus::Delivered;
    }
}

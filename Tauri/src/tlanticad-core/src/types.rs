//! Core types for TlantiCAD

use chrono::{DateTime, Utc};
use nalgebra::{Isometry3, Point3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identificador único
pub type Id = Uuid;

/// Tipo de trabajo dental (replica WorkParamsDB de Exocad)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkType {
    CrownAnatomic,
    CrownReduced,
    CrownVeneer,
    CrownInlay,
    CrownOnlay,
    Bridge,
    AbutmentCustom,
    AbutmentStock,
    ScrewRetainedCrown,
    Bar,
    TelescopePrimary,
    TelescopeSecondary,
    BiteSplint,
    Attachment,
    Model,
    WaxUp,
    Diagnostic,
    Provisional,
    PartialFramework,
    FullDenture,
}

impl WorkType {
    pub fn display_name(&self) -> &'static str {
        match self {
            WorkType::CrownAnatomic => "Anatomic Crown",
            WorkType::CrownReduced => "Reduced Crown",
            WorkType::CrownVeneer => "Veneer",
            WorkType::CrownInlay => "Inlay",
            WorkType::CrownOnlay => "Onlay",
            WorkType::Bridge => "Bridge",
            WorkType::AbutmentCustom => "Custom Abutment",
            WorkType::AbutmentStock => "Stock Abutment",
            WorkType::ScrewRetainedCrown => "Screw-Retained Crown",
            WorkType::Bar => "Bar",
            WorkType::TelescopePrimary => "Telescope Primary",
            WorkType::TelescopeSecondary => "Telescope Secondary",
            WorkType::BiteSplint => "Bite Splint",
            WorkType::Attachment => "Attachment",
            WorkType::Model => "Model",
            WorkType::WaxUp => "WaxUp",
            WorkType::Diagnostic => "Diagnostic",
            WorkType::Provisional => "Provisional",
            WorkType::PartialFramework => "Partial Framework",
            WorkType::FullDenture => "Full Denture",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            WorkType::CrownAnatomic => "CRW-A",
            WorkType::CrownReduced => "CRW-R",
            WorkType::CrownVeneer => "VNR",
            WorkType::CrownInlay => "INL",
            WorkType::CrownOnlay => "ONL",
            WorkType::Bridge => "BRG",
            WorkType::AbutmentCustom => "ABT-C",
            WorkType::AbutmentStock => "ABT-S",
            WorkType::ScrewRetainedCrown => "SRC",
            WorkType::Bar => "BAR",
            WorkType::TelescopePrimary => "TEL-P",
            WorkType::TelescopeSecondary => "TEL-S",
            WorkType::BiteSplint => "SPL",
            WorkType::Attachment => "ATT",
            WorkType::Model => "MOD",
            WorkType::WaxUp => "WAX",
            WorkType::Diagnostic => "DIA",
            WorkType::Provisional => "PRV",
            WorkType::PartialFramework => "PART",
            WorkType::FullDenture => "FULL",
        }
    }

    pub fn available_processors(&self) -> Vec<ProcessorType> {
        use ProcessorType::*;
        match self {
            WorkType::CrownAnatomic | WorkType::CrownReduced => vec![
                PreparationMargin, PlaceModelTooth, AdaptToothmodel,
                CrownBottom, Connector, Freeform,
            ],
            WorkType::CrownVeneer => vec![
                PreparationMargin, PlaceModelTooth, AdaptToothmodel,
                CrownBottom, Freeform,
            ],
            WorkType::CrownInlay => vec![
                PreparationMargin, InlayBottom, OffsetInlay, Freeform,
            ],
            WorkType::CrownOnlay => vec![
                PreparationMargin, InlayBottom, OffsetInlay, CrownBottom, Freeform,
            ],
            WorkType::Bridge => vec![
                PreparationMargin, PlaceModelTooth, AdaptToothmodel,
                CrownBottom, Connector, CopyAndPasteTooth, Freeform,
            ],
            WorkType::AbutmentCustom => vec![
                SelectImplantType, AbutmentMarker, EmergenceProfile,
                AbutmentBottom, InsertionDirection, AbutmentEdit, SetScrewChannel,
            ],
            WorkType::ScrewRetainedCrown => vec![
                SelectImplantType, PlaceModelTooth, SetScrewChannel,
                CrownBottom, Freeform,
            ],
            WorkType::Bar => vec![
                SelectImplantType, EmergenceProfile, AbutmentBottom, Bar,
            ],
            WorkType::TelescopePrimary => vec![
                PreparationMargin, PrimaryTelescope, TelescopeInsertionDirection,
                FreeformTelescope,
            ],
            WorkType::BiteSplint => vec![
                BiteSplintBottom, BiteSplintTop, FreeformBiteSplint,
            ],
            WorkType::Model => vec![
                ModelAlignment, ModelSegmentation, ModelDieWithBore,
            ],
            WorkType::WaxUp => vec![
                WaxUp, VirtualWaxUpBottom, VirtualWaxUpBase,
            ],
            _ => vec![Freeform],
        }
    }
}

/// Tipos de procesadores/herramientas (replica buttons.xml de Exocad)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessorType {
    // Import/Export
    ImportScan,
    ImportDicom,
    ExportStl,
    ExportObj,
    
    // Margin
    PreparationMargin,
    CorrectPreparationMargin,
    AutoDetectMargin,
    VirtualPreparationMargin,
    
    // Implant/Abutment
    SelectImplantType,
    AbutmentMarker,
    EmergenceProfile,
    AbutmentBottom,
    InsertionDirection,
    AbutmentEdit,
    SetScrewChannel,
    ScrewHoleDesign,
    
    // Anatomy
    PlaceModelTooth,
    LoadCustomToothmodel,
    LoadBridgeModels,
    AdaptToothmodel,
    CopyAndPasteTooth,
    MirrorHealthyToSitu,
    CorrectPlacement,
    
    // Crown/Bridge
    CrownBottom,
    ProvisionalCrownTop,
    Connector,
    DeleteConnector,
    
    // Bar
    Bar,
    FreeformBar,
    ShrinkBarSegmentTooth,
    
    // Telescope
    PrimaryTelescope,
    TelescopeInsertionDirection,
    FreeformTelescope,
    ShrinkTelescopes,
    
    // Inlay
    InlayBottom,
    OffsetInlay,
    OffsetCoping,
    LingualBand,
    
    // Bite Splint
    BiteSplintBottom,
    BiteSplintTop,
    FreeformBiteSplint,
    
    // WaxUp
    WaxUp,
    VirtualWaxUpBottom,
    VirtualWaxUpBase,
    
    // Gingiva
    ShrinkGingiva,
    FreeformGingiva,
    AdjustingSitu,
    
    // Model
    ModelAlignment,
    ModelSegmentation,
    ModelDieWithBore,
    
    // Freeform
    Freeform,
    Shrink,
    FreeformScanData,
    
    // Tools
    MeasuringPoints,
    AnalyzeMesh,
    SectionView,
    MergeParts,
    DeleteReconstructions,
    MinThickness,
    OverPress,
}

impl ProcessorType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProcessorType::ImportScan => "Import Scan",
            ProcessorType::ImportDicom => "Import DICOM",
            ProcessorType::ExportStl => "Export STL",
            ProcessorType::ExportObj => "Export OBJ",
            ProcessorType::PreparationMargin => "Preparation Margin",
            ProcessorType::CorrectPreparationMargin => "Correct Margin",
            ProcessorType::AutoDetectMargin => "Auto Detect Margin",
            ProcessorType::VirtualPreparationMargin => "Virtual Prep Margin",
            ProcessorType::SelectImplantType => "Select Implant",
            ProcessorType::AbutmentMarker => "Abutment Marker",
            ProcessorType::EmergenceProfile => "Emergence Profile",
            ProcessorType::AbutmentBottom => "Abutment Bottom",
            ProcessorType::InsertionDirection => "Insertion Direction",
            ProcessorType::AbutmentEdit => "Edit Abutment",
            ProcessorType::SetScrewChannel => "Screw Channel",
            ProcessorType::ScrewHoleDesign => "Screw Hole Design",
            ProcessorType::PlaceModelTooth => "Place Model Tooth",
            ProcessorType::LoadCustomToothmodel => "Custom Tooth Model",
            ProcessorType::LoadBridgeModels => "Bridge Models",
            ProcessorType::AdaptToothmodel => "Adapt Tooth",
            ProcessorType::CopyAndPasteTooth => "Copy/Mirror Tooth",
            ProcessorType::MirrorHealthyToSitu => "Mirror Healthy",
            ProcessorType::CorrectPlacement => "Correct Placement",
            ProcessorType::CrownBottom => "Crown Bottom",
            ProcessorType::ProvisionalCrownTop => "Provisional Crown",
            ProcessorType::Connector => "Connector",
            ProcessorType::DeleteConnector => "Delete Connector",
            ProcessorType::Bar => "Bar Design",
            ProcessorType::FreeformBar => "Freeform Bar",
            ProcessorType::ShrinkBarSegmentTooth => "Shrink Bar Segment",
            ProcessorType::PrimaryTelescope => "Telescope",
            ProcessorType::TelescopeInsertionDirection => "Telescope Direction",
            ProcessorType::FreeformTelescope => "Freeform Telescope",
            ProcessorType::ShrinkTelescopes => "Shrink Telescopes",
            ProcessorType::InlayBottom => "Inlay Bottom",
            ProcessorType::OffsetInlay => "Offset Inlay",
            ProcessorType::OffsetCoping => "Offset Coping",
            ProcessorType::LingualBand => "Lingual Band",
            ProcessorType::BiteSplintBottom => "Splint Bottom",
            ProcessorType::BiteSplintTop => "Splint Top",
            ProcessorType::FreeformBiteSplint => "Freeform Splint",
            ProcessorType::WaxUp => "WaxUp",
            ProcessorType::VirtualWaxUpBottom => "WaxUp Bottom",
            ProcessorType::VirtualWaxUpBase => "WaxUp Base",
            ProcessorType::ShrinkGingiva => "Shrink Gingiva",
            ProcessorType::FreeformGingiva => "Freeform Gingiva",
            ProcessorType::AdjustingSitu => "Adjust Situ",
            ProcessorType::ModelAlignment => "Align Model",
            ProcessorType::ModelSegmentation => "Segment Model",
            ProcessorType::ModelDieWithBore => "Die with Bore",
            ProcessorType::Freeform => "Freeform",
            ProcessorType::Shrink => "Shrink",
            ProcessorType::FreeformScanData => "Edit Scan Data",
            ProcessorType::MeasuringPoints => "Measure",
            ProcessorType::AnalyzeMesh => "Analyze",
            ProcessorType::SectionView => "Section View",
            ProcessorType::MergeParts => "Merge Parts",
            ProcessorType::DeleteReconstructions => "Delete All",
            ProcessorType::MinThickness => "Min Thickness",
            ProcessorType::OverPress => "Overpress",
        }
    }

    pub fn category(&self) -> ProcessorCategory {
        use ProcessorCategory::*;
        match self {
            ProcessorType::ImportScan | ProcessorType::ImportDicom => Import,
            ProcessorType::ExportStl | ProcessorType::ExportObj => Export,
            ProcessorType::PreparationMargin | ProcessorType::CorrectPreparationMargin |
            ProcessorType::AutoDetectMargin | ProcessorType::VirtualPreparationMargin => Margin,
            ProcessorType::SelectImplantType | ProcessorType::AbutmentMarker |
            ProcessorType::EmergenceProfile | ProcessorType::AbutmentBottom |
            ProcessorType::InsertionDirection | ProcessorType::AbutmentEdit |
            ProcessorType::SetScrewChannel | ProcessorType::ScrewHoleDesign => Abutment,
            ProcessorType::PlaceModelTooth | ProcessorType::LoadCustomToothmodel |
            ProcessorType::LoadBridgeModels | ProcessorType::AdaptToothmodel |
            ProcessorType::CopyAndPasteTooth | ProcessorType::MirrorHealthyToSitu |
            ProcessorType::CorrectPlacement => Anatomy,
            ProcessorType::CrownBottom | ProcessorType::ProvisionalCrownTop => Crown,
            ProcessorType::Connector | ProcessorType::DeleteConnector => Bridge,
            ProcessorType::Bar | ProcessorType::FreeformBar | ProcessorType::ShrinkBarSegmentTooth => Bar,
            ProcessorType::PrimaryTelescope | ProcessorType::TelescopeInsertionDirection |
            ProcessorType::FreeformTelescope | ProcessorType::ShrinkTelescopes => Telescope,
            ProcessorType::InlayBottom | ProcessorType::OffsetInlay |
            ProcessorType::OffsetCoping | ProcessorType::LingualBand => Inlay,
            ProcessorType::BiteSplintBottom | ProcessorType::BiteSplintTop |
            ProcessorType::FreeformBiteSplint => BiteSplint,
            ProcessorType::WaxUp | ProcessorType::VirtualWaxUpBottom |
            ProcessorType::VirtualWaxUpBase => WaxUp,
            ProcessorType::ShrinkGingiva | ProcessorType::FreeformGingiva |
            ProcessorType::AdjustingSitu => Gingiva,
            ProcessorType::ModelAlignment | ProcessorType::ModelSegmentation |
            ProcessorType::ModelDieWithBore => Model,
            ProcessorType::Freeform | ProcessorType::Shrink | ProcessorType::FreeformScanData => Freeform,
            _ => Tools,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessorCategory {
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
}

/// Proyecto dental (replica DentalDB de Exocad)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Id,
    pub case_number: String,
    pub patient_name: String,
    pub dentist: String,
    pub clinic: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub work_type: WorkType,
    pub teeth: Vec<Tooth>,
    pub materials: Vec<MaterialAssignment>,
    pub scans: Vec<Scan>,
    pub designs: Vec<Design>,
    pub status: ProjectStatus,
    pub notes: String,
    pub technician: Option<String>,
    pub is_deleted: bool,
    pub global_shade: Option<String>,
    pub antagonist_scan_mode: Option<String>,
    pub is_imported: bool,
}

/// Estado del proyecto
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    New,
    ScanImported,
    MarginDefined,
    InDesign,
    DesignComplete,
    ReadyForManufacturing,
    Manufactured,
    Delivered,
}

/// Diente en el proyecto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tooth {
    pub number: u8,
    pub name: String,
    pub is_present: bool,
    pub is_prepared: bool,
    pub preparation_margin: Option<MarginLine>,
    pub antagonist: Option<u8>,
    pub design: Option<Design>,
}

/// Línea de margen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginLine {
    pub points: Vec<Point3<f64>>,
    pub is_closed: bool,
    pub confidence: f64, // 0.0 - 1.0 para detección automática
}

/// Asignación de material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAssignment {
    pub tooth_number: u8,
    pub material: Material,
    pub shade: String,
}

/// Material dental
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub material_type: MaterialType,
    pub manufacturer: String,
    pub available_shades: Vec<String>,
    pub milling_params: MillingParameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    Zirconia,
    LithiumDisilicate,
    Pmma,
    Peek,
    Titanium,
    CobaltChrome,
    Composite,
    Wax,
}

/// Parámetros de fresado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MillingParameters {
    pub tool_diameter: f64,
    pub spindle_speed: u32,
    pub feed_rate: f64,
    pub step_down: f64,
}

/// Scan importado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: Id,
    pub scan_type: ScanType,
    pub file_path: String,
    pub transformation: Isometry3<f64>,
    pub is_visible: bool,
    pub opacity: f64,
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
    ModelUpper,
    ModelLower,
}

/// Diseño CAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    pub id: Id,
    pub name: String,
    pub design_type: DesignType,
    pub tooth_number: u8,
    pub mesh_id: Option<Id>,
    pub parameters: DesignParameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignType {
    Crown,
    Abutment,
    BridgeConnector,
    Bar,
    Telescope,
    Inlay,
    BiteSplint,
    WaxUp,
}

/// Parámetros de diseño
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignParameters {
    pub min_thickness: f64,
    pub cement_gap: f64,
    pub extra_spacing: f64,
    pub margin_chamfer: f64,
}

impl Default for DesignParameters {
    fn default() -> Self {
        Self {
            min_thickness: 0.4,
            cement_gap: 0.05,
            extra_spacing: 0.02,
            margin_chamfer: 0.2,
        }
    }
}

/// Paciente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Id,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub patient_id: String,
    pub notes: String,
}

/// Doctor/Dentista
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dentist {
    pub id: Id,
    pub name: String,
    pub clinic: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

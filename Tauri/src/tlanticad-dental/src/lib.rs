//! TlantiCAD Dental — Unified dental-specific algorithms
//!
//! Segmentation, margin detection, insertion axis, occlusion analysis,
//! manufacturing constraints, parametric tooth design, notation,
//! undercut visualization, dynamic occlusion, tooth library, scan import,
//! model creation, partial restorations, dentures, bar/telescope,
//! bite splints, workflow automation, smile design, virtual articulator,
//! advanced occlusion, surgical guides, removable prosthetics,
//! orthodontics, maxillofacial, shade matching, and quality validation.

pub mod segmentation;
pub mod margin;
pub mod insertion;
pub mod occlusion;
pub mod manufacturing;
pub mod parametric;
pub mod notation;
pub mod undercut_vis;
pub mod occlusal_compass;
pub mod antagonist;
pub mod dynamic_occlusion;
pub mod margin_editor;
pub mod tooth_library;
pub mod scan_import;
pub mod model_creation;
pub mod partial_restoration;
pub mod denture;
pub mod bar_telescope;
pub mod bite_splint;
pub mod automation;
// S251-S300: Dental Avanzado
pub mod smile_design;
pub mod virtual_articulator;
pub mod advanced_occlusion;
pub mod surgical_guide;
pub mod removable_prosthetics;
pub mod orthodontic;
pub mod maxillofacial;
pub mod shade_matching;
pub mod quality_validation;

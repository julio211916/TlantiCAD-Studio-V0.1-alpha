//! TlantiCAD Implant Module
//!
//! Implant library, positioning, collision detection, surgical planning,
//! and abutment interface management.

pub mod library;
pub mod positioning;
pub mod collision;
pub mod planning;
pub mod abutment_interface;
pub mod motor;

// AR-V373 — implant manager (change-type, delete, references, validation)
pub mod manager;

// AR-V380 — fixation / surgical guide
pub mod fixation_guide;

// AR-V388 — reference-objects definer for implant planning
pub mod reference_definer;

// AR-V389 — mesh edits for implant planning (trim / weld / drop floats)
pub mod planning_edit;

// AR-V390 — implant placement ↔ tooth-chart linker
pub mod tooth_linker;

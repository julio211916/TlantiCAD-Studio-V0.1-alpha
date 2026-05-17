//! TlantiCAD Denture — Duplicate Denture workflow.
//!
//! Ports: `DentalProcessors/DuplicateDentureMarginProcessor`,
//! `DuplicateDentureToothSegmentationProcessor`,
//! `DuplicateDentureMeshEditProcessor`.
//!
//! AR-V399.

pub mod duplicate;

// AR-V415 — denture scan freeform classification (tooth/flange/base/impression).
pub mod scan_freeform;

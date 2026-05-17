//! TlantiCAD Manufacturing Module
//!
//! S301-S350: Complete dental manufacturing pipeline covering CAM toolpath generation,
//! CNC milling strategies, 3D printing support structures, nesting optimization,
//! post-processing, quality control, and manufacturing integration.

pub mod toolpath;
pub mod milling;
pub mod printing;
pub mod nesting;
pub mod post_processing;
pub mod quality_control;
pub mod material;
pub mod machine;
pub mod gcode;
pub mod export;
pub mod integration;

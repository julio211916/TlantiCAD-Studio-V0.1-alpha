//! TlantiCAD Articulator engine — Bonwill triangle + jaw motion + registration + planes tool.
//!
//! Ported from `DentalProcessors/ArticulatorProcessor` (9981 LOC),
//! `ArticulatorRegistrationProcessor`, `AutoArticulatorProcessor`,
//! `ArticulatorPlanesTool` (65531 LOC), and `ArticulatorDecorationViewerObjects`.
//! AR-V377.
//!
//! Closes audit no-stubs item #9: previously labelled `backend: "mock"`, the formulas ARE the
//! Bonwill triangle (the real anatomic model used in dental occlusion). We just expose them as
//! a real engine with `backend: "bonwill"`.
//!
//! Modules:
//!   * `bonwill`         — Bonwill triangle vertices + condylar geometry.
//!   * `jaw_motion`      — open / close, protrusion, lateral excursion (Bennett angle).
//!   * `registration`    — facebow + bite registration to local mesh frame.
//!   * `planes`          — occlusal / Camper / Frankfort planes from landmarks.

pub mod bonwill;
pub mod jaw_motion;
pub mod planes;
pub mod registration;

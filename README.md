TlantiCAD Studio V0.1-alpha - Complete README Overview
Project Overview
TlantiCAD Studio is a comprehensive open-source dental CAD application combining 3D reconstruction, segmentation, and design capabilities for clinical dental professionals.

Architecture
Multi-Layered Stack
Code
React UI → Tauri Commands → Rust Orchestrator → Python Backend → Artifacts → Case Manifest
Frontend: React/TypeScript/Vue UI
Desktop Framework: Tauri (Rust)
Backend:
Rust: Case storage, asset vault, CAD jobs, clinical artifacts
Python 3.11+: DICOM processing (pydicom), AI inference (ONNX/PyTorch), volume processing (SimpleITK/VTK)
Storage: SQLite local database + case folder vault
Core Modules (11 Specialized Components)
CAD Core - Mesh-first restorative design
DICOM Core - CBCT/DICOM processing
Implant Core - Implant planning & abutment design
Surgical Guide - Sleeves, guides, drill protocols
Splint & Articulator - Bite splints, jaw motion
Cephalometrics - Landmarks, tracing, reports
Aligners - Tooth setup, staging, IPR, trays
Ortho - Smile design, waxup handoff
Model Creator - Printable models, bases, dies
Partials - Survey, framework, clasps
Manufacturing - Validation, CAM handoff
Development Status (as of April 30, 2026)
Active App: Tauri/Cargo.toml
Active Workspace: Tauri/src/Cargo.toml
Quarantine: Legacy modules awaiting migration
Target Architecture: 27+ specialized crates (geometry, mesh, CSG, formats, AI runtime, compute router)
Key Design Principles
✅ Local-only backend - All data stays on user's machine
✅ Role-based permissions - Module-level access control
✅ Job tracking - Async operations with progress monitoring
✅ Asset vault - Structured storage for meshes, DICOM, reports
✅ Workflow-driven - 6-step standardized pipelines (import → clean → segment → design → validate → export)

Language Composition
TypeScript: 26.5%
Python: 26.3% (AI/ML, DICOM)
Rust: 20.7% (Performance-critical operations)
JavaScript: 7.8%
Vue: 5.3%
C++: 4.9%
Other: 8.5%

# TlantiCAD Module Map

| Module | Source roots | Runtime contract |
| --- | --- | --- |
| Imaging | `frontend/io`, `frontend/composables`, `frontend/workspaces/tlanti-cad/features/dicom-viewer`, `Tauri/backend/python/trame_slicer_sidecar.py` | Local DICOM/VTK/trame-slicer viewing, MPR, volume rendering, segmentation preview, mesh handoff |
| CAD | `frontend/workspaces/tlanti-cad`, `frontend/core`, `Tauri/src/tlanticad-*` | Dental panels call typed Tauri commands and Rust crates |
| Case Management | `frontend/workspaces/tlanti-db`, `frontend/core/ports`, `Tauri/src/case_*` | Patient, case, required assets, module launch and history |
| Mesh Engine | `Tauri/meshlib`, `Tauri/meshlib-wasm`, `frontend/io/vtk`, `Tauri/src/mesh`, `Tauri/src/tlanticad-geometry` | MeshLib repair/boolean/offset, VTK volume bridge, Rust algebra for bounds/normals/transforms |
| Asset Library | `frontend/core/use-cases/mesh-vault-import-use-case.ts`, `Tauri/src/mesh_vault.rs`, `Tauri/library` | Path-backed STL/OBJ/PLY/DICOM import with manifest/progress |
| AI Sidecar | `Tauri/backend/app`, `Tauri/backend/modules`, `Tauri/backend/python` | Offline FastAPI, pydicom, ONNX/Torch readiness |
| Runtime Diagnostics | `frontend/scripts`, `Tauri/src/system_runtime.rs`, `Tauri/src/lib.rs` | Gate phases, runtime info and module catalog |
| Next/Tauri/FastAPI Bridge | `frontend/app`, `frontend/runtime`, `Tauri/src/lib.rs`, `Tauri/backend/app/main.py` | Local webview shell, typed backend endpoint command, FastAPI local health/CORS contract |

## Pending Hardening

- Keep legacy VolView handlers out of the required runtime path; the clinical route uses `DicomClinicalWorkspace` plus the trame-slicer sidecar.
- Replace remaining full-buffer imports in source roots with path handles and chunked jobs.
- Promote MeshLib WASM operations into dedicated workers with transferable ArrayBuffers only after Mesh Vault has produced bounded handles.
- Add Rust job queue cancellation and bounded concurrency to CAD/DICOM heavy commands.
- Add GPU lifecycle tests for Three/VTK viewers: dispose geometry, textures, render targets and workers.
- Add Python healthcheck execution with model manifest hash once local ONNX assets are installed.
- Split Next release bundles so TlantiDB, CAD, DICOM, AI and mesh codecs compile as isolated chunks instead of one large production graph.

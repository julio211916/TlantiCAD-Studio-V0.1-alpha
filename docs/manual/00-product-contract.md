# TlantiCAD Product Contract

TlantiCAD Studio is an offline dental workstation. The active tree is `frontend/` plus `Tauri/`; no production workflow may depend on `app/tlanticad`.

## Clinical Shell

- `TlantiDB` owns case creation, workload selection, required assets and case search.
- `TlantiWorkspacePreloader` owns transitions between case management and clinical modules.
- `TlantiCAD` owns CAD/DICOM workspaces and receives `caseId` plus canonical `moduleId`.

## Runtime Split

- React: UI, state selection, metadata and progress only.
- Three/VTK: rendering only.
- Tauri commands: local orchestration, file handles, job handles and security boundary.
- Rust: mesh/CAD core and path-stream processing.
- Python/FastAPI: DICOM, AI and model-backed jobs on loopback only.

## Performance Rule

Clinical assets move as paths, manifests and handles. React must not own full CBCT/STL/OBJ/PLY buffers for production workflows.

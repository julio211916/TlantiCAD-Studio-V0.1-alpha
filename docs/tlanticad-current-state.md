# TlantiCAD Current State

## Technical Analysis

The repository contains strong existing material that was running in parallel: VolView/VTK imaging roots, TlantiCAD CAD workspace, TlantiDB case management, Mesh Vault imports, typed IPC, Rust dental crates and the Python DICOM/AI sidecar.

The active contract is now the original `frontend/` + `Tauri/` tree. It does not discard those roots. It maps the real `frontend/workspaces` surfaces into one desktop workstation surface through Next.js 16.2.4 and keeps the existing source roots available as engines, modules and adapters.

## Problems Detected

- The Rust workspace had `tlanticad-db` pointing to a missing folder. It now resolves to the real `tlanticad-workspace` crate path.
- The active build root was moved back into `frontend/` so the project keeps its original folder shape: Next.js, TypeScript and React live in `frontend`, while Tauri remains in `Tauri`.
- The frontend shell was disconnected from the real workspaces. The Next app now renders `frontend/workspaces/TlantiWorkspacePreloader`, `frontend/workspaces/tlanti-db/TlantiDbWorkspace` and `frontend/workspaces/tlanti-cad/TlantiCadWorkspace`.
- Mesh Vault still had a direct Tauri invoke bypass. It now routes through `frontend/lib/ipc`.
- VolView/VTK existed as imaging capability but did not have a small core use-case contract. `ImagingWorkflowUseCase` now connects local imaging validation to Mesh Vault path-backed import.
- Recovery checks were not a product contract. `recovery:gate` now verifies source roots, frontend, IPC, Tauri, Python, assets and runtime.

## Performance Notes

- DICOM/STL/OBJ/PLY must stay path-backed. React should receive manifests, previews and progress only.
- The assets phase is intentionally isolated so normal source/frontend diagnostics do not traverse large dental libraries.
- The active app lazy-loads the imaging runtime panel and keeps the module map as small JSON-like metadata.
- The Next dev route currently compiles the real workspaces in about 42s on first hit; the app shell code path itself reported 320ms after the heavy dependency graph compiled.

## Bottlenecks

- Existing viewers still need deeper GPU cleanup: geometry disposal, texture lifecycle, culling and LOD.
- Existing frontend roots still contain remote references that the gate reports as warnings for hardening.
- Production `next build --webpack` still exceeds an 8 GB heap and remains heavy even at 12 GB. The next engineering slice should split CAD/DICOM/AI into route-level and interaction-level chunks before treating release packaging as done.
- Full compile of all dental Rust crates is separate from the active desktop contract and should be staged after metadata is consistently clean.

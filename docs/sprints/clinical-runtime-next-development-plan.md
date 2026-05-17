# Clinical Runtime Next Development Plan

Updated: 2026-05-01

## Closed In This Slice

- Slicer runtime is packaged outside the frontend render path.
- SlicerAutomatedDentalTools is registered as a local extension.
- Clinical models are resolved through the model manifest and local cache.
- FastAPI/trame sidecar exposes runtime, model and clinical job endpoints.
- Tauri IPC exposes typed runtime/model/job start/status/cancel commands.
- DICOM Clinical Workspace starts Slicer jobs, polls status, shows logs, reports artifacts and can cancel an active job.
- Bundle resources no longer include `resources/slicer/**/*`; cache/download/job folders are excluded from Tauri build scanning.
- Gates passed: Python compile, frontend typecheck, DICOM/AI QA, recovery gate, bundle budget and cargo check.

## Systematic Debugging Findings

- Root cause 1: clinical jobs existed as API contract, but `start_job` did not launch mapped headless workflows. Fix: map supported workflows to Slicer CLI or nnUNet commands and persist job state.
- Root cause 2: failed jobs could leave state stuck in `queued` because artifact manifest writing assumed an existing output directory. Fix: create the output directory before writing artifacts.
- Root cause 3: Tauri build script stalled because the bundle resource glob scanned 39 GB of Slicer model cache/download data. Fix: bundle only runtime, extension and manifest; models stay in the downloader-managed cache.

## Remaining Clinical Work

- Add a small licensed/public CBCT or NIfTI smoke fixture with checksum.
- Run first real adult dental segmentation end-to-end on that fixture and register `.seg.nrrd`/mesh outputs.
- Add workflow-specific option forms for AReg T2 moving scan, IOS mesh inputs and CLI-C params.
- Decide whether AMASSS should be used through its legacy model layout or replaced by DentalSegmentator for the default CBCT segmentation path.
- Move model cache from repo resources into an app data cache for production packaging.
- Add packaging/security recovery phases for macOS signing, capabilities and resource size budgets.

## Next 6 Sprints

1. S031 Strict Sidecar Recovery: health, logs, model status and job state must pass without frontend boot.
2. S032 Real Fixture Smoke: public fixture download, checksum, DICOM/NIfTI conversion, one real segmentation job.
3. S033 Artifact Registry: persist clinical outputs as CAD-ready handles and surface them in Mesh Vault.
4. S034 Workflow Forms: collect required options for ALI, ASO, AReg and CLI-C instead of generic JSON.
5. S035 Packaging Budget: app bundle size gate, external model cache, resource allowlist and runtime relocation.
6. S036 Clinical UX Hardening: progress taxonomy, cancellation semantics, logs drawer, artifact preview and retry actions.

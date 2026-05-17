# Offline Clinical Contract

TlantiCAD is offline-first. Network access is restricted to local loopback sidecars.

## Allowed

- `127.0.0.1`, `localhost` and `::1` for supervised local services.
- Local filesystem access through Tauri commands and explicit capabilities.
- Local Python, ONNX Runtime, pydicom, VTK/SimpleITK and Rust jobs.

## Blocked

- Remote HTTP APIs for clinical workflows.
- Cloud model inference for DICOM, STL, OBJ, PLY, PHI or case metadata.
- Browser-side full-buffer DICOM/mesh processing as a production path.

## Model Contract

Every clinical model must have a local manifest with model id, hash, license, modality, expected input handle and output artifact contract.

# TlantiCAD Local Dental Backend

This backend is local-only. It mirrors a modern FastAPI/ML service layout while respecting the desktop product boundary:

- Tauri starts and supervises the local Python sidecar.
- React calls Tauri commands or local loopback endpoints only.
- Python owns DICOM metadata, pydicom anonymization, SimpleITK/VTK volume processing and ONNX/PyTorch inference adapters.
- Rust owns case storage, asset vault, CAD jobs and clinical artifact manifests.

Runtime path:

```text
React UI -> Tauri command -> Rust orchestrator -> Python local backend/modules -> Rust artifact manifest -> React handle
```

Clinical DICOM path:

```text
DICOM -> pydicom metadata/PII -> SimpleITK/VTK volume -> segmentation/inference -> mesh/artifact -> case manifest
```

Use Python 3.11+:

```bash
python3.11 -m venv .tlanticad/python/.venv
.tlanticad/python/.venv/bin/python -m pip install -r backend/python/requirements.txt
```

#!/usr/bin/env python3
import json
import sys
from pathlib import Path

import pydicom
import vtk


def main() -> int:
    if len(sys.argv) != 2:
        print(json.dumps({"ok": False, "error": "usage: dicom_healthcheck.py <dicom-path>"}))
        return 2

    dicom_path = Path(sys.argv[1])
    if not dicom_path.exists():
        print(json.dumps({"ok": False, "error": f"missing file: {dicom_path}"}))
        return 2

    dataset = pydicom.dcmread(str(dicom_path), stop_before_pixels=True)
    file_meta = getattr(dataset, "file_meta", None)
    payload = {
        "ok": True,
        "engine": "pydicom",
        "pydicomVersion": pydicom.__version__,
        "vtkVersion": vtk.vtkVersion.GetVTKVersion(),
        "path": str(dicom_path),
        "patientId": str(getattr(dataset, "PatientID", "")),
        "patientNamePresent": bool(getattr(dataset, "PatientName", "")),
        "modality": str(getattr(dataset, "Modality", "")),
        "studyInstanceUid": str(getattr(dataset, "StudyInstanceUID", "")),
        "sopClassUid": str(getattr(file_meta, "MediaStorageSOPClassUID", "")),
        "transferSyntaxUid": str(getattr(file_meta, "TransferSyntaxUID", "")),
        "rows": int(getattr(dataset, "Rows", 0) or 0),
        "columns": int(getattr(dataset, "Columns", 0) or 0),
        "hasPixelData": "PixelData" in dataset,
    }
    print(json.dumps(payload, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

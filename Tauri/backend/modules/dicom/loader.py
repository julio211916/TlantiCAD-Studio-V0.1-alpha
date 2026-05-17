from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pydicom


@dataclass(frozen=True)
class DicomRuntimeStatus:
    pydicom_version: str
    simpleitk_available: bool
    vtk_available: bool


def _module_available(name: str) -> bool:
    try:
        __import__(name)
        return True
    except Exception:
        return False


def get_dicom_runtime_status() -> DicomRuntimeStatus:
    return DicomRuntimeStatus(
        pydicom_version=pydicom.__version__,
        simpleitk_available=_module_available("SimpleITK"),
        vtk_available=_module_available("vtk"),
    )


def inspect_dicom(path: str | Path, *, include_pixels: bool = False) -> dict[str, Any]:
    dicom_path = Path(path)
    if not dicom_path.exists():
        raise FileNotFoundError(f"DICOM file not found: {dicom_path}")

    dataset = pydicom.dcmread(str(dicom_path), stop_before_pixels=not include_pixels)
    return {
        "path": str(dicom_path),
        "sopInstanceUid": str(getattr(dataset, "SOPInstanceUID", "")),
        "studyInstanceUid": str(getattr(dataset, "StudyInstanceUID", "")),
        "seriesInstanceUid": str(getattr(dataset, "SeriesInstanceUID", "")),
        "patientId": str(getattr(dataset, "PatientID", "")),
        "patientName": str(getattr(dataset, "PatientName", "")),
        "modality": str(getattr(dataset, "Modality", "")),
        "rows": int(getattr(dataset, "Rows", 0) or 0),
        "columns": int(getattr(dataset, "Columns", 0) or 0),
        "instanceNumber": int(getattr(dataset, "InstanceNumber", 0) or 0),
        "hasPixelData": "PixelData" in dataset,
        "transferSyntaxUid": str(getattr(dataset.file_meta, "TransferSyntaxUID", "")),
    }


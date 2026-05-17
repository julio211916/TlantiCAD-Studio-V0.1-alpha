from __future__ import annotations

from pathlib import Path

from backend.modules.dicom.anonymization import anonymize_dicom
from backend.modules.dicom.loader import get_dicom_runtime_status, inspect_dicom


def runtime_capabilities() -> dict[str, object]:
    status = get_dicom_runtime_status()
    return {
        "pydicomVersion": status.pydicom_version,
        "simpleItkAvailable": status.simpleitk_available,
        "vtkAvailable": status.vtk_available,
        "owner": "backend-python",
    }


def inspect_study_file(path: str) -> dict[str, object]:
    return inspect_dicom(Path(path), include_pixels=False)


def anonymize_study_file(source: str, target: str, patient_id: str) -> dict[str, str]:
    return anonymize_dicom(source, target, patient_id=patient_id)


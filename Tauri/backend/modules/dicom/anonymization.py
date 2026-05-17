from __future__ import annotations

from pathlib import Path

import pydicom

PII_TAGS = ("PatientName", "PatientID", "PatientBirthDate", "PatientSex")


def anonymize_dicom(source: str | Path, target: str | Path, *, patient_id: str) -> dict[str, str]:
    source_path = Path(source)
    target_path = Path(target)
    if not source_path.exists():
        raise FileNotFoundError(f"DICOM file not found: {source_path}")

    dataset = pydicom.dcmread(str(source_path))
    for tag in PII_TAGS:
        if hasattr(dataset, tag):
            setattr(dataset, tag, "")
    dataset.PatientID = patient_id

    target_path.parent.mkdir(parents=True, exist_ok=True)
    dataset.save_as(str(target_path))
    return {"source": str(source_path), "target": str(target_path), "patientId": patient_id}


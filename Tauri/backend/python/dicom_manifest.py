#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

import pydicom


def _text(value: Any) -> str:
    return "" if value is None else str(value)


def inspect_instance(path: Path) -> dict[str, Any] | None:
    try:
        dataset = pydicom.dcmread(str(path), stop_before_pixels=True, force=True)
    except Exception:
        return None

    stat = path.stat()
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)

    return {
        "path": str(path),
        "fileName": path.name,
        "bytes": stat.st_size,
        "sha256": digest.hexdigest(),
        "sopInstanceUid": _text(getattr(dataset, "SOPInstanceUID", "")),
        "studyInstanceUid": _text(getattr(dataset, "StudyInstanceUID", "")),
        "seriesInstanceUid": _text(getattr(dataset, "SeriesInstanceUID", "")),
        "patientIdPresent": bool(_text(getattr(dataset, "PatientID", ""))),
        "patientNamePresent": bool(_text(getattr(dataset, "PatientName", ""))),
        "modality": _text(getattr(dataset, "Modality", "")),
        "rows": int(getattr(dataset, "Rows", 0) or 0),
        "columns": int(getattr(dataset, "Columns", 0) or 0),
        "instanceNumber": int(getattr(dataset, "InstanceNumber", 0) or 0),
        "hasPixelData": "PixelData" in dataset,
    }


def scan_source(source: Path) -> list[dict[str, Any]]:
    candidates = [source] if source.is_file() else [path for path in source.rglob("*") if path.is_file()]
    instances = [record for path in candidates if (record := inspect_instance(path))]
    return sorted(instances, key=lambda item: (item.get("instanceNumber") or 0, item["fileName"]))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True)
    parser.add_argument("--case-id", required=True)
    parser.add_argument("--role", default="dicom")
    parser.add_argument("--manifest-path", required=True)
    args = parser.parse_args()

    source = Path(args.source).expanduser().resolve()
    if not source.exists():
        raise FileNotFoundError(f"DICOM source not found: {source}")

    instances = scan_source(source)
    if not instances:
        raise RuntimeError(f"No readable DICOM instances found under {source}")

    bytes_total = sum(item["bytes"] for item in instances)
    first = instances[0]
    manifest = {
        "manifestId": Path(args.manifest_path).stem,
        "caseId": args.case_id,
        "sourcePath": str(source),
        "clinicalRole": args.role,
        "engine": "python-pydicom",
        "pydicomVersion": pydicom.__version__,
        "fileCount": len(instances),
        "bytes": bytes_total,
        "studyUid": first.get("studyInstanceUid") or "",
        "seriesUid": first.get("seriesInstanceUid") or "",
        "modality": first.get("modality") or "",
        "rows": first.get("rows") or 0,
        "columns": first.get("columns") or 0,
        "anonymizationStatus": "pii-detected" if any(item["patientIdPresent"] or item["patientNamePresent"] for item in instances) else "no-pii-tags-detected",
        "instances": instances,
    }

    manifest_path = Path(args.manifest_path)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(json.dumps({**manifest, "manifestPath": str(manifest_path)}))


if __name__ == "__main__":
    main()

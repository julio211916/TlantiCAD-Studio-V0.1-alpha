from __future__ import annotations


def unsupported_reference_tool() -> dict[str, str | bool]:
    return {
        "id": "pneumonia-reference-only",
        "available": False,
        "domain": "medical-xray-reference",
        "notes": "Kept only as an architecture reference. TlantiCAD production tools must be dental-specific.",
    }


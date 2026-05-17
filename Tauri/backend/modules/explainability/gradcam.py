from __future__ import annotations


def captum_available() -> bool:
    try:
        __import__("captum")
        return True
    except Exception:
        return False


def gradcam_capability() -> dict[str, str | bool]:
    return {
        "id": "gradcam-dev",
        "available": captum_available(),
        "mode": "dev-only",
        "notes": "Captum is intentionally kept in requirements-ml-dev.txt; production uses ONNX Runtime outputs.",
    }


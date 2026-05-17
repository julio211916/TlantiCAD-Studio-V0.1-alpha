from __future__ import annotations

from fastapi import APIRouter

router = APIRouter()


@router.get("")
def tool_settings() -> dict[str, object]:
    return {
        "windowLevel": {"defaultPreset": "bone", "wl": 400.0, "ww": 3000.0},
        "dicom": {"maxBrowserInstances": 512, "owner": "python-pydicom"},
        "vtk": {"volumeToMesh": "python-vtk", "rendering": "react-three-or-cornerstone"},
        "clinicalExport": {"allowMocks": False, "blockWhenModelMissing": True},
    }


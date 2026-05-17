from __future__ import annotations

from fastapi import APIRouter

from backend.app.services.inference_service import list_local_models, reference_tools

router = APIRouter()


@router.get("")
def models() -> dict[str, object]:
    return {
        "dental": list_local_models(),
        "referenceOnly": reference_tools(),
    }


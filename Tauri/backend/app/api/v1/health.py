from __future__ import annotations

from fastapi import APIRouter

from backend.app.core.settings import get_settings
from backend.app.services.dicom_service import runtime_capabilities

router = APIRouter()


@router.get("/health/local")
def local_health() -> dict[str, object]:
    settings = get_settings()
    return {
        "status": "ready",
        "offlineOnly": settings.offline_only,
        "dicom": runtime_capabilities(),
        "queue": {"driver": "redis", "url": settings.redis_url, "requiredForClinicalExport": False},
        "database": {"url": settings.database_url, "owner": "sqlmodel"},
    }


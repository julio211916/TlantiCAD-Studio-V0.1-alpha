from __future__ import annotations

from fastapi import APIRouter

from backend.app.services.task_service import task_status

router = APIRouter()


@router.get("/{task_id}")
def get_task_status(task_id: str) -> dict[str, object]:
    return task_status(task_id)


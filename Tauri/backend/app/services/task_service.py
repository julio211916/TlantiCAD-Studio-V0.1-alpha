from __future__ import annotations

from dataclasses import dataclass
from time import time


@dataclass(frozen=True)
class LocalTaskStatus:
    task_id: str
    state: str
    updated_at: float
    progress: float
    detail: str


def task_status(task_id: str) -> dict[str, object]:
    return LocalTaskStatus(
        task_id=task_id,
        state="unknown",
        updated_at=time(),
        progress=0.0,
        detail="No durable worker state found. Use Tauri job handles for clinical CAD artifacts.",
    ).__dict__


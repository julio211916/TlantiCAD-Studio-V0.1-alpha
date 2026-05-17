from __future__ import annotations

from fastapi import APIRouter, WebSocket

router = APIRouter()


@router.websocket("/events")
async def events(websocket: WebSocket) -> None:
    await websocket.accept()
    await websocket.send_json(
        {
            "type": "runtime.ready",
            "source": "tlanticad-local-backend",
            "detail": "WebSocket channel is local-only and ready for job progress events.",
        }
    )
    await websocket.close()


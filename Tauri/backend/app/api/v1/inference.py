from __future__ import annotations

from pydantic import BaseModel, Field
from fastapi import APIRouter

from backend.app.services.inference_service import start_inference_task

router = APIRouter()


class InferenceStartRequest(BaseModel):
    asset_handle: str = Field(min_length=1)
    model_id: str = Field(min_length=1)


@router.post("/start")
def start_inference(request: InferenceStartRequest) -> dict[str, object]:
    return start_inference_task(asset_handle=request.asset_handle, model_id=request.model_id)


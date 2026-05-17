from __future__ import annotations

from pathlib import Path
from uuid import uuid4

from backend.modules.ml_tools.pneumonia_tool import unsupported_reference_tool
from backend.modules.ml_tools.registry import dental_tool_registry


def list_local_models(model_root: Path | None = None) -> list[dict[str, object]]:
    return dental_tool_registry(model_root)


def start_inference_task(*, asset_handle: str, model_id: str) -> dict[str, object]:
    task_id = f"infer-{uuid4()}"
    registered_models = list_local_models()
    model = next((entry for entry in registered_models if entry["id"] == model_id), None)
    if model is None:
        return {
            "taskId": task_id,
            "state": "blocked",
            "assetHandle": asset_handle,
            "modelId": model_id,
            "reason": "model is not registered in the local dental model registry",
        }
    if not model["production_ready"]:
        return {
            "taskId": task_id,
            "state": "blocked",
            "assetHandle": asset_handle,
            "modelId": model_id,
            "reason": "clinical inference is blocked until a real local ONNX/INT8 model is installed",
        }
    return {
        "taskId": task_id,
        "state": "queued",
        "assetHandle": asset_handle,
        "modelId": model_id,
        "backend": model["default_backend"],
    }


def reference_tools() -> list[dict[str, object]]:
    return [unsupported_reference_tool()]


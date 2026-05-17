from __future__ import annotations

from pathlib import Path

from backend.modules.inference.base import InferenceBackend, InferenceInput, InferenceOutput


class PyTorchInference(InferenceBackend):
    def __init__(self, model_path: str | Path | None = None) -> None:
        self.model_path = Path(model_path) if model_path else None

    def is_available(self) -> bool:
        try:
            __import__("torch")
        except Exception:
            return False
        return self.model_path is not None and self.model_path.exists()

    def run(self, inference_input: InferenceInput) -> InferenceOutput:
        if not self.is_available():
            raise RuntimeError("PyTorch dev model is unavailable; use ONNX Runtime for production")
        return InferenceOutput(
            model_id=inference_input.model_id,
            backend="pytorch-dev",
            scores={},
            metadata={"mode": "dev-gpu-or-cpu"},
        )


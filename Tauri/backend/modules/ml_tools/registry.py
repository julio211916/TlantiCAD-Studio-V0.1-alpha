from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from backend.modules.explainability.gradcam import gradcam_capability
from backend.modules.inference.onnx_backend import OnnxInference
from backend.modules.inference.pytorch_backend import PyTorchInference


@dataclass(frozen=True)
class MlToolDescriptor:
    id: str
    label: str
    domain: str
    default_backend: str
    production_ready: bool
    notes: str


def dental_tool_registry(model_root: Path | None = None) -> list[dict[str, object]]:
    model_root = model_root or Path("backend/ml/models")
    onnx_backend = OnnxInference(model_root / "dental_cbct_segmentation_int8.onnx")
    torch_backend = PyTorchInference(model_root / "dental_cbct_segmentation.pt")
    tool = MlToolDescriptor(
        id="dental-cbct-segmentation",
        label="Dental CBCT Segmentation",
        domain="dental-dicom",
        default_backend="onnxruntime-int8",
        production_ready=onnx_backend.is_available(),
        notes="Local-only segmentation contract. Export stays blocked until a real ONNX model is registered.",
    )
    return [
        {
            **tool.__dict__,
            "backends": {
                "onnxruntime": onnx_backend.is_available(),
                "pytorchDev": torch_backend.is_available(),
            },
            "explainability": gradcam_capability(),
        }
    ]


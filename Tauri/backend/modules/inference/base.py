from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class InferenceInput:
    asset_handle: str
    model_id: str
    payload_path: Path | None = None


@dataclass(frozen=True)
class InferenceOutput:
    model_id: str
    backend: str
    scores: dict[str, float]
    artifact_path: Path | None = None
    metadata: dict[str, Any] | None = None


class InferenceBackend(ABC):
    @abstractmethod
    def is_available(self) -> bool:
        raise NotImplementedError

    @abstractmethod
    def run(self, inference_input: InferenceInput) -> InferenceOutput:
        raise NotImplementedError


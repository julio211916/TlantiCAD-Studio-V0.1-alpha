from __future__ import annotations

from dataclasses import dataclass

import numpy as np


@dataclass(frozen=True)
class PreprocessResult:
    shape: tuple[int, ...]
    dtype: str
    min_value: float
    max_value: float


def normalize_hounsfield_like(volume: np.ndarray, *, lower: float = -1000.0, upper: float = 3000.0) -> np.ndarray:
    clipped = np.clip(volume.astype(np.float32, copy=False), lower, upper)
    return (clipped - lower) / (upper - lower)


def summarize_volume(volume: np.ndarray) -> PreprocessResult:
    return PreprocessResult(
        shape=tuple(int(value) for value in volume.shape),
        dtype=str(volume.dtype),
        min_value=float(np.min(volume)) if volume.size else 0.0,
        max_value=float(np.max(volume)) if volume.size else 0.0,
    )


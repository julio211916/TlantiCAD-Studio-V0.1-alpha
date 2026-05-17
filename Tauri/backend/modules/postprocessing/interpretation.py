from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ConfidenceBand:
    label: str
    min_score: float
    max_score: float


BANDS = (
    ConfidenceBand("low", 0.0, 0.5),
    ConfidenceBand("review", 0.5, 0.85),
    ConfidenceBand("high", 0.85, 1.0),
)


def confidence_band(score: float) -> str:
    bounded = max(0.0, min(1.0, score))
    for band in BANDS:
        if band.min_score <= bounded <= band.max_score:
            return band.label
    return "review"


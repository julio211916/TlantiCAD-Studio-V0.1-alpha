from __future__ import annotations

from pydantic import BaseModel, Field
from fastapi import APIRouter, HTTPException

from backend.app.services.dicom_service import anonymize_study_file, inspect_study_file

router = APIRouter()


class StudyInspectRequest(BaseModel):
    path: str = Field(min_length=1)


class StudyAnonymizeRequest(BaseModel):
    source: str = Field(min_length=1)
    target: str = Field(min_length=1)
    patient_id: str = Field(min_length=1)


@router.post("/inspect")
def inspect_study(request: StudyInspectRequest) -> dict[str, object]:
    try:
        return inspect_study_file(request.path)
    except FileNotFoundError as error:
        raise HTTPException(status_code=404, detail=str(error)) from error


@router.post("/anonymize")
def anonymize_study(request: StudyAnonymizeRequest) -> dict[str, str]:
    try:
        return anonymize_study_file(request.source, request.target, request.patient_id)
    except FileNotFoundError as error:
        raise HTTPException(status_code=404, detail=str(error)) from error


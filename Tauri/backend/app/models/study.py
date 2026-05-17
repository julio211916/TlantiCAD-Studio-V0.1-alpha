from datetime import datetime, timezone
from typing import Optional
from uuid import uuid4

from sqlmodel import Field, SQLModel


class Study(SQLModel, table=True):
    id: str = Field(default_factory=lambda: str(uuid4()), primary_key=True)
    patient_id: str
    modality: str = "CT"
    study_instance_uid: Optional[str] = None
    source_path: str
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

from __future__ import annotations

import importlib.util
import os
import sys
import time
import uuid
from pathlib import Path
from typing import Any

from fastapi import FastAPI
from pydantic import BaseModel

from slicer_automated_dental_runtime import (
    cancel_job,
    download_fixture,
    download_all_models,
    download_slicer_runtime,
    extension_status,
    fixture_status,
    job_status,
    model_status,
    runtime_status,
    start_job,
)


STARTED_AT = time.time()
APP_NAME = "tlanticad-trame-slicer-sidecar"


class DicomLoadRequest(BaseModel):
    sourcePath: str
    caseId: str | None = None


class TrameSessionRequest(BaseModel):
    sourcePath: str | None = None
    caseId: str | None = None


class SlicerModelsDownloadRequest(BaseModel):
    includeOptional: bool = True


class SlicerFixtureDownloadRequest(BaseModel):
    fixtureId: str = "amasss-mg-test-scan"


class SlicerClinicalJobRequest(BaseModel):
    caseId: str
    workflowId: str
    sourcePath: str
    outputDir: str | None = None
    modelId: str | None = None
    options: dict[str, Any] = {}


def _module_status(name: str) -> dict[str, Any]:
    spec = importlib.util.find_spec(name)
    if spec is None:
        return {"name": name, "available": False, "version": None}
    version = None
    try:
        module = __import__(name)
        version = getattr(module, "__version__", None)
    except Exception as exc:  # pragma: no cover - diagnostic payload only
        return {"name": name, "available": False, "version": None, "error": str(exc)}
    return {"name": name, "available": True, "version": version}


def _slicer_status() -> dict[str, Any]:
    modules = [_module_status(name) for name in ("vtk", "trame", "trame_vtk", "trame_tauri", "trame_slicer")]
    return {
        "ready": all(item["available"] for item in modules),
        "modules": modules,
        "python": sys.executable,
        "pythonVersion": sys.version.split()[0],
    }


app = FastAPI(title=APP_NAME, version="0.1.0")


@app.get("/health")
def health() -> dict[str, Any]:
    slicer = _slicer_status()
    return {
        "ok": True,
        "name": APP_NAME,
        "uptimeSecs": int(time.time() - STARTED_AT),
        "slicerReady": slicer["ready"],
        "python": slicer["python"],
    }


@app.post("/dicom/load")
def dicom_load(request: DicomLoadRequest) -> dict[str, Any]:
    source = Path(request.sourcePath).expanduser()
    exists = source.exists()
    return {
        "accepted": exists,
        "caseId": request.caseId,
        "sourcePath": str(source),
        "kind": "directory" if source.is_dir() else "file",
        "exists": exists,
        "engine": "trame-slicer",
        "message": "DICOM source accepted for local Slicer/trame session." if exists else "DICOM source path does not exist.",
    }


@app.get("/slicer/status")
def slicer_status() -> dict[str, Any]:
    return _slicer_status()


@app.post("/trame/session")
def trame_session(request: TrameSessionRequest) -> dict[str, Any]:
    session_id = f"tlanti-trame-{uuid.uuid4().hex[:12]}"
    port = int(os.environ.get("TLANTI_TRAME_SLICER_PORT", "17494"))
    return {
        "sessionId": session_id,
        "url": f"http://127.0.0.1:{port}/slicer/status?session={session_id}",
        "viewer": "trame-slicer",
        "state": "ready" if _slicer_status()["ready"] else "degraded",
        "sourcePath": request.sourcePath,
        "caseId": request.caseId,
    }


@app.get("/slicer/runtime/status")
def slicer_runtime_status() -> dict[str, Any]:
    return runtime_status()


@app.post("/slicer/runtime/download")
def slicer_runtime_download() -> dict[str, Any]:
    return download_slicer_runtime()


@app.get("/slicer/extensions/status")
def slicer_extensions_status() -> dict[str, Any]:
    return extension_status()


@app.get("/slicer/models/status")
def slicer_models_status() -> dict[str, Any]:
    return model_status()


@app.get("/slicer/fixtures/status")
def slicer_fixtures_status() -> dict[str, Any]:
    return fixture_status()


@app.post("/slicer/fixtures/download")
def slicer_fixtures_download(request: SlicerFixtureDownloadRequest) -> dict[str, Any]:
    return download_fixture(request.fixtureId)


@app.post("/slicer/models/download-all")
def slicer_models_download_all(request: SlicerModelsDownloadRequest) -> dict[str, Any]:
    return download_all_models(include_optional=request.includeOptional)


@app.post("/slicer/jobs/start")
def slicer_jobs_start(request: SlicerClinicalJobRequest) -> dict[str, Any]:
    return start_job(request.model_dump())


@app.get("/slicer/jobs/{job_id}")
def slicer_jobs_status(job_id: str) -> dict[str, Any]:
    return job_status(job_id)


@app.post("/slicer/jobs/{job_id}/cancel")
def slicer_jobs_cancel(job_id: str) -> dict[str, Any]:
    return cancel_job(job_id)

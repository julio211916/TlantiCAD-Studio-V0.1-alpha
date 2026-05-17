from __future__ import annotations

import hashlib
import json
import os
import plistlib
import shutil
import subprocess
import sys
import threading
import time
import urllib.request
import uuid
import zipfile
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
SLICER_ROOT = ROOT / "resources" / "slicer"
MANIFEST_PATH = SLICER_ROOT / "models" / "manifest.json"
MODEL_CACHE = SLICER_ROOT / "models" / "cache"
DOWNLOADS = SLICER_ROOT / "models" / "downloads"
FIXTURE_MANIFEST_PATH = SLICER_ROOT / "fixtures" / "manifest.json"
FIXTURE_CACHE = SLICER_ROOT / "fixtures" / "cache"
JOBS_ROOT = SLICER_ROOT / "jobs"
DOWNLOAD_TIMEOUT_SECONDS = int(os.environ.get("TLANTI_SLICER_DOWNLOAD_TIMEOUT_SECONDS", "900"))
DOWNLOAD_RETRIES = int(os.environ.get("TLANTI_SLICER_DOWNLOAD_RETRIES", "3"))
JOBS: dict[str, dict[str, Any]] = {}
JOB_PROCESSES: dict[str, subprocess.Popen[str]] = {}
JOBS_LOCK = threading.RLock()


def load_manifest() -> dict[str, Any]:
    if not MANIFEST_PATH.exists():
        return {
            "schemaVersion": 0,
            "models": [],
            "error": f"missing manifest: {MANIFEST_PATH}",
        }
    return json.loads(MANIFEST_PATH.read_text())


def load_fixture_manifest() -> dict[str, Any]:
    if not FIXTURE_MANIFEST_PATH.exists():
        return {
            "schemaVersion": 0,
            "fixtures": [],
            "error": f"missing manifest: {FIXTURE_MANIFEST_PATH}",
        }
    return json.loads(FIXTURE_MANIFEST_PATH.read_text())


def slicer_executable() -> Path:
    env_path = os.environ.get("TLANTI_SLICER_EXECUTABLE")
    if env_path:
        return Path(env_path).expanduser()
    manifest = load_manifest()
    relative = manifest.get("slicer", {}).get("relativeExecutable", "runtime/Slicer.app/Contents/MacOS/Slicer")
    return SLICER_ROOT / relative


def runtime_status() -> dict[str, Any]:
    manifest = load_manifest()
    executable = slicer_executable()
    extension_path = SLICER_ROOT / manifest.get("extension", {}).get("relativePath", "extensions/SlicerAutomatedDentalTools")
    version_output = None
    if executable.exists():
        try:
            probe = subprocess.run(
                [
                    str(executable),
                    "--no-main-window",
                    "--disable-settings",
                    "--python-code",
                    "import slicer; print(slicer.app.applicationVersion); slicer.util.exit(0)",
                ],
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )
            probe_output = (probe.stdout or probe.stderr).strip()
            version_output = probe_output.splitlines()[-1] if probe_output else None
        except Exception as exc:
            version_output = f"version probe failed: {exc}"
    return {
        "ready": executable.exists() and extension_path.exists(),
        "slicerRoot": str(SLICER_ROOT),
        "manifestPath": str(MANIFEST_PATH),
        "executable": str(executable),
        "executablePresent": executable.exists(),
        "extensionPath": str(extension_path),
        "extensionPresent": extension_path.exists(),
        "version": version_output,
        "downloadUrl": manifest.get("slicer", {}).get("macosDownloadUrl"),
        "sha512": manifest.get("slicer", {}).get("macosSha512"),
    }


def _sha512(path: Path) -> str:
    digest = hashlib.sha512()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_slicer_runtime() -> dict[str, Any]:
    manifest = load_manifest()
    slicer = manifest.get("slicer", {})
    url = slicer.get("macosDownloadUrl")
    if not url:
        return {"ok": False, "error": "Slicer manifest has no macOS download URL.", "status": runtime_status()}
    executable = slicer_executable()
    if executable.exists():
        return {"ok": True, "state": "installed", "status": runtime_status()}
    runtime_dir = SLICER_ROOT / "runtime"
    runtime_dir.mkdir(parents=True, exist_ok=True)
    dmg_path = DOWNLOADS / "slicer" / "Slicer.dmg"
    _download(url, dmg_path)
    expected_sha512 = slicer.get("macosSha512")
    if expected_sha512 and _sha512(dmg_path) != expected_sha512:
        return {"ok": False, "error": "Slicer DMG SHA512 mismatch.", "download": str(dmg_path), "status": runtime_status()}
    if os.uname().sysname != "Darwin":
        return {"ok": False, "error": "Automatic Slicer DMG extraction is currently macOS-only.", "download": str(dmg_path), "status": runtime_status()}

    attach = subprocess.run(
        ["sh", "-c", 'yes | hdiutil attach "$SLICER_DMG" -nobrowse -noautoopen -plist'],
        capture_output=True,
        text=True,
        check=False,
        env={**os.environ, "SLICER_DMG": str(dmg_path)},
    )
    if attach.returncode != 0:
        return {"ok": False, "error": attach.stderr.strip() or attach.stdout.strip(), "download": str(dmg_path), "status": runtime_status()}
    plist_start = attach.stdout.find("<?xml")
    if plist_start < 0:
        return {"ok": False, "error": "Slicer DMG mounted without plist output.", "download": str(dmg_path), "status": runtime_status()}
    attach_plist = plistlib.loads(attach.stdout[plist_start:].encode("utf-8"))
    mounted = [
        Path(entity["mount-point"])
        for entity in attach_plist.get("system-entities", [])
        if entity.get("mount-point")
    ]
    try:
        app_candidates = []
        for mount in mounted:
            app_candidates.extend(mount.glob("*.app"))
        if not app_candidates:
            raise RuntimeError("No .app bundle found in mounted Slicer DMG.")
        target_app = runtime_dir / "Slicer.app"
        if target_app.exists():
            shutil.rmtree(target_app)
        shutil.copytree(app_candidates[0], target_app, symlinks=True)
    finally:
        for mount in mounted:
            subprocess.run(["hdiutil", "detach", str(mount), "-quiet"], capture_output=True, text=True, check=False)
    return {"ok": executable.exists(), "state": "installed" if executable.exists() else "failed", "status": runtime_status()}


def _artifact_target(model: dict[str, Any], artifact: dict[str, Any]) -> Path:
    layout = model.get("layout")
    base = MODEL_CACHE / model["id"]
    if layout:
        base = base / layout
    return base / artifact["name"]


def _fixture_target(fixture: dict[str, Any]) -> Path:
    relative = fixture.get("target") or f'{fixture["id"]}.bin'
    return SLICER_ROOT / "fixtures" / relative


def _download(url: str, target: Path) -> int:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists() and target.stat().st_size > 0:
        return target.stat().st_size
    tmp = target.with_suffix(target.suffix + ".part")
    last_error: Exception | None = None
    for attempt in range(1, DOWNLOAD_RETRIES + 1):
        tmp.unlink(missing_ok=True)
        try:
            request = urllib.request.Request(url, headers={"User-Agent": "TlantiCAD-SlicerRuntime/0.1"})
            with urllib.request.urlopen(request, timeout=DOWNLOAD_TIMEOUT_SECONDS) as response, open(tmp, "wb") as handle:
                shutil.copyfileobj(response, handle)
                expected_size = response.headers.get("Content-Length")
            if expected_size and tmp.stat().st_size != int(expected_size):
                actual = tmp.stat().st_size
                raise RuntimeError(f"Download truncated for {url}: expected {expected_size} bytes, got {actual}.")
            tmp.replace(target)
            return target.stat().st_size
        except Exception as exc:
            last_error = exc
            tmp.unlink(missing_ok=True)
            if attempt < DOWNLOAD_RETRIES:
                time.sleep(min(30, 2**attempt))
    raise RuntimeError(f"Download failed after {DOWNLOAD_RETRIES} attempts for {url}: {last_error}")


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def model_status() -> dict[str, Any]:
    manifest = load_manifest()
    models = []
    for model in manifest.get("models", []):
        artifacts = []
        installed = True
        for artifact in model.get("artifacts", []):
            target = _artifact_target(model, artifact)
            extracted_marker = MODEL_CACHE / model["id"] / ".extracted" / artifact["name"].replace("/", "__")
            present = target.exists() or extracted_marker.exists()
            installed = installed and present
            artifacts.append(
                {
                    "name": artifact["name"],
                    "url": artifact.get("url"),
                    "kind": artifact.get("kind", "file"),
                    "target": str(target),
                    "present": present,
                    "bytes": target.stat().st_size if target.exists() else None,
                    "sha256": _sha256(target) if target.exists() and target.is_file() else None,
                }
            )
        models.append(
            {
                "id": model["id"],
                "label": model.get("label", model["id"]),
                "workflowIds": model.get("workflowIds", []),
                "required": bool(model.get("required", False)),
                "installed": installed,
                "state": "installed" if installed else "downloadable",
                "artifacts": artifacts,
            }
        )
    required = [model for model in models if model["required"]]
    return {
        "manifestPath": str(MANIFEST_PATH),
        "cacheRoot": str(MODEL_CACHE),
        "total": len(models),
        "installed": sum(1 for model in models if model["installed"]),
        "requiredInstalled": all(model["installed"] for model in required),
        "models": models,
    }


def download_all_models(include_optional: bool = True) -> dict[str, Any]:
    manifest = load_manifest()
    DOWNLOADS.mkdir(parents=True, exist_ok=True)
    MODEL_CACHE.mkdir(parents=True, exist_ok=True)
    results = []
    failures = []
    for model in manifest.get("models", []):
        if not include_optional and not model.get("required", False):
            continue
        for artifact in model.get("artifacts", []):
            target = _artifact_target(model, artifact)
            marker = MODEL_CACHE / model["id"] / ".extracted" / artifact["name"].replace("/", "__")
            if target.exists() or marker.exists():
                results.append({"modelId": model["id"], "artifact": artifact["name"], "state": "installed", "target": str(target)})
                continue
            try:
                download_target = DOWNLOADS / model["id"] / artifact["name"].replace("/", "__")
                size = _download(artifact["url"], download_target)
                if artifact.get("sha256") and _sha256(download_target) != artifact["sha256"]:
                    raise RuntimeError("checksum mismatch")
                if artifact.get("kind") == "zip":
                    extract_root = MODEL_CACHE / model["id"]
                    extract_root.mkdir(parents=True, exist_ok=True)
                    with zipfile.ZipFile(download_target) as archive:
                        archive.extractall(extract_root)
                    marker.parent.mkdir(parents=True, exist_ok=True)
                    marker.write_text(json.dumps({"download": str(download_target), "extractedAt": time.time()}))
                    target = extract_root
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(download_target, target)
                results.append({"modelId": model["id"], "artifact": artifact["name"], "state": "downloaded", "target": str(target), "bytes": size})
            except Exception as exc:
                failure = {"modelId": model["id"], "artifact": artifact["name"], "state": "failed", "url": artifact.get("url"), "error": str(exc)}
                failures.append(failure)
                results.append(failure)
    return {
        "ok": not failures,
        "failures": failures,
        "results": results,
        "status": model_status(),
    }


def extension_status() -> dict[str, Any]:
    manifest = load_manifest()
    extension = manifest.get("extension", {})
    extension_path = SLICER_ROOT / extension.get("relativePath", "extensions/SlicerAutomatedDentalTools")
    module_files = sorted(path.name for path in extension_path.glob("*/*.py"))[:80] if extension_path.exists() else []
    return {
        "id": extension.get("id"),
        "url": extension.get("url"),
        "path": str(extension_path),
        "present": extension_path.exists(),
        "moduleFiles": module_files,
    }


def fixture_status() -> dict[str, Any]:
    manifest = load_fixture_manifest()
    fixtures = []
    for fixture in manifest.get("fixtures", []):
        target = _fixture_target(fixture)
        present = target.exists()
        actual_sha256 = _sha256(target) if present else None
        expected_sha256 = fixture.get("sha256")
        fixtures.append(
            {
                "id": fixture["id"],
                "label": fixture.get("label", fixture["id"]),
                "modality": fixture.get("modality"),
                "format": fixture.get("format"),
                "source": fixture.get("source"),
                "url": fixture.get("url"),
                "target": str(target),
                "expectedBytes": fixture.get("bytes"),
                "expectedSha256": expected_sha256,
                "present": present,
                "bytes": target.stat().st_size if present else None,
                "sha256": actual_sha256,
                "validChecksum": bool(present and expected_sha256 and actual_sha256 == expected_sha256),
            }
        )
    return {
        "manifestPath": str(FIXTURE_MANIFEST_PATH),
        "cacheRoot": str(FIXTURE_CACHE),
        "total": len(fixtures),
        "ready": all(item["present"] and item["validChecksum"] for item in fixtures),
        "fixtures": fixtures,
    }


def download_fixture(fixture_id: str) -> dict[str, Any]:
    manifest = load_fixture_manifest()
    fixture = next((item for item in manifest.get("fixtures", []) if item.get("id") == fixture_id), None)
    if fixture is None:
        raise RuntimeError(f"Unknown fixture: {fixture_id}")
    url = fixture.get("url")
    if not url:
        raise RuntimeError(f"Fixture has no URL: {fixture_id}")
    target = _fixture_target(fixture)
    size = _download(url, target)
    expected_sha256 = fixture.get("sha256")
    actual_sha256 = _sha256(target)
    if expected_sha256 and actual_sha256 != expected_sha256:
        raise RuntimeError(f"Fixture checksum mismatch for {fixture_id}: expected {expected_sha256}, got {actual_sha256}")
    expected_bytes = fixture.get("bytes")
    if expected_bytes and int(expected_bytes) != target.stat().st_size:
        raise RuntimeError(
            f"Fixture size mismatch for {fixture_id}: expected {expected_bytes} bytes, got {target.stat().st_size}"
        )
    return {
        "ok": True,
        "fixtureId": fixture_id,
        "target": str(target),
        "bytes": size,
        "sha256": actual_sha256,
        "status": fixture_status(),
    }


def _workflow_model_ids(workflow_id: str) -> list[str]:
    return [
        model["id"]
        for model in load_manifest().get("models", [])
        if workflow_id in model.get("workflowIds", [])
    ]


def _now() -> int:
    return int(time.time())


def _job_dir(job_id: str, request: dict[str, Any]) -> Path:
    output_dir = request.get("outputDir")
    if output_dir:
        return Path(output_dir).expanduser() / job_id
    return JOBS_ROOT / job_id


def _public_status(status: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in status.items() if key not in {"process"}}


def _write_status(status: dict[str, Any]) -> None:
    job_dir = Path(status["jobDir"])
    job_dir.mkdir(parents=True, exist_ok=True)
    (job_dir / "status.json").write_text(json.dumps(_public_status(status), indent=2))


def _set_job(job_id: str, **updates: Any) -> dict[str, Any]:
    with JOBS_LOCK:
        status = JOBS[job_id]
        status.update(updates)
        status["updatedAt"] = _now()
        _write_status(status)
        return _public_status(status)


def _append_log(job_id: str, line: str) -> None:
    line = line.rstrip()
    if not line:
        return
    with JOBS_LOCK:
        status = JOBS[job_id]
        event = {"ts": _now(), "line": line}
        status["logs"].append(line)
        status["logs"] = status["logs"][-300:]
        job_dir = Path(status["jobDir"])
        job_dir.mkdir(parents=True, exist_ok=True)
        with open(job_dir / "logs.ndjson", "a", encoding="utf-8") as handle:
            handle.write(json.dumps(event) + "\n")
        if "<filter-progress>" in line:
            try:
                value = float(line.split("<filter-progress>", 1)[1].split("</filter-progress>", 1)[0])
                status["progress"] = max(0.0, min(1.0, value if value <= 1 else value / 100))
            except Exception:
                pass
        elif line.startswith("[PROGRESS]"):
            try:
                value = float(line.replace("[PROGRESS]", "").strip())
                status["progress"] = max(0.0, min(1.0, value / 100))
            except Exception:
                pass
        _write_status(status)


def _sha256_quick(path: Path) -> str:
    return _sha256(path)


def _stem_without_medical_suffix(path: Path) -> str:
    name = path.name
    for suffix in (".nii.gz", ".seg.nrrd", ".nrrd", ".nii"):
        if name.endswith(suffix):
            return name[: -len(suffix)]
    return path.stem


def _labelmap_to_seg_nrrd(labelmap_path: Path, output_dir: Path) -> Path:
    import SimpleITK as sitk

    image = sitk.ReadImage(str(labelmap_path))
    target = output_dir / f"{_stem_without_medical_suffix(labelmap_path)}.seg.nrrd"
    sitk.WriteImage(image, str(target), True)
    return target


def _labelmap_to_stl(labelmap_path: Path, output_dir: Path) -> Path | None:
    import vtk

    reader = vtk.vtkNIFTIImageReader() if labelmap_path.name.endswith((".nii", ".nii.gz")) else vtk.vtkNrrdReader()
    reader.SetFileName(str(labelmap_path))
    reader.Update()

    image = reader.GetOutput()
    scalar_range = image.GetScalarRange()
    if scalar_range[1] <= 0:
        return None

    contour = vtk.vtkDiscreteMarchingCubes()
    contour.SetInputData(image)
    contour.GenerateValues(1, 1, int(scalar_range[1]))
    contour.Update()

    cleaner = vtk.vtkCleanPolyData()
    cleaner.SetInputConnection(contour.GetOutputPort())
    cleaner.Update()

    target = output_dir / f"{_stem_without_medical_suffix(labelmap_path)}.stl"
    writer = vtk.vtkSTLWriter()
    writer.SetFileName(str(target))
    writer.SetInputConnection(cleaner.GetOutputPort())
    writer.Write()
    return target if target.exists() and target.stat().st_size > 0 else None


def _postprocess_clinical_outputs(job_id: str) -> None:
    with JOBS_LOCK:
        output_dir = Path(JOBS[job_id]["outputDir"])
    output_dir.mkdir(parents=True, exist_ok=True)
    labelmaps = [
        path
        for path in sorted(output_dir.rglob("*"))
        if path.is_file()
        and path.name not in {"artifacts.json", "logs.ndjson", "status.json"}
        and path.name.endswith((".nii", ".nii.gz", ".nrrd", ".seg.nrrd"))
    ]
    for labelmap in labelmaps:
        try:
            if not labelmap.name.endswith(".seg.nrrd"):
                seg_path = _labelmap_to_seg_nrrd(labelmap, output_dir)
                _append_log(job_id, f"Postprocessed labelmap to Slicer segmentation handle: {seg_path}")
            stl_path = _labelmap_to_stl(labelmap, output_dir)
            if stl_path:
                _append_log(job_id, f"Postprocessed labelmap to Mesh Vault-ready STL: {stl_path}")
        except Exception as exc:
            _append_log(job_id, f"Postprocess warning for {labelmap}: {exc}")


def _collect_artifacts(job_id: str) -> list[dict[str, Any]]:
    with JOBS_LOCK:
        status = JOBS[job_id]
        output_dir = Path(status["outputDir"])
    artifacts = []
    output_dir.mkdir(parents=True, exist_ok=True)
    if output_dir.exists():
        for path in sorted(output_dir.rglob("*")):
            if not path.is_file() or path.name in {"status.json", "artifacts.json", "logs.ndjson"}:
                continue
            kind = path.suffix.lower().lstrip(".") or "file"
            artifact = {
                "id": f"{job_id}:{path.relative_to(output_dir)}",
                "kind": "seg-nrrd" if path.name.endswith(".seg.nrrd") else kind,
                "path": str(path),
                "bytes": path.stat().st_size,
                "sha256": _sha256_quick(path),
                "producer": "slicer-automated-dental-tools",
                "createdAt": _now(),
            }
            if path.suffix.lower() == ".stl":
                artifact["meshVaultImportRequest"] = {
                    "sourcePath": str(path),
                    "kind": "stl-mesh",
                    "moduleId": "dicom",
                    "role": "clinical-segmentation-mesh",
                    "metadata": {
                        "sourceJobId": job_id,
                        "producer": "slicer-automated-dental-tools",
                    },
                }
            artifacts.append(artifact)
    manifest = output_dir / "artifacts.json"
    manifest.write_text(json.dumps({"jobId": job_id, "artifacts": artifacts}, indent=2))
    return artifacts


def _prepare_nnunet_input(source: Path, job_dir: Path) -> Path:
    input_dir = job_dir / "nnunet-input"
    input_dir.mkdir(parents=True, exist_ok=True)
    candidates = [source] if source.is_file() else sorted(
        path for path in source.iterdir() if path.name.lower().endswith((".nii.gz", ".nii"))
    ) if source.is_dir() else []
    if not candidates:
        raise RuntimeError("nnUNet workflows require a local .nii or .nii.gz input; DICOM folder conversion is not wired for this Slicer job yet.")
    source_file = candidates[0]
    target = input_dir / "case_0000.nii.gz"
    if source_file.name.endswith(".nii.gz"):
        shutil.copy2(source_file, target)
    else:
        shutil.copy2(source_file, input_dir / "case_0000.nii")
    return input_dir


def _nnunet_command(request: dict[str, Any], job_dir: Path, model_id: str, dataset_folder: str) -> tuple[list[str], dict[str, str]]:
    source = Path(request["sourcePath"]).expanduser()
    input_dir = _prepare_nnunet_input(source, job_dir)
    output_dir = job_dir / "output"
    output_dir.mkdir(parents=True, exist_ok=True)
    model_root = MODEL_CACHE / model_id
    dataset_path = model_root / dataset_folder
    plans = dataset_path / "nnUNetTrainer__nnUNetPlans__3d_fullres"
    checkpoint = plans / "fold_0" / "checkpoint_final.pth"
    if not checkpoint.exists():
        raise RuntimeError(f"Missing nnUNet checkpoint for {model_id}: {checkpoint}")
    env = {
        "nnUNet_results": str(model_root),
        "nnUNet_raw": str(job_dir / "nnunet-raw"),
        "nnUNet_preprocessed": str(job_dir / "nnunet-preprocessed"),
    }
    command = [
        str(Path(sys.executable).parent / "nnUNetv2_predict"),
        "-i",
        str(input_dir),
        "-o",
        str(output_dir),
        "-d",
        dataset_folder,
        "-c",
        "3d_fullres",
        "-f",
        "0",
        "-device",
        "cpu",
        "--disable_tta",
    ]
    return command, env


def _slicer_launch_command(module_path: Path, module_name: str, args: list[str]) -> tuple[list[str], dict[str, str]]:
    return [
        str(slicer_executable()),
        "--no-main-window",
        "--disable-settings",
        "--additional-module-path",
        str(module_path),
        "--launch",
        module_name,
        *args,
    ], {}


def _build_workflow_command(request: dict[str, Any], job_id: str) -> tuple[list[str], dict[str, str]]:
    workflow_id = request.get("workflowId") or "cbct-segmentation"
    source = str(Path(request["sourcePath"]).expanduser())
    job_dir = Path(JOBS[job_id]["jobDir"])
    output = Path(JOBS[job_id]["outputDir"])
    temp = job_dir / "tmp"
    temp.mkdir(parents=True, exist_ok=True)
    extension = SLICER_ROOT / "extensions" / "SlicerAutomatedDentalTools"

    if workflow_id in {"cbct-segmentation", "adult-dental-segmentation"}:
        return _nnunet_command(request, job_dir, "dentalsegmentator-adult-nnunet", "Dataset111_453CT")
    if workflow_id == "pediatric-segmentation":
        return _nnunet_command(request, job_dir, "pediatric-dental-segmentator", "Dataset001_380CT")
    if workflow_id == "universal-labeling":
        return _nnunet_command(request, job_dir, "universal-lab-dental-segmentator", "Dataset002_380CT")
    if workflow_id == "nasomaxillary-segmentation":
        return _nnunet_command(request, job_dir, "nasomaxillary-dental-segmentator", "Dataset001_max4")
    if workflow_id == "cbct-landmarks":
        return _slicer_launch_command(
            extension / "ALI_CBCT",
            "ALI_CBCT",
            [source, str(MODEL_CACHE / "ali-cbct-landmarks"), "Cranial_Base,Upper_Bones_v2,Lower_Bones_1", str(output), str(temp), "false", "[1,0.3]", "[1,1]", "[64,64,64]", "10"],
        )
    if workflow_id == "ios-landmarks":
        return _slicer_launch_command(
            extension / "ALI_IOS",
            "ALI_IOS",
            [source, str(MODEL_CACHE / "ali-ios-landmarks"), "O,C", "UR6,UL6,LR6,LL6", str(output), "224", "0", "1", str(job_dir / "ali-ios.log")],
        )
    if workflow_id == "cbct-orientation":
        return _slicer_launch_command(
            extension / "ASO_CBCT" / "PRE_ASO_CBCT",
            "PRE_ASO_CBCT",
            [source, str(output), str(MODEL_CACHE / "aso-cbct-orientation"), "false", str(temp), "false"],
        )
    if workflow_id == "ios-orientation":
        return _slicer_launch_command(
            extension / "ASO_IOS" / "PRE_ASO_IOS",
            "PRE_ASO_IOS",
            [source, str(MODEL_CACHE / "aso-ios-orientation"), str(output), "_PRE_ASO", "UR6,UL6,LR6,LL6", "false", "Upper", str(job_dir / "errors"), str(job_dir / "pre-aso-ios.log")],
        )
    if workflow_id == "cbct-registration":
        options = request.get("options") or {}
        t2 = options.get("movingPath")
        if not t2:
            raise RuntimeError("cbct-registration requires options.movingPath for T2/moving scan folder.")
        return _slicer_launch_command(
            extension / "AREG_CBCT",
            "AREG_CBCT",
            [source, str(Path(str(t2)).expanduser()), str(options.get("regType", "MAX")), str(output), "_AREG", "false", "0", str(temp), "true", str(options.get("maskFolderT1", "None"))],
        )
    if workflow_id == "ios-registration":
        options = request.get("options") or {}
        t2 = options.get("movingPath")
        if not t2:
            raise RuntimeError("ios-registration requires options.movingPath for T2/moving mesh folder.")
        return _slicer_launch_command(
            extension / "AREG_IOS",
            "AREG_IOS",
            [source, str(Path(str(t2)).expanduser()), str(output), str(MODEL_CACHE / "areg-ios-registration"), "_AREG", str(job_dir / "areg-ios.log"), "registration"],
        )
    if workflow_id == "canine-localization":
        params = job_dir / "clic-params.json"
        params.write_text(json.dumps({"input_path": source, "model_folder": str(MODEL_CACHE / "clic-impacted-canines"), "output_dir": str(output), "suffix": "seg"}))
        return [
            str(slicer_executable()),
            "--no-main-window",
            "--disable-settings",
            "--python-script",
            str(extension / "CLIC" / "runner" / "clic_runner.py"),
            f"--params_json={params}",
        ], {}
    raise RuntimeError(f"No headless Slicer CLI adapter has been mapped for workflow {workflow_id}.")


def _run_job(job_id: str, request: dict[str, Any]) -> None:
    try:
        command, extra_env = _build_workflow_command(request, job_id)
        _set_job(job_id, state="running", progress=0.02, message="Launching clinical workflow.", command=command)
        _append_log(job_id, "$ " + " ".join(command))
        env = {**os.environ, **extra_env}
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            cwd=str(Path(JOBS[job_id]["jobDir"])),
            env=env,
        )
        with JOBS_LOCK:
            JOB_PROCESSES[job_id] = process
        assert process.stdout is not None
        for line in process.stdout:
            _append_log(job_id, line)
        returncode = process.wait()
        with JOBS_LOCK:
            JOB_PROCESSES.pop(job_id, None)
        if JOBS[job_id].get("state") == "cancelled":
            return
        if returncode == 0:
            _postprocess_clinical_outputs(job_id)
        artifacts = _collect_artifacts(job_id)
        if returncode != 0:
            _set_job(job_id, state="failed", progress=1.0, message=f"Clinical workflow failed with exit code {returncode}.", error=f"exit code {returncode}", outputArtifacts=artifacts)
            return
        if not artifacts:
            _set_job(job_id, state="failed", progress=1.0, message="Clinical workflow finished without output artifacts.", error="no-output-artifacts", outputArtifacts=[])
            return
        _set_job(job_id, state="completed", progress=1.0, message="Clinical workflow completed with artifact manifest.", outputArtifacts=artifacts, error=None)
    except Exception as exc:
        _append_log(job_id, f"ERROR: {exc}")
        _set_job(job_id, state="failed", progress=1.0, message=str(exc), error=str(exc), outputArtifacts=_collect_artifacts(job_id))


def start_job(request: dict[str, Any]) -> dict[str, Any]:
    job_id = f"slicer-job-{uuid.uuid4().hex[:12]}"
    workflow_id = request.get("workflowId") or "cbct-segmentation"
    source_path = Path(request.get("sourcePath") or "").expanduser()
    runtime = runtime_status()
    models = model_status()
    needed = set(_workflow_model_ids(workflow_id))
    missing = [model["id"] for model in models["models"] if model["id"] in needed and not model["installed"]]
    if not runtime["ready"]:
        state = "failed"
        error = "3D Slicer runtime or SlicerAutomatedDentalTools extension is not packaged yet."
    elif missing:
        state = "downloading"
        error = None
    elif not source_path.exists():
        state = "failed"
        error = f"Input path does not exist: {source_path}"
    else:
        state = "queued"
        error = None
    job_dir = _job_dir(job_id, request)
    output_dir = job_dir / "output"
    status = {
        "jobId": job_id,
        "workflowId": workflow_id,
        "state": state,
        "progress": 0.0 if state != "failed" else 1.0,
        "message": error or ("Queued clinical workflow." if state == "queued" else "Required models are being downloaded before clinical execution."),
        "logs": [error] if error else [f"workflow {workflow_id} requires models: {', '.join(sorted(needed))}"],
        "inputHandle": request.get("sourcePath"),
        "jobDir": str(job_dir),
        "outputDir": str(output_dir),
        "outputArtifacts": [],
        "modelStatus": {"missing": missing, "required": sorted(needed)},
        "error": error,
        "createdAt": _now(),
        "updatedAt": _now(),
    }
    with JOBS_LOCK:
        JOBS[job_id] = status
        _write_status(status)
    if state == "queued":
        thread = threading.Thread(target=_run_job, args=(job_id, request), daemon=True)
        thread.start()
    return _public_status(status)


def job_status(job_id: str) -> dict[str, Any]:
    with JOBS_LOCK:
        if job_id in JOBS:
            return _public_status(JOBS[job_id])
    status_path = JOBS_ROOT / job_id / "status.json"
    if status_path.exists():
        return json.loads(status_path.read_text())
    return {
        "jobId": job_id,
        "workflowId": None,
        "state": "failed",
        "progress": 1.0,
        "message": "Unknown Slicer clinical job.",
        "logs": [],
        "outputArtifacts": [],
        "error": "unknown job",
        "updatedAt": _now(),
    }


def cancel_job(job_id: str) -> dict[str, Any]:
    with JOBS_LOCK:
        process = JOB_PROCESSES.get(job_id)
    if process and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
    if job_id in JOBS:
        _append_log(job_id, "Cancellation requested by user.")
        return _set_job(job_id, state="cancelled", progress=1.0, message="Slicer clinical job cancelled.", error=None)
    return job_status(job_id)

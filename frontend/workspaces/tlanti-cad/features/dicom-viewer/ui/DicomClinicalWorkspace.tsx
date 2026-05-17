import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  ArrowLeft,
  Beaker,
  Box,
  BrainCircuit,
  Database,
  Download,
  FolderOpen,
  Layers3,
  PackagePlus,
  PauseCircle,
  PlayCircle,
  RefreshCw,
  ScanLine,
} from 'lucide-react';

import type { CadWorkspaceHostProps } from '@/components/cad/CadWorkspaceHost';
import { ImportClinicalArtifactUseCase, MeshVaultImportUseCase, TauriMeshVault } from '@/core';
import {
  dicomSegmentationStart,
  dicomSegmentationJobStatus,
  dicomSegmentationToMeshStart,
  dicomSeriesImportStart,
  dicomSeriesJobStatus,
  dicomVolumeBuildStart,
  dicomVolumeJobStatus,
  type DicomJobStatus,
  type DicomSegmentationJobStatus,
} from '@/lib/dicom-jobs';
import { ipc } from '@/lib/ipc';
import {
  slicerClinicalJobCancel,
  slicerClinicalJobStart,
  slicerClinicalJobStatus,
  slicerFixturesDownload,
  slicerFixturesStatus,
  slicerModelsDownloadAll,
  slicerModelsStatus,
  slicerRuntimeDownload,
  slicerRuntimeStatus,
  type SlicerClinicalJobStatus,
  type SlicerFixturesStatus,
  type SlicerModelsStatus,
  type SlicerRuntimeStatus,
} from '@/lib/slicer-clinical';

type SidecarStatus = {
  name: string;
  url: string;
  running: boolean;
  pid: number | null;
  lastHealthCheck: number | null;
  lastHealthCheckLatencyMs: number | null;
  uptimeSecs: number | null;
  restartCount: number;
  lastError: string | null;
};

type TrameSession = {
  sessionId: string;
  url: string;
  viewer: string;
  state: string;
  sourcePath: string | null;
};

type ClinicalStepState = 'idle' | 'running' | 'done' | 'failed';

interface ClinicalStep {
  id: string;
  label: string;
  detail: string;
  state: ClinicalStepState;
  message: string;
}

type MeshImportState = {
  artifactPath: string;
  status: 'idle' | 'running' | 'completed' | 'failed';
  message: string;
};

const POLL_INTERVAL_MS = 900;
const SLICER_WORKFLOWS = [
  ['cbct-segmentation', 'CBCT segmentation'],
  ['adult-dental-segmentation', 'Adult dental segmentation'],
  ['pediatric-segmentation', 'Pediatric segmentation'],
  ['universal-labeling', 'Universal labeling'],
  ['cbct-landmarks', 'ALI-CBCT landmarks'],
  ['ios-landmarks', 'ALI-IOS landmarks'],
  ['cbct-orientation', 'ASO-CBCT orientation'],
  ['ios-orientation', 'ASO-IOS orientation'],
  ['cbct-registration', 'AReg-CBCT registration'],
  ['ios-registration', 'AReg-IOS registration'],
  ['canine-localization', 'CLI-C canines'],
] as const;

function statusToStepState(status: DicomJobStatus | DicomSegmentationJobStatus | null): ClinicalStepState {
  if (!status) return 'idle';
  if (status.state === 'completed') return 'done';
  if (status.state === 'failed' || status.state === 'cancelled') return 'failed';
  return 'running';
}

function statusMessage(status: DicomJobStatus | DicomSegmentationJobStatus | null, fallback: string): string {
  if (!status) return fallback;
  if (status.error) return status.error;
  return status.message || `${Math.round(status.progress * 100)}%`;
}

async function waitForJob<T extends DicomJobStatus>(
  initial: T,
  poll: (jobId: string) => Promise<T>,
  onUpdate: (status: T) => void,
): Promise<T> {
  let current = initial;
  onUpdate(current);
  while (current.state === 'queued' || current.state === 'running') {
    await new Promise((resolve) => window.setTimeout(resolve, POLL_INTERVAL_MS));
    current = await poll(current.jobId);
    onUpdate(current);
  }
  if (current.state === 'failed' || current.state === 'cancelled') {
    throw new Error(current.error ?? current.message ?? `DICOM job ${current.jobId} ${current.state}`);
  }
  return current;
}

async function waitForSlicerJob(
  initial: SlicerClinicalJobStatus,
  onUpdate: (status: SlicerClinicalJobStatus) => void,
): Promise<SlicerClinicalJobStatus> {
  let current = initial;
  onUpdate(current);
  while (current.state === 'queued' || current.state === 'running' || current.state === 'downloading') {
    await new Promise((resolve) => window.setTimeout(resolve, POLL_INTERVAL_MS));
    current = await slicerClinicalJobStatus(current.jobId);
    onUpdate(current);
  }
  if (current.state === 'failed' || current.state === 'cancelled') {
    throw new Error(current.error ?? current.message ?? `Slicer clinical job ${current.jobId} ${current.state}`);
  }
  return current;
}

export function DicomClinicalWorkspace({
  caseId,
  language,
  setLanguage,
  themeMode,
  setThemeMode,
  onBackToDb,
}: CadWorkspaceHostProps) {
  const [sourcePath, setSourcePath] = useState('');
  const [sidecar, setSidecar] = useState<SidecarStatus | null>(null);
  const [trameSession, setTrameSession] = useState<TrameSession | null>(null);
  const [importStatus, setImportStatus] = useState<DicomJobStatus | null>(null);
  const [volumeStatus, setVolumeStatus] = useState<DicomJobStatus | null>(null);
  const [segmentationStatus, setSegmentationStatus] = useState<DicomSegmentationJobStatus | null>(null);
  const [meshStatus, setMeshStatus] = useState<DicomSegmentationJobStatus | null>(null);
  const [workflowId, setWorkflowId] = useState<(typeof SLICER_WORKFLOWS)[number][0]>('cbct-segmentation');
  const [runtimeStatus, setRuntimeStatus] = useState<SlicerRuntimeStatus | null>(null);
  const [modelsStatus, setModelsStatus] = useState<SlicerModelsStatus | null>(null);
  const [fixturesStatus, setFixturesStatus] = useState<SlicerFixturesStatus | null>(null);
  const [slicerJobStatus, setSlicerJobStatus] = useState<SlicerClinicalJobStatus | null>(null);
  const [meshImportState, setMeshImportState] = useState<MeshImportState | null>(null);
  const [isDownloadingFixture, setIsDownloadingFixture] = useState(false);
  const [isDownloadingModels, setIsDownloadingModels] = useState(false);
  const [isDownloadingRuntime, setIsDownloadingRuntime] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const meshVaultImportUseCase = useMemo(() => new MeshVaultImportUseCase(new TauriMeshVault()), []);
  const importClinicalArtifactUseCase = useMemo(() => new ImportClinicalArtifactUseCase(meshVaultImportUseCase), [meshVaultImportUseCase]);

  const refreshSidecar = useCallback(async () => {
    const status = await ipc<void, SidecarStatus>('trame_slicer_sidecar_status');
    setSidecar(status);
    if (status.running) {
      const [runtime, models, fixtures] = await Promise.all([slicerRuntimeStatus(), slicerModelsStatus(), slicerFixturesStatus()]);
      setRuntimeStatus(runtime);
      setModelsStatus(models);
      setFixturesStatus(fixtures);
    }
    return status;
  }, []);

  useEffect(() => {
    void refreshSidecar().catch((nextError) => {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    });
  }, [refreshSidecar]);

  const startSidecar = useCallback(async () => {
    setError(null);
    const status = await ipc<void, SidecarStatus>('trame_slicer_sidecar_start', undefined, { timeoutMs: 45_000 });
    setSidecar(status);
  }, []);

  const stopSidecar = useCallback(async () => {
    setError(null);
    const status = await ipc<void, SidecarStatus>('trame_slicer_sidecar_stop');
    setSidecar(status);
  }, []);

  const downloadModels = useCallback(async () => {
    setIsDownloadingModels(true);
    setError(null);
    try {
      await startSidecar();
      await slicerModelsDownloadAll(true);
      setRuntimeStatus(await slicerRuntimeStatus());
      setModelsStatus(await slicerModelsStatus());
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsDownloadingModels(false);
    }
  }, [startSidecar]);

  const preparePublicFixture = useCallback(async () => {
    setIsDownloadingFixture(true);
    setError(null);
    try {
      await startSidecar();
      const status = await slicerFixturesStatus();
      const fixture = status.fixtures.find((item) => item.id === 'amasss-mg-test-scan');
      if (!fixture) {
        throw new Error('Public CBCT fixture amasss-mg-test-scan is not registered.');
      }
      if (!fixture.present || !fixture.validChecksum) {
        await slicerFixturesDownload(fixture.id);
      }
      const refreshed = await slicerFixturesStatus();
      setFixturesStatus(refreshed);
      const readyFixture = refreshed.fixtures.find((item) => item.id === 'amasss-mg-test-scan');
      if (!readyFixture?.present || !readyFixture.validChecksum) {
        throw new Error('Fixture download finished without a valid checksum.');
      }
      setSourcePath(readyFixture.target);
      return readyFixture.target;
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
      throw nextError;
    } finally {
      setIsDownloadingFixture(false);
    }
  }, [startSidecar]);

  const downloadRuntime = useCallback(async () => {
    setIsDownloadingRuntime(true);
    setError(null);
    try {
      await startSidecar();
      await slicerRuntimeDownload();
      setRuntimeStatus(await slicerRuntimeStatus());
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsDownloadingRuntime(false);
    }
  }, [startSidecar]);

  const openTrameSession = useCallback(async (path: string) => {
    const response = await fetch('http://127.0.0.1:17494/trame/session', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sourcePath: path, caseId: caseId ?? 'no-case' }),
    });
    if (!response.ok) {
      throw new Error(`trame session returned HTTP ${response.status}`);
    }
    const payload = (await response.json()) as TrameSession;
    setTrameSession(payload);
  }, [caseId]);

  const runClinicalPipeline = useCallback(async () => {
    const path = sourcePath.trim();
    if (!path) {
      setError('Select a local DICOM folder or seed file path before starting the clinical pipeline.');
      return;
    }

    setIsRunning(true);
    setError(null);
    setImportStatus(null);
    setVolumeStatus(null);
    setSegmentationStatus(null);
    setMeshStatus(null);

    try {
      await startSidecar();
      await openTrameSession(path);

      const imported = await waitForJob(
        await dicomSeriesImportStart({
          caseId: caseId ?? 'no-case',
          sourcePath: path,
          clinicalRole: 'diagnostic-dicom',
        }),
        dicomSeriesJobStatus,
        setImportStatus,
      );

      const manifestPath = imported.manifest?.manifestPath ?? imported.artifactPath;
      if (!manifestPath) {
        throw new Error('DICOM import finished without a manifest path.');
      }

      const volume = await waitForJob(
        await dicomVolumeBuildStart({
          caseId: caseId ?? 'no-case',
          seriesManifestPath: manifestPath,
        }),
        dicomVolumeJobStatus,
        setVolumeStatus,
      );

      const volumeId = volume.artifactPath ?? volume.manifest?.manifestId ?? volume.jobId;
      const segmentation = await waitForJob(
        await dicomSegmentationStart({
          caseId: caseId ?? 'no-case',
          volumeId,
          targetLabel: 'mandible-maxilla-teeth',
          lowerThreshold: 250,
          upperThreshold: 3071,
        }),
        dicomSegmentationJobStatus,
        setSegmentationStatus,
      );

      await waitForJob(
        await dicomSegmentationToMeshStart({
          caseId: caseId ?? 'no-case',
          segmentationJobId: segmentation.jobId,
        }),
        dicomSegmentationJobStatus,
        setMeshStatus,
      );
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsRunning(false);
      void refreshSidecar().catch(() => undefined);
    }
  }, [caseId, openTrameSession, refreshSidecar, sourcePath, startSidecar]);

  const runSlicerClinicalJob = useCallback(async () => {
    const path = sourcePath.trim();
    if (!path) {
      setError('Select a local DICOM/NIfTI/IOS path before starting the Slicer clinical job.');
      return;
    }

    setIsRunning(true);
    setError(null);
    setSlicerJobStatus(null);

    try {
      await startSidecar();
      await openTrameSession(path);
      setRuntimeStatus(await slicerRuntimeStatus());
      setModelsStatus(await slicerModelsStatus());
      const status = await slicerClinicalJobStart({
        caseId: caseId ?? 'no-case',
        workflowId,
        sourcePath: path,
        options: { shell: 'tlanticad-dicom-clinical-workspace' },
      });
      await waitForSlicerJob(status, setSlicerJobStatus);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsRunning(false);
      void refreshSidecar().catch(() => undefined);
    }
  }, [caseId, openTrameSession, refreshSidecar, sourcePath, startSidecar, workflowId]);

  const runFixtureSmoke = useCallback(async () => {
    try {
      setWorkflowId('adult-dental-segmentation');
      const path = (await preparePublicFixture()) ?? sourcePath.trim();
      setIsRunning(true);
      setError(null);
      setSlicerJobStatus(null);

      await startSidecar();
      await openTrameSession(path);
      setRuntimeStatus(await slicerRuntimeStatus());
      setModelsStatus(await slicerModelsStatus());
      const status = await slicerClinicalJobStart({
        caseId: caseId ?? 'no-case',
        workflowId: 'adult-dental-segmentation',
        sourcePath: path,
        options: {
          shell: 'tlanticad-dicom-clinical-workspace',
          smokeFixtureId: 'amasss-mg-test-scan',
          smoke: true,
        },
      });
      await waitForSlicerJob(status, setSlicerJobStatus);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsRunning(false);
      void refreshSidecar().catch(() => undefined);
    }
  }, [caseId, openTrameSession, preparePublicFixture, refreshSidecar, sourcePath, startSidecar]);

  const cancelSlicerClinicalJob = useCallback(async () => {
    if (!slicerJobStatus?.jobId) {
      return;
    }
    setError(null);
    try {
      setSlicerJobStatus(await slicerClinicalJobCancel(slicerJobStatus.jobId));
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    } finally {
      setIsRunning(false);
    }
  }, [slicerJobStatus?.jobId]);

  const importArtifactToMeshVault = useCallback(async (artifactPath: string, artifactId?: string) => {
    if (!caseId) {
      setError('Open a clinical case before importing a mesh artifact into Mesh Vault.');
      return;
    }
    setMeshImportState({
      artifactPath,
      status: 'running',
      message: 'Importing artifact into Mesh Vault.',
    });
    setError(null);
    try {
      const result = await importClinicalArtifactUseCase.execute({
        caseId,
        artifactPath,
        displayName: artifactPath.split(/[\\/]/).filter(Boolean).at(-1) ?? 'clinical-segmentation.stl',
        moduleId: 'dicom',
        sourceJobId: slicerJobStatus?.jobId ?? artifactId,
        artifactKind: 'stl',
      });
      setMeshImportState({
        artifactPath,
        status: 'completed',
        message: `Mesh Vault handle ready: ${result.file.meshVault?.meshKey ?? result.jobId}`,
      });
    } catch (nextError) {
      const message = nextError instanceof Error ? nextError.message : String(nextError);
      setMeshImportState({
        artifactPath,
        status: 'failed',
        message,
      });
      setError(message);
    }
  }, [caseId, importClinicalArtifactUseCase, slicerJobStatus?.jobId]);

  const publicFixture = fixturesStatus?.fixtures.find((item) => item.id === 'amasss-mg-test-scan') ?? null;
  const stlArtifacts = slicerJobStatus?.outputArtifacts.filter((artifact) => String(artifact.path ?? '').toLowerCase().endsWith('.stl')) ?? [];

  const steps = useMemo<ClinicalStep[]>(() => [
    {
      id: 'sidecar',
      label: 'FastAPI / trame-slicer',
      detail: 'Tauri-supervised local medical sidecar.',
      state: sidecar?.running ? 'done' : 'idle',
      message: sidecar?.running ? `${sidecar.url} healthy` : sidecar?.lastError ?? 'Not started',
    },
    {
      id: 'import',
      label: 'DICOM import',
      detail: 'Rust/Python owns file IO and manifest generation.',
      state: statusToStepState(importStatus),
      message: statusMessage(importStatus, 'Waiting for source path'),
    },
    {
      id: 'volume',
      label: 'VTK volume build',
      detail: 'Build volume artifact before AI work reaches React.',
      state: statusToStepState(volumeStatus),
      message: statusMessage(volumeStatus, 'Waiting for manifest'),
    },
    {
      id: 'segmentation',
      label: 'SlicerAutomatedDentalTools',
      detail: 'AMASSS, BatchDentalSegmentator, ALI, ASO, AReg and CLI-C job boundary.',
      state: slicerJobStatus ? (slicerJobStatus.state === 'completed' ? 'done' : slicerJobStatus.state === 'failed' || slicerJobStatus.state === 'cancelled' ? 'failed' : 'running') : statusToStepState(segmentationStatus),
      message: slicerJobStatus?.message ?? statusMessage(segmentationStatus, 'Waiting for clinical Slicer job'),
    },
    {
      id: 'mesh',
      label: 'Mesh handoff',
      detail: 'Clinical artifact handle for CAD and surgical guide.',
      state: statusToStepState(meshStatus),
      message: statusMessage(meshStatus, 'Waiting for segmentation'),
    },
  ], [importStatus, meshStatus, segmentationStatus, sidecar, slicerJobStatus, volumeStatus]);

  return (
    <div className="flex h-dvh w-full flex-col bg-[#f2f4f7] text-slate-950">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-slate-200 bg-white px-3">
        <div className="flex items-center gap-2">
          <button type="button" onClick={onBackToDb} className="grid size-9 place-items-center rounded-md text-slate-500 hover:bg-slate-100" aria-label="Back to TlantiDB">
            <ArrowLeft className="size-4" />
          </button>
          <div>
            <p className="text-sm font-semibold">DICOM Clinical Workspace</p>
            <p className="text-[11px] text-slate-500">VTK + trame-slicer sidecar, Tauri job handles, no browser pixel-buffer contract</p>
          </div>
        </div>
        <div className="flex items-center gap-1">
          <button type="button" onClick={() => void refreshSidecar()} className="grid size-9 place-items-center rounded-md text-slate-600 hover:bg-slate-100" title="Refresh sidecar">
            <RefreshCw className="size-4" />
          </button>
          <button type="button" onClick={() => setThemeMode(themeMode === 'dark' ? 'light' : 'dark')} className="h-9 rounded-md px-3 text-xs font-medium text-slate-600 hover:bg-slate-100">
            {themeMode === 'dark' ? 'Light' : 'Dark'}
          </button>
          <button type="button" onClick={() => setLanguage(language === 'es' ? 'en' : 'es')} className="h-9 rounded-md px-3 text-xs font-medium text-slate-600 hover:bg-slate-100">
            {language.toUpperCase()}
          </button>
        </div>
      </header>

      <main className="grid min-h-0 flex-1 grid-cols-[340px_minmax(0,1fr)_320px]">
        <aside className="min-h-0 overflow-y-auto border-r border-slate-200 bg-white p-4">
          <section className="space-y-3">
            <div>
              <h2 className="text-sm font-semibold">Local study source</h2>
              <p className="mt-1 text-xs leading-5 text-slate-500">Use a folder or DICOM seed file. React sends a path and receives job state, manifests and artifact handles.</p>
            </div>
            <label className="block text-xs font-medium text-slate-600" htmlFor="dicom-clinical-source">DICOM path</label>
            <input
              id="dicom-clinical-source"
              value={sourcePath}
              onChange={(event) => setSourcePath(event.target.value)}
              placeholder="/path/to/cbct-or-study"
              className="h-10 w-full rounded-md border border-slate-200 bg-white px-3 text-sm outline-none focus:border-blue-400"
            />
            <div className="rounded-md border border-slate-200 bg-slate-50 p-3">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-xs font-semibold text-slate-800">Public smoke fixture</p>
                  <p className="mt-1 text-[11px] leading-5 text-slate-500">
                    {publicFixture ? `${publicFixture.label} · ${publicFixture.present && publicFixture.validChecksum ? 'ready' : 'not prepared'}` : 'Fixture manifest not loaded'}
                  </p>
                </div>
                <Beaker className="mt-0.5 size-4 text-slate-500" />
              </div>
              <div className="mt-3 grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={() => void preparePublicFixture()}
                  disabled={isDownloadingFixture || isRunning}
                  className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-xs font-semibold text-slate-700 hover:bg-slate-100 disabled:opacity-45"
                >
                  <Download className="size-4" />
                  {isDownloadingFixture ? 'Preparing fixture' : 'Prepare fixture'}
                </button>
                <button
                  type="button"
                  onClick={() => void runFixtureSmoke()}
                  disabled={isDownloadingFixture || isRunning || isDownloadingModels || isDownloadingRuntime}
                  className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-emerald-200 bg-emerald-50 px-3 text-xs font-semibold text-emerald-700 hover:bg-emerald-100 disabled:opacity-45"
                >
                  <PlayCircle className="size-4" />
                  Run smoke
                </button>
              </div>
            </div>
            <label className="block text-xs font-medium text-slate-600" htmlFor="slicer-clinical-workflow">Clinical workflow</label>
            <select
              id="slicer-clinical-workflow"
              value={workflowId}
              onChange={(event) => setWorkflowId(event.target.value as typeof workflowId)}
              className="h-10 w-full rounded-md border border-slate-200 bg-white px-3 text-sm outline-none focus:border-blue-400"
            >
              {SLICER_WORKFLOWS.map(([id, label]) => (
                <option key={id} value={id}>{label}</option>
              ))}
            </select>
            <button
              type="button"
              onClick={() => { if (isRunning && slicerJobStatus?.jobId) { void cancelSlicerClinicalJob(); } else { void runSlicerClinicalJob(); } }}
              disabled={(isRunning && !slicerJobStatus?.jobId) || isDownloadingFixture || isDownloadingModels || isDownloadingRuntime}
              className="inline-flex h-10 w-full items-center justify-center gap-2 rounded-md bg-blue-600 px-3 text-sm font-semibold text-white hover:bg-blue-700 disabled:opacity-45"
            >
              {isRunning ? <PauseCircle className="size-4" /> : <PlayCircle className="size-4" />}
              {isRunning ? 'Cancel Slicer clinical job' : 'Run Slicer clinical job'}
            </button>
            <button type="button" onClick={() => void runClinicalPipeline()} disabled={isRunning || isDownloadingModels || isDownloadingRuntime} className="inline-flex h-10 w-full items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-45">
              <Activity className="size-4" />
              Run local fallback pipeline
            </button>
            {error ? <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">{error}</p> : null}
          </section>

          <section className="mt-6 space-y-2">
            <h2 className="text-sm font-semibold">Sidecar controls</h2>
            <div className="grid grid-cols-2 gap-2">
              <button type="button" onClick={() => void startSidecar()} className="h-9 rounded-md border border-slate-200 bg-white px-3 text-xs font-medium hover:bg-slate-50">Start</button>
              <button type="button" onClick={() => void stopSidecar()} className="h-9 rounded-md border border-slate-200 bg-white px-3 text-xs font-medium hover:bg-slate-50">Stop</button>
            </div>
            <button type="button" onClick={() => void downloadModels()} disabled={isDownloadingModels} className="h-9 w-full rounded-md border border-blue-200 bg-blue-50 px-3 text-xs font-semibold text-blue-700 hover:bg-blue-100 disabled:opacity-45">
              {isDownloadingModels ? 'Downloading clinical models' : 'Download all clinical models'}
            </button>
            <button type="button" onClick={() => void downloadRuntime()} disabled={isDownloadingRuntime} className="h-9 w-full rounded-md border border-slate-200 bg-white px-3 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-45">
              {isDownloadingRuntime ? 'Downloading Slicer runtime' : 'Download Slicer runtime'}
            </button>
            <dl className="grid gap-2 text-xs">
              <div className="rounded-md border border-slate-200 px-3 py-2">
                <dt className="text-slate-400">Status</dt>
                <dd className="font-medium text-slate-700">{sidecar?.running ? 'running' : 'offline'}</dd>
              </div>
              <div className="rounded-md border border-slate-200 px-3 py-2">
                <dt className="text-slate-400">URL</dt>
                <dd className="truncate font-medium text-slate-700">{sidecar?.url ?? 'http://127.0.0.1:17494'}</dd>
              </div>
              <div className="rounded-md border border-slate-200 px-3 py-2">
                <dt className="text-slate-400">Slicer runtime</dt>
                <dd className="truncate font-medium text-slate-700">{runtimeStatus?.ready ? 'packaged' : runtimeStatus?.executablePresent ? 'extension missing' : 'runtime missing'}</dd>
              </div>
              <div className="rounded-md border border-slate-200 px-3 py-2">
                <dt className="text-slate-400">Models</dt>
                <dd className="font-medium text-slate-700">{modelsStatus ? `${modelsStatus.installed}/${modelsStatus.total} installed` : 'not checked'}</dd>
              </div>
              <div className="rounded-md border border-slate-200 px-3 py-2">
                <dt className="text-slate-400">Fixture</dt>
                <dd className="font-medium text-slate-700">{publicFixture?.present && publicFixture.validChecksum ? 'verified' : 'pending'}</dd>
              </div>
            </dl>
          </section>
        </aside>

        <section className="min-h-0 overflow-hidden bg-[#111827] p-4 text-white">
          <div className="grid h-full min-h-[560px] grid-cols-[minmax(0,1.2fr)_minmax(280px,0.8fr)] grid-rows-2 gap-3">
            <div className="relative row-span-2 overflow-hidden rounded-lg border border-white/10 bg-black">
              <div className="absolute left-3 top-3 z-10 rounded-md border border-white/10 bg-white/10 px-2 py-1 text-xs text-slate-200">
                trame-slicer session
              </div>
              {trameSession ? (
                <iframe title="trame-slicer clinical session" src={trameSession.url} className="h-full w-full border-0 bg-black" />
              ) : (
                <div className="grid h-full place-items-center">
                  <div className="max-w-md text-center">
                    <ScanLine className="mx-auto size-12 text-blue-300" />
                    <h2 className="mt-4 text-lg font-semibold">Awaiting local DICOM session</h2>
                    <p className="mt-2 text-sm leading-6 text-slate-400">Start the sidecar and run the pipeline. The viewport is owned by the local Python/trame service; the shell keeps only handles and clinical state.</p>
                  </div>
                </div>
              )}
            </div>
            {[
              ['Axial', Layers3],
              ['Volume', Box],
              ['AI', BrainCircuit],
            ].map(([label, Icon]) => (
              <div key={label as string} className="relative overflow-hidden rounded-lg border border-white/10 bg-slate-950">
                <div className="absolute left-3 top-3 rounded-md bg-white/10 px-2 py-1 text-xs text-slate-200">{label as string}</div>
                <div className="grid h-full place-items-center text-xs text-slate-500">
                  {React.createElement(Icon as typeof Activity, { className: 'mb-2 size-6 text-slate-600' })}
                  Artifact handle pending
                </div>
              </div>
            ))}
          </div>
        </section>

        <aside className="min-h-0 overflow-y-auto border-l border-slate-200 bg-white p-4">
          <h2 className="text-sm font-semibold">Clinical runtime</h2>
          {slicerJobStatus ? (
            <section className="mt-3 rounded-md border border-slate-200 bg-slate-50 p-3 text-xs">
              <div className="flex items-center justify-between gap-3">
                <p className="font-semibold text-slate-800">{slicerJobStatus.workflowId}</p>
                <span className="rounded bg-white px-2 py-0.5 text-[10px] font-semibold uppercase text-slate-600">{slicerJobStatus.state}</span>
              </div>
              <p className="mt-2 text-slate-500">{slicerJobStatus.message}</p>
              <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-slate-200">
                <div className="h-full rounded-full bg-blue-600" style={{ width: `${Math.round(slicerJobStatus.progress * 100)}%` }} />
              </div>
              {stlArtifacts.length > 0 ? (
                <div className="mt-3 space-y-1">
                  <p className="font-medium text-slate-700">Artifacts</p>
                  {stlArtifacts.map((artifact) => {
                    const artifactPath = String(artifact.path ?? '');
                    const isImportingThis = meshImportState?.artifactPath === artifactPath && meshImportState.status === 'running';
                    return (
                      <div key={String(artifact.id ?? artifact.path)} className="rounded-md border border-slate-200 bg-white px-2 py-2">
                        <p className="truncate text-slate-600">{artifactPath}</p>
                        <div className="mt-2 flex gap-2">
                          <button
                            type="button"
                            onClick={() => void importArtifactToMeshVault(artifactPath, String(artifact.id ?? artifact.path))}
                            disabled={isImportingThis}
                            className="inline-flex h-8 items-center justify-center gap-2 rounded-md border border-slate-200 bg-slate-50 px-2 text-[11px] font-semibold text-slate-700 hover:bg-slate-100 disabled:opacity-45"
                          >
                            <PackagePlus className="size-3.5" />
                            {isImportingThis ? 'Importing' : 'Import to Mesh Vault'}
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : null}
              {meshImportState ? (
                <div className="mt-3 rounded-md border border-slate-200 bg-white px-3 py-2 text-[11px] text-slate-600">
                  <div className="flex items-center gap-2">
                    <Database className="size-3.5 text-slate-500" />
                    <span className="font-semibold text-slate-700">Mesh Vault handoff</span>
                  </div>
                  <p className="mt-1">{meshImportState.message}</p>
                </div>
              ) : null}
              {slicerJobStatus.logs.length > 0 ? (
                <pre className="mt-3 max-h-56 overflow-auto rounded-md bg-slate-950 p-2 text-[10px] leading-4 text-slate-200">{slicerJobStatus.logs.join('\n')}</pre>
              ) : null}
            </section>
          ) : null}
          <div className="mt-3 space-y-2">
            {steps.map((step) => (
              <div key={step.id} className="rounded-md border border-slate-200 bg-white px-3 py-3 text-xs">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className="font-semibold text-slate-800">{step.label}</p>
                    <p className="mt-1 leading-5 text-slate-500">{step.detail}</p>
                  </div>
                  <span className={[
                    'rounded px-2 py-0.5 text-[10px] font-semibold uppercase',
                    step.state === 'done' ? 'bg-emerald-50 text-emerald-700' : '',
                    step.state === 'running' ? 'bg-blue-50 text-blue-700' : '',
                    step.state === 'failed' ? 'bg-red-50 text-red-700' : '',
                    step.state === 'idle' ? 'bg-slate-100 text-slate-500' : '',
                  ].join(' ')}>
                    {step.state}
                  </span>
                </div>
                <p className="mt-2 truncate text-slate-500">{step.message}</p>
              </div>
            ))}
          </div>
        </aside>
      </main>
    </div>
  );
}

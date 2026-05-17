import React, { Suspense, lazy, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  ArrowLeft,
  Bot,
  Box,
  Camera,
  Check,
  ChevronLeft,
  CircleAlert,
  Cuboid,
  Download,
  Eye,
  EyeOff,
  Grid3X3,
  Info,
  Layers3,
  Loader2,
  Moon,
  Move3D,
  PanelLeftClose,
  Ruler,
  RotateCcw,
  RotateCw,
  Save,
  Scissors,
  Settings,
  Sparkles,
  Sun,
  Undo2,
  Waypoints,
} from 'lucide-react';

import type { Language, ThemeMode } from '@/types';
import { CAD_TOOL_REGISTRY, getCadToolDefinition } from '@/lib/cad-tool-registry';
import { isTauriRuntime } from '@/lib/desktop-system';
import {
  exportCrownManifest,
  getCrownJobStatus,
  startCrownJob,
  type CrownJobStatusResponse,
} from '@/lib/crown-workflow';
import { createTauriCadOrchestratorAdapter } from '../infrastructure/tauri-cad-orchestrator-adapter';
import { cadSceneStore, useCadSceneStore } from '../state/cad-scene-store';
import type { CadSceneTool } from '../domain/cad-scene';

const CadViewportShell = lazy(() => import('./CadViewportShell').then((module) => ({ default: module.CadViewportShell })));

export interface CadDesktopShellProps {
  language: Language;
  setLanguage: (lang: Language) => void;
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  caseId?: string;
  moduleId?: string;
  onBackToDb: () => void;
}

const toolIcons: Record<CadSceneTool, React.ComponentType<{ className?: string }>> = {
  select: Box,
  inspect: Info,
  section: Scissors,
  measure: Ruler,
};

type CrownClinicalStageStatus = 'pending' | 'active' | 'done' | 'blocked';

interface CrownClinicalStage {
  id: string;
  label: string;
  detail: string;
  status: CrownClinicalStageStatus;
}

interface CrownUiState {
  alignmentOpen: boolean;
  crownJob: CrownJobStatusResponse | null;
  error: string | null;
  exportManifestPath: string | null;
  running: boolean;
}

interface CadShellQaApi {
  getSnapshot: typeof cadSceneStore.getState;
  getToolRegistry: () => typeof CAD_TOOL_REGISTRY;
  getClinicalWorkflow: () => {
    workflow: 'crown';
    stageCount: number;
    exportBlockedWithoutDesktop: boolean;
    fixtureCaseId: string;
  };
  setTool: (tool: CadSceneTool) => void;
  resetFixtureScene: () => void;
  toggleGrid: () => void;
}

declare global {
  interface Window {
    __TLANTICAD_CAD_SHELL_QA__?: CadShellQaApi;
  }
}

function cx(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(' ');
}

function stageTone(status: CrownClinicalStageStatus) {
  if (status === 'done') return 'border-emerald-200 bg-emerald-50 text-emerald-700';
  if (status === 'active') return 'border-blue-200 bg-blue-50 text-blue-700';
  if (status === 'blocked') return 'border-red-200 bg-red-50 text-red-700';
  return 'border-slate-200 bg-white text-slate-500';
}

function CrownAlignmentModal({
  open,
  onClose,
  onConfirm,
}: {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
}) {
  if (!open) return null;

  const views = ['Frontal', 'Lateral R', 'Occlusal', 'Lateral L'];

  return (
    <div className="absolute inset-0 z-40 grid place-items-center bg-slate-950/55 backdrop-blur-[2px]" data-testid="crown-alignment-modal">
      <section className="w-[min(1180px,calc(100vw-48px))] rounded-[10px] border border-slate-200 bg-white shadow-2xl">
        <header className="flex items-center gap-4 border-b border-slate-200 px-5 py-4">
          <button
            type="button"
            onClick={onClose}
            className="grid h-10 w-10 place-items-center rounded-lg border border-slate-200 bg-slate-50 text-slate-500 hover:bg-slate-100"
            aria-label="Back"
          >
            <ChevronLeft className="h-5 w-5" />
          </button>
          <div>
            <h2 className="text-base font-semibold text-blue-700">Occlusal Alignment (Fine-Tuning)</h2>
            <p className="mt-1 text-sm text-slate-500">Fine-tune model alignment before crown generation.</p>
          </div>
        </header>
        <div className="grid gap-1 p-4 md:grid-cols-2">
          {views.map((view, index) => (
            <div key={view} className="relative aspect-[16/8.4] overflow-hidden rounded-lg border border-blue-100 bg-[#eef5ff]">
              <div className="absolute inset-0 bg-[linear-gradient(#c9dcf8_1px,transparent_1px),linear-gradient(90deg,#c9dcf8_1px,transparent_1px)] bg-[size:36px_36px] opacity-70" />
              <div
                className={cx(
                  'absolute rounded-[50%] border border-emerald-700/20 bg-emerald-300/75 shadow-xl',
                  index === 2 ? 'left-[32%] top-[26%] h-[48%] w-[34%]' : 'left-[34%] top-[39%] h-[20%] w-[34%]',
                )}
              />
              <div className="absolute bottom-3 right-3 rounded-full border border-slate-200 bg-white/75 px-2 py-1 text-[11px] font-medium text-slate-500">
                {view}
              </div>
            </div>
          ))}
        </div>
        <footer className="flex items-center justify-between border-t border-slate-200 px-5 py-4">
          <div className="flex items-center gap-4 text-sm text-slate-500">
            <span>Rotate <kbd className="rounded border border-slate-200 bg-slate-50 px-1.5 py-0.5">Mouse</kbd></span>
            <span>Translate <kbd className="rounded border border-slate-200 bg-slate-50 px-1.5 py-0.5">Shift</kbd></span>
            <span>Reset Position <kbd className="rounded border border-slate-200 bg-slate-50 px-1.5 py-0.5">R</kbd></span>
          </div>
          <button
            type="button"
            onClick={onConfirm}
            className="rounded-lg bg-blue-600 px-5 py-3 text-sm font-semibold text-white shadow-sm hover:bg-blue-700"
            data-testid="confirm-alignment"
          >
            Confirm Alignment
          </button>
        </footer>
      </section>
    </div>
  );
}

export function CadDesktopShell({
  language,
  setLanguage,
  themeMode,
  setThemeMode,
  caseId,
  moduleId,
  onBackToDb,
}: CadDesktopShellProps) {
  const orchestrator = useMemo(() => createTauriCadOrchestratorAdapter(), []);
  const activeTool = useCadSceneStore((state) => state.activeTool);
  const selectedEntityId = useCadSceneStore((state) => state.selectedEntityId);
  const gridVisible = useCadSceneStore((state) => state.gridVisible);
  const entities = useCadSceneStore((state) => state.entities);
  const bootstrap = useCadSceneStore((state) => state.bootstrap);
  const bootstrapStatus = useCadSceneStore((state) => state.bootstrapStatus);
  const bootstrapError = useCadSceneStore((state) => state.bootstrapError);
  const performance = useCadSceneStore((state) => state.performance);
  const setActiveTool = useCadSceneStore((state) => state.setActiveTool);
  const selectEntity = useCadSceneStore((state) => state.selectEntity);
  const toggleGrid = useCadSceneStore((state) => state.toggleGrid);
  const toggleEntityVisibility = useCadSceneStore((state) => state.toggleEntityVisibility);
  const resetFixtureScene = useCadSceneStore((state) => state.resetFixtureScene);
  const setBootstrapLoading = useCadSceneStore((state) => state.setBootstrapLoading);
  const setBootstrapReady = useCadSceneStore((state) => state.setBootstrapReady);
  const setBootstrapError = useCadSceneStore((state) => state.setBootstrapError);
  const [crownState, setCrownState] = useState<CrownUiState>({
    alignmentOpen: true,
    crownJob: null,
    error: null,
    exportManifestPath: null,
    running: false,
  });
  const [alignmentConfirmed, setAlignmentConfirmed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setBootstrapLoading();
    orchestrator.bootstrapShell({ caseId, moduleId })
      .then((result) => {
        if (!cancelled) setBootstrapReady(result);
      })
      .catch((error) => {
        if (!cancelled) setBootstrapError(error instanceof Error ? error.message : String(error));
      });

    return () => {
      cancelled = true;
    };
  }, [caseId, moduleId, orchestrator, setBootstrapError, setBootstrapLoading, setBootstrapReady]);

  useEffect(() => {
    const qaApi: CadShellQaApi = {
      getSnapshot: cadSceneStore.getState,
      getToolRegistry: () => CAD_TOOL_REGISTRY,
      getClinicalWorkflow: () => ({
        workflow: 'crown',
        stageCount: 7,
        exportBlockedWithoutDesktop: !isTauriRuntime(),
        fixtureCaseId: caseId ?? 'clinical-hardening-smoke',
      }),
      setTool: (tool) => cadSceneStore.getState().setActiveTool(tool),
      resetFixtureScene: () => cadSceneStore.getState().resetFixtureScene(),
      toggleGrid: () => cadSceneStore.getState().toggleGrid(),
    };

    window.__TLANTICAD_CAD_SHELL_QA__ = qaApi;

    return () => {
      if (window.__TLANTICAD_CAD_SHELL_QA__ === qaApi) {
        delete window.__TLANTICAD_CAD_SHELL_QA__;
      }
    };
  }, [caseId]);

  const selectedEntity = useMemo(
    () => entities.find((entity) => entity.id === selectedEntityId) ?? null,
    [entities, selectedEntityId],
  );

  const crownStages = useMemo<CrownClinicalStage[]>(() => {
    const completed = crownState.crownJob?.phase === 'completed';
    const failed = crownState.crownJob?.phase === 'failed' || crownState.error !== null;
    return [
      { id: 'case', label: 'Case', detail: caseId ?? 'clinical-hardening-smoke', status: 'done' },
      { id: 'asset', label: 'Prep Scan', detail: 'STL handle + metadata', status: 'done' },
      { id: 'alignment', label: 'Occlusal Alignment', detail: alignmentConfirmed ? 'confirmed' : 'fine-tune required', status: alignmentConfirmed ? 'done' : 'active' },
      { id: 'margin', label: 'Margin', detail: 'local preview ready', status: alignmentConfirmed ? 'done' : 'pending' },
      { id: 'axis', label: 'Insertion Axis', detail: '+Z occlusal hint', status: alignmentConfirmed ? 'done' : 'pending' },
      { id: 'crown', label: 'Crown Job', detail: crownState.crownJob?.phase ?? 'not started', status: completed ? 'done' : failed ? 'blocked' : crownState.running ? 'active' : 'pending' },
      { id: 'export', label: 'Manifest', detail: crownState.exportManifestPath ?? 'blocked until real artifact', status: crownState.exportManifestPath ? 'done' : completed ? 'active' : 'pending' },
    ];
  }, [alignmentConfirmed, caseId, crownState.crownJob?.phase, crownState.error, crownState.exportManifestPath, crownState.running]);

  async function runCrownClinicalSlice() {
    if (!isTauriRuntime()) {
      setCrownState((state) => ({
        ...state,
        error: 'Desktop runtime required: browser smoke can verify UI, but clinical export is blocked without Tauri/Rust.',
      }));
      return;
    }

    setCrownState((state) => ({ ...state, error: null, running: true }));
    try {
      const smokeRoot = '.tlanticad/clinical-hardening-smoke';
      const start = await startCrownJob({
        config: {
          fdi: 26,
          material: 'hybrid-ceramic',
          occlusalHint: [0, 0, 1],
          marginCurvatureThreshold: 0.18,
          libraryTargetHeightMm: 9.5,
        },
        prepStl: 'src-tauri/tools/Teeth78.stl',
        libraryToothStl: 'src-tauri/tools/Tooth_Libraries/GenericSolid2_spare.stl',
        outputOuterStl: `${smokeRoot}/crown-26-output.stl`,
      }, `${smokeRoot}/crown-26-job-manifest.json`);

      let status = start;
      for (let poll = 0; poll < 80 && !['completed', 'failed', 'cancelled'].includes(status.phase); poll += 1) {
        await new Promise((resolve) => window.setTimeout(resolve, 250));
        status = await getCrownJobStatus(start.jobId);
        setCrownState((state) => ({ ...state, crownJob: status }));
      }

      if (status.phase !== 'completed') {
        throw new Error(status.error ?? `Crown job ended as ${status.phase}`);
      }

      const manifestPath = `${smokeRoot}/crown-26-export-manifest.json`;
      await exportCrownManifest(status.jobId, manifestPath);
      setCrownState((state) => ({
        ...state,
        crownJob: status,
        exportManifestPath: manifestPath,
        running: false,
      }));
    } catch (error) {
      setCrownState((state) => ({
        ...state,
        error: error instanceof Error ? error.message : String(error),
        running: false,
      }));
    }
  }

  const capabilityItems = [
    ['Rust mesh', bootstrap?.capabilities.rustMeshOps],
    ['Python AI', bootstrap?.capabilities.pythonAi],
    ['DICOM', bootstrap?.capabilities.dicomPipeline],
    ['Export', bootstrap?.capabilities.exportPipeline],
  ] as const;

  return (
    <main className="flex h-screen min-h-0 flex-col bg-[#f7f8fb] text-slate-900" data-testid="cad-desktop-shell">
      <header className="flex h-[54px] shrink-0 items-center border-b border-slate-200 bg-white px-4">
        <div className="flex min-w-0 flex-1 items-center gap-3">
          <button type="button" onClick={onBackToDb} className="grid h-9 w-9 place-items-center rounded-md text-slate-500 hover:bg-slate-100" aria-label="Back to Workspace">
            <ArrowLeft className="h-4 w-4" />
          </button>
          <div className="h-5 w-px bg-slate-200" />
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold">TlantiCAD</div>
            <div className="truncate text-[11px] text-slate-500">{caseId ?? 'clinical-hardening-smoke'} / Crown AI / offline workstation</div>
          </div>
        </div>
        <div className="text-center">
          <div className="text-sm font-semibold tracking-wide">ROSA SERRANO</div>
          <div className="text-[11px] text-blue-700">ID: 2026-1485</div>
        </div>
        <div className="flex flex-1 items-center justify-end gap-1">
          {[
            [Undo2, 'Anular'],
            [RotateCw, 'Rehacer'],
            [Info, 'Info'],
            [Settings, 'Configuraciones'],
            [Download, 'Exportar'],
          ].map(([Icon, label]) => (
            <button key={label as string} type="button" className="grid h-9 min-w-14 place-items-center rounded-md text-[10px] text-slate-600 hover:bg-slate-100" title={label as string}>
              <Icon className="h-4 w-4" />
              <span>{label as string}</span>
            </button>
          ))}
          <button
            type="button"
            onClick={() => setThemeMode(themeMode === 'dark' ? 'light' : 'dark')}
            className="grid h-9 w-9 place-items-center rounded-md text-slate-600 hover:bg-slate-100"
            aria-label="Toggle theme"
          >
            {themeMode === 'dark' ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
          </button>
          <button type="button" onClick={() => setLanguage(language === 'es' ? 'en' : 'es')} className="h-9 rounded-md px-2 text-xs font-medium text-slate-600 hover:bg-slate-100">
            {language.toUpperCase()}
          </button>
        </div>
      </header>

      <section className="grid min-h-0 flex-1 grid-cols-[78px_294px_minmax(0,1fr)]">
        <nav className="flex min-h-0 flex-col gap-2 border-r border-slate-200 bg-white px-3 py-4">
          {([
            [Cuboid, 'Planificación', true],
            [Sparkles, 'Crown AI', false],
            [Waypoints, 'Guía quirúrgica', false],
            [Ruler, 'Medir', false],
            [Layers3, 'Densidad', false],
          ] as Array<[React.ComponentType<{ className?: string }>, string, boolean]>).map(([Icon, label, active]) => (
            <button
              key={label}
              type="button"
              className={cx(
                'grid h-[58px] place-items-center rounded-lg border text-[11px]',
                active ? 'border-blue-200 bg-blue-50 text-blue-700' : 'border-transparent text-slate-500 hover:bg-slate-50',
              )}
            >
              <Icon className="h-5 w-5" />
              <span className="truncate">{label}</span>
            </button>
          ))}
          <div className="mt-auto grid h-9 place-items-center rounded-full bg-slate-100 text-slate-500">
            <PanelLeftClose className="h-4 w-4" />
          </div>
        </nav>

        <aside className="min-h-0 overflow-y-auto border-r border-slate-200 bg-white">
          <section className="border-b border-slate-200 p-3">
            <div className="mb-2 flex items-center justify-between">
              <h2 className="text-sm font-semibold">Models</h2>
              <button type="button" className="rounded-md px-2 py-1 text-sm font-semibold text-blue-600 hover:bg-blue-50">Add +</button>
            </div>
            {entities.filter((entity) => entity.kind === 'mesh').map((entity) => (
              <div
                key={entity.id}
                className={cx(
                  'mb-2 rounded-md border px-2 py-2',
                  selectedEntityId === entity.id ? 'border-blue-500 bg-blue-50' : 'border-slate-200 bg-white',
                )}
              >
                <div className="flex items-center gap-2">
                  <span className="h-3 w-3 rounded bg-emerald-300" />
                  <button type="button" onClick={() => selectEntity(entity.id)} className="min-w-0 flex-1 truncate text-left text-sm text-blue-700">
                    {entity.label}
                  </button>
                  <button type="button" onClick={() => toggleEntityVisibility(entity.id)} className="text-slate-400">
                    {entity.visible ? <Eye className="h-4 w-4" /> : <EyeOff className="h-4 w-4" />}
                  </button>
                </div>
              </div>
            ))}
          </section>

          <section className="border-b border-slate-200 p-4">
            <div className="mb-3 flex items-center gap-2">
              <div className="h-px flex-1 bg-slate-200" />
              <span className="text-sm font-medium text-slate-500">Start</span>
              <div className="h-px flex-1 bg-slate-200" />
            </div>
            <div className="grid grid-cols-2 gap-2">
              <button type="button" onClick={() => setCrownState((state) => ({ ...state, alignmentOpen: true }))} className="rounded-lg bg-slate-100 p-3 text-sm font-medium text-slate-600 hover:bg-blue-50 hover:text-blue-700" data-testid="open-crown-alignment">
                <Sparkles className="mx-auto mb-2 h-6 w-6" />
                Crown AI
              </button>
              <button type="button" className="rounded-lg bg-slate-100 p-3 text-sm font-medium text-slate-600">
                <Box className="mx-auto mb-2 h-6 w-6" />
                Splint
              </button>
              <button type="button" className="rounded-lg bg-slate-100 p-3 text-sm font-medium text-slate-600">
                <Layers3 className="mx-auto mb-2 h-6 w-6" />
                Model Build
              </button>
            </div>
          </section>

          <section className="p-3">
            <div className="mb-3 grid grid-cols-2 rounded-lg border border-slate-200 bg-slate-50 p-1">
              <button type="button" className="rounded-md bg-white py-2 text-sm font-medium text-blue-700 shadow-sm">Main Tools</button>
              <button type="button" className="rounded-md py-2 text-sm font-medium text-slate-500">Advanced Tools</button>
            </div>
            <div className="grid grid-cols-3 gap-2">
              {CAD_TOOL_REGISTRY.map((item) => {
                const Icon = toolIcons[item.id];
                const active = activeTool === item.id;
                return (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => setActiveTool(item.id)}
                    className={cx(
                      'h-[66px] rounded-lg border text-[11px]',
                      active ? 'border-blue-400 bg-blue-50 text-blue-700' : 'border-slate-100 bg-white text-slate-500 hover:border-slate-200',
                    )}
                    title={`${item.label} (${item.hotkey}) - ${item.workflowPhase}`}
                    data-tool-id={item.id}
                  >
                    <Icon className="mx-auto mb-1 h-5 w-5" />
                    <span>{item.label}</span>
                  </button>
                );
              })}
              <button type="button" onClick={toggleGrid} className={cx('h-[66px] rounded-lg border text-[11px]', gridVisible ? 'border-blue-400 bg-blue-50 text-blue-700' : 'border-slate-100 bg-white text-slate-500')}>
                <Grid3X3 className="mx-auto mb-1 h-5 w-5" />
                Mesh
              </button>
            </div>
          </section>
        </aside>

        <div className="relative min-h-0 bg-[#f7f8fb]" data-testid="cad-viewport-region">
          <div className="absolute left-8 top-8 z-20 space-y-2 text-sm text-slate-500">
            <input type="range" min={0} max={100} defaultValue={78} className="w-36 accent-blue-500" aria-label="Model opacity" />
            <div className="flex gap-2">
              <button type="button" className="grid h-9 w-9 place-items-center rounded-lg bg-blue-100 text-blue-700"><Cuboid className="h-5 w-5" /></button>
              <button type="button" className="grid h-9 w-9 place-items-center rounded-lg bg-white text-slate-400 shadow-sm"><Box className="h-5 w-5" /></button>
              <button type="button" className="grid h-9 w-9 place-items-center rounded-lg bg-white text-slate-400 shadow-sm"><Grid3X3 className="h-5 w-5" /></button>
            </div>
            <div className="space-y-1">
              <div>▾ 150,154</div>
              <div>◌ 299,510</div>
              <div className="inline-flex items-center gap-2 rounded-lg border border-red-300 bg-white px-3 py-2 font-medium text-red-500">
                1 Hole <CircleAlert className="h-4 w-4" />
              </div>
            </div>
          </div>

          <div className="absolute left-1/2 top-6 z-20 flex -translate-x-1/2 items-center gap-1 rounded-xl border border-slate-200 bg-white/90 px-3 py-2 shadow-sm backdrop-blur">
            <button type="button" className="rounded-md border border-slate-200 px-3 py-2 text-sm font-medium text-slate-400">Clear Selection</button>
            {CAD_TOOL_REGISTRY.map((item) => {
              const Icon = toolIcons[item.id];
              return (
                <button key={item.id} type="button" onClick={() => setActiveTool(item.id)} className={cx('grid h-9 w-9 place-items-center rounded-md', activeTool === item.id ? 'bg-blue-100 text-blue-700' : 'text-slate-500 hover:bg-slate-100')}>
                  <Icon className="h-4 w-4" />
                </button>
              );
            })}
            <div className="ml-2 h-5 w-36 rounded-full bg-slate-100">
              <div className="h-full w-[68%] rounded-full bg-gradient-to-r from-blue-300 to-blue-600" />
            </div>
          </div>

          <Suspense
            fallback={
              <div className="grid h-full place-items-center text-sm text-slate-400">
                <div className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Loading local CAD viewport
                </div>
              </div>
            }
          >
            <CadViewportShell />
          </Suspense>

          <div className="absolute bottom-16 right-8 z-20 grid place-items-center gap-1 text-xs font-semibold">
            <span className="rounded-full bg-cyan-400 px-2 py-1 text-white">H</span>
            <div className="grid h-14 w-14 place-items-center rounded-full border border-slate-200 bg-white/90 shadow-sm">HEAD</div>
            <div className="flex gap-8 text-white">
              <span className="rounded-full bg-emerald-400 px-2 py-1">R</span>
              <span className="rounded-full bg-emerald-400 px-2 py-1">L</span>
            </div>
            <span className="rounded-full bg-sky-400 px-2 py-1 text-white">F</span>
          </div>

          <div className="absolute bottom-0 left-0 right-0 z-20 flex h-10 items-center justify-center gap-8 border-t border-slate-200 bg-white text-sm text-slate-500">
            <span>Zoom <kbd className="rounded border border-slate-200 px-1">Mouse</kbd></span>
            <span>Pan <kbd className="rounded border border-slate-200 px-1">Shift</kbd></span>
            <span>Rotate 3D <kbd className="rounded border border-slate-200 px-1">Mouse</kbd></span>
            <button type="button" onClick={resetFixtureScene}>Reset View <kbd className="rounded border border-slate-200 px-1">R</kbd></button>
          </div>

          <section className="absolute bottom-14 left-8 z-30 w-[min(520px,calc(100vw-430px))] rounded-xl border border-slate-200 bg-white/95 p-3 shadow-lg backdrop-blur" data-testid="clinical-crown-panel">
            <div className="mb-2 flex items-center justify-between">
              <div>
                <div className="text-sm font-semibold text-slate-900">Crown clinical smoke</div>
                <div className="text-xs text-slate-500">Case {'->'} Tooth {'->'} Prep scan {'->'} Margin {'->'} Axis {'->'} Artifact {'->'} Manifest</div>
              </div>
              <button type="button" onClick={runCrownClinicalSlice} className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-700" data-testid="run-crown-slice">
                {crownState.running ? <Loader2 className="h-4 w-4 animate-spin" /> : <Sparkles className="h-4 w-4" />}
                Run Crown
              </button>
            </div>
            <div className="grid grid-cols-7 gap-1">
              {crownStages.map((stage) => (
                <div key={stage.id} className={cx('rounded-md border px-2 py-1.5', stageTone(stage.status))}>
                  <div className="truncate text-[11px] font-semibold">{stage.label}</div>
                  <div className="truncate text-[10px] opacity-80">{stage.detail}</div>
                </div>
              ))}
            </div>
            {crownState.error && (
              <div className="mt-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700" data-testid="crown-export-blocked">
                {crownState.error}
              </div>
            )}
          </section>

          <CrownAlignmentModal
            open={crownState.alignmentOpen}
            onClose={() => setCrownState((state) => ({ ...state, alignmentOpen: false }))}
            onConfirm={() => {
              setAlignmentConfirmed(true);
              setCrownState((state) => ({ ...state, alignmentOpen: false, error: null }));
            }}
          />
        </div>
      </section>

      <footer className="flex h-9 shrink-0 items-center justify-between border-t border-slate-200 bg-white px-3 text-xs text-slate-500">
        <div className="flex items-center gap-2">
          {bootstrapStatus === 'loading' ? <Loader2 className="h-3.5 w-3.5 animate-spin text-blue-500" /> : <Activity className="h-3.5 w-3.5 text-emerald-500" />}
          {bootstrapStatus === 'ready' ? bootstrap?.route : bootstrapStatus}
          {bootstrapError ? <span className="text-red-500">{bootstrapError}</span> : null}
        </div>
        <div className="hidden items-center gap-2 md:flex">
          {capabilityItems.map(([label, enabled]) => (
            <span key={label} className={enabled ? 'text-emerald-600' : 'text-slate-400'}>{label}: {enabled ? 'ready' : 'offline'}</span>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <Save className="h-3.5 w-3.5" />
          <Bot className="h-3.5 w-3.5 text-blue-500" />
          <span>{selectedEntity?.label ?? getCadToolDefinition(activeTool).label} / {performance.renderQuality}</span>
          <Camera className="h-3.5 w-3.5" />
        </div>
      </footer>
    </main>
  );
}

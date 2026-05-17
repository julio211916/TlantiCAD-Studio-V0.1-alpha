import React, { Suspense, lazy, useEffect, useMemo } from 'react';
import {
  Activity,
  ArrowLeft,
  Bot,
  Box,
  Braces,
  Crosshair,
  Eye,
  EyeOff,
  Grid3X3,
  Loader2,
  Moon,
  Move3D,
  RotateCcw,
  Ruler,
  Scissors,
  Sun,
} from 'lucide-react';

import type { Language, ThemeMode } from '@/types';
import {
  createTauriCadOrchestratorAdapter,
  useCadSceneStore,
  type CadSceneTool,
} from '@/features/cad-scene';

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

const toolItems: Array<{ id: CadSceneTool; label: string; icon: React.ComponentType<{ className?: string }> }> = [
  { id: 'select', label: 'Select', icon: Crosshair },
  { id: 'inspect', label: 'Inspect', icon: Box },
  { id: 'section', label: 'Section', icon: Scissors },
  { id: 'measure', label: 'Measure', icon: Ruler },
];

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

  const selectedEntity = useMemo(
    () => entities.find((entity) => entity.id === selectedEntityId) ?? null,
    [entities, selectedEntityId],
  );

  const capabilityItems = [
    ['Rust mesh', bootstrap?.capabilities.rustMeshOps],
    ['Python AI', bootstrap?.capabilities.pythonAi],
    ['DICOM', bootstrap?.capabilities.dicomPipeline],
    ['Export', bootstrap?.capabilities.exportPipeline],
  ] as const;

  return (
    <main className="flex h-screen min-h-0 flex-col bg-[#080a0d] text-slate-100" data-testid="cad-desktop-shell">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-slate-800 bg-[#0d1117]/95 px-3">
        <div className="flex min-w-0 items-center gap-2">
          <button
            type="button"
            onClick={onBackToDb}
            className="grid h-9 w-9 place-items-center rounded-md border border-slate-700 bg-slate-900 text-slate-200 hover:bg-slate-800"
            aria-label="Back to Workspace"
          >
            <ArrowLeft className="h-4 w-4" />
          </button>
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold tracking-wide text-slate-100">TlantiCAD Shell</div>
            <div className="truncate text-xs text-slate-400">
              {caseId ? `Case ${caseId}` : 'No case'} / {moduleId ?? 'cad'} / {language.toUpperCase()}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="hidden items-center gap-1 rounded-md border border-slate-800 bg-slate-950 px-2 py-1 text-xs text-slate-400 md:flex">
            {bootstrapStatus === 'loading' ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Activity className="h-3.5 w-3.5" />}
            {bootstrapStatus === 'ready' ? bootstrap?.route : bootstrapStatus}
          </div>
          <button
            type="button"
            onClick={() => setLanguage(language === 'es' ? 'en' : 'es')}
            className="h-9 rounded-md border border-slate-700 bg-slate-900 px-3 text-xs font-medium text-slate-200 hover:bg-slate-800"
          >
            {language.toUpperCase()}
          </button>
          <button
            type="button"
            onClick={() => setThemeMode(themeMode === 'dark' ? 'light' : 'dark')}
            className="grid h-9 w-9 place-items-center rounded-md border border-slate-700 bg-slate-900 text-slate-200 hover:bg-slate-800"
            aria-label="Toggle theme"
          >
            {themeMode === 'dark' ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
          </button>
        </div>
      </header>

      <section className="grid min-h-0 flex-1 grid-cols-[56px_minmax(0,1fr)_320px] max-lg:grid-cols-[52px_minmax(0,1fr)]">
        <nav className="flex min-h-0 flex-col items-center gap-2 border-r border-slate-800 bg-[#0b0f14] py-3">
          {toolItems.map((item) => {
            const Icon = item.icon;
            const active = activeTool === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => setActiveTool(item.id)}
                className={`grid h-10 w-10 place-items-center rounded-md border text-slate-200 ${
                  active ? 'border-cyan-400 bg-cyan-500/20 text-cyan-100' : 'border-slate-800 bg-slate-950 hover:bg-slate-900'
                }`}
                aria-label={item.label}
                title={item.label}
              >
                <Icon className="h-4 w-4" />
              </button>
            );
          })}
          <div className="my-1 h-px w-8 bg-slate-800" />
          <button
            type="button"
            onClick={toggleGrid}
            className={`grid h-10 w-10 place-items-center rounded-md border ${
              gridVisible ? 'border-emerald-500/70 bg-emerald-500/15 text-emerald-100' : 'border-slate-800 bg-slate-950 text-slate-400'
            }`}
            aria-label="Toggle grid"
            title="Toggle grid"
          >
            <Grid3X3 className="h-4 w-4" />
          </button>
          <button
            type="button"
            onClick={resetFixtureScene}
            className="grid h-10 w-10 place-items-center rounded-md border border-slate-800 bg-slate-950 text-slate-300 hover:bg-slate-900"
            aria-label="Reset scene"
            title="Reset scene"
          >
            <RotateCcw className="h-4 w-4" />
          </button>
        </nav>

        <div className="relative min-h-0 bg-black" data-testid="cad-viewport-region">
          <Suspense
            fallback={
              <div className="grid h-full place-items-center bg-[#07090c] text-sm text-slate-400">
                <div className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Loading local CAD viewport
                </div>
              </div>
            }
          >
            <CadViewportShell />
          </Suspense>
        </div>

        <aside className="min-h-0 overflow-y-auto border-l border-slate-800 bg-[#0b0f14] p-4 max-lg:hidden">
          <section className="rounded-md border border-slate-800 bg-slate-950/80 p-3">
            <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-100">
              <Move3D className="h-4 w-4 text-cyan-300" />
              Scene state
            </div>
            <div className="space-y-1 text-xs text-slate-400">
              <div>Tool: <span className="text-slate-100">{activeTool}</span></div>
              <div>Selected: <span className="text-slate-100">{selectedEntity?.label ?? 'none'}</span></div>
              <div>Render: <span className="text-slate-100">{performance.renderQuality} / DPR {performance.dprMax}</span></div>
            </div>
          </section>

          <section className="mt-3 rounded-md border border-slate-800 bg-slate-950/80 p-3">
            <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-100">
              <Box className="h-4 w-4 text-cyan-300" />
              Scene graph
            </div>
            <div className="space-y-2">
              {entities.map((entity) => (
                <div
                  key={entity.id}
                  className={`rounded-md border p-2 ${
                    selectedEntityId === entity.id ? 'border-cyan-500 bg-cyan-500/10' : 'border-slate-800 bg-slate-900/60'
                  }`}
                >
                  <div className="flex items-center justify-between gap-2">
                    <button
                      type="button"
                      onClick={() => selectEntity(entity.id)}
                      className="min-w-0 truncate text-left text-xs font-medium text-slate-100"
                    >
                      {entity.label}
                    </button>
                    <button
                      type="button"
                      onClick={() => toggleEntityVisibility(entity.id)}
                      className="grid h-7 w-7 shrink-0 place-items-center rounded border border-slate-700 text-slate-300 hover:bg-slate-800"
                      aria-label={`Toggle ${entity.label}`}
                    >
                      {entity.visible ? <Eye className="h-3.5 w-3.5" /> : <EyeOff className="h-3.5 w-3.5" />}
                    </button>
                  </div>
                  <div className="mt-1 text-[11px] text-slate-500">{entity.kind} / {entity.triangleBudget.toLocaleString()} tris</div>
                </div>
              ))}
            </div>
          </section>

          <section className="mt-3 rounded-md border border-slate-800 bg-slate-950/80 p-3">
            <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-100">
              <Braces className="h-4 w-4 text-cyan-300" />
              Local extensions
            </div>
            <div className="grid grid-cols-2 gap-2">
              {capabilityItems.map(([label, enabled]) => (
                <div key={label} className="rounded border border-slate-800 bg-slate-900 px-2 py-1.5 text-xs">
                  <div className={enabled ? 'text-emerald-300' : 'text-slate-500'}>{enabled ? 'ready' : 'offline'}</div>
                  <div className="text-slate-300">{label}</div>
                </div>
              ))}
            </div>
            {bootstrapError && <div className="mt-2 rounded border border-red-900 bg-red-950/40 p-2 text-xs text-red-200">{bootstrapError}</div>}
            <div className="mt-3 space-y-2">
              {(bootstrap?.extensionPoints ?? []).map((extension) => (
                <div key={extension.id} className="rounded border border-slate-800 bg-slate-900/70 p-2 text-xs">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-slate-100">{extension.label}</span>
                    <span className="text-slate-500">{extension.layer}</span>
                  </div>
                  <div className="mt-1 text-slate-500">{extension.notes}</div>
                </div>
              ))}
            </div>
          </section>
        </aside>
      </section>

      <footer className="flex h-9 shrink-0 items-center justify-between border-t border-slate-800 bg-[#0d1117] px-3 text-xs text-slate-400">
        <div className="flex items-center gap-2">
          <Bot className="h-3.5 w-3.5 text-cyan-300" />
          Offline CAD shell / React UI + Three viewport + Tauri orchestration
        </div>
        <div>Workflow: Import - Clean - Segment - Design - Validate - Export</div>
      </footer>
    </main>
  );
}

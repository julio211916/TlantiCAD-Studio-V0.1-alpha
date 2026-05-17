'use client';

import React, { lazy, Suspense, useCallback, useEffect, useMemo, useState } from 'react';
import { MotionConfig } from 'framer-motion';

import { AppQueryProvider } from '@/providers/AppQueryProvider';
import { ToastProvider } from '@/components/ui/use-toast';
import { WorkspaceErrorBoundary } from '@/components/workspaces/WorkspaceErrorBoundary';
import { TlantiWorkspacePreloader } from '@/components/workspaces/TlantiWorkspacePreloader';
import { WORKSPACE_TITLES, type AppWorkspaceId } from '@/components/workspaces/workspace.config';
import { buildWorkspaceUrl, getWorkspaceLaunchContext, type WorkspaceLaunchContext } from '@/lib/workspace-routing';
import { buildWorkspacePreloaderState, type WorkspacePreloaderPhase } from '@/lib/workspace-orchestrator';
import type { WorkspaceContextSyncRequest } from '@/lib/workspace-shell-contract';
import { I18nProvider } from '@/lib/i18n';
import { CommandPalette, commandRegistry } from '@/features/command-palette';
import { HotkeyHelpOverlay, hotkeyRegistry } from '@/features/hotkeys';
import type { Language, ThemeMode } from '@/types';
import { LocalRuntimeBridge } from './runtime/LocalRuntimeBridge';

const TlantiDbWorkspace = lazy(() =>
  import('@/components/workspaces/tlanti-db/TlantiDbWorkspace').then((module) => ({
    default: module.TlantiDbWorkspace,
  })),
);

const TlantiCadWorkspace = lazy(() =>
  import('@/components/workspaces/tlanti-cad/TlantiCadWorkspace').then((module) => ({
    default: module.TlantiCadWorkspace,
  })),
);

const BOOT_PRELOADER_MIN_MS = 320;
const WORKSPACE_TRANSITION_MS = 420;

export function App() {
  const initialLaunchContext = useMemo(() => getWorkspaceLaunchContext(), []);
  const [language, setLanguage] = useState<Language>('es');
  const [themeMode, setThemeMode] = useState<ThemeMode>('dark');
  const [workspaceContext, setWorkspaceContext] = useState<WorkspaceLaunchContext>(initialLaunchContext);
  const [activeWorkspace, setActiveWorkspace] = useState<AppWorkspaceId>(initialLaunchContext.workspace);
  const [bootReady, setBootReady] = useState(false);
  const [preloaderPhase, setPreloaderPhase] = useState<WorkspacePreloaderPhase>('boot');
  const [showPreloader, setShowPreloader] = useState(true);

  useEffect(() => {
    document.documentElement.classList.toggle('light', themeMode === 'light');
    document.documentElement.classList.toggle('dark', themeMode === 'dark');
    document.documentElement.style.setProperty('--bg-color', themeMode === 'light' ? '#f5f5f5' : '#000000');
    document.documentElement.style.setProperty('--text-color', themeMode === 'light' ? '#000000' : '#ffffff');
  }, [themeMode]);

  useEffect(() => {
    const startedAt = performance.now();
    let timeoutId: number | undefined;
    let cancelled = false;

    const completeBoot = () => {
      const remaining = Math.max(0, BOOT_PRELOADER_MIN_MS - (performance.now() - startedAt));
      timeoutId = window.setTimeout(() => {
        if (cancelled) return;
        setBootReady(true);
        setShowPreloader(false);
      }, remaining);
    };

    if (document.readyState === 'complete') {
      completeBoot();
    } else {
      window.addEventListener('load', completeBoot, { once: true });
    }

    return () => {
      cancelled = true;
      if (timeoutId !== undefined) window.clearTimeout(timeoutId);
      window.removeEventListener('load', completeBoot);
    };
  }, []);

  useEffect(() => {
    const syncFromUrl = () => {
      const nextContext = getWorkspaceLaunchContext();
      setWorkspaceContext(nextContext);
      setActiveWorkspace(nextContext.workspace);
    };

    window.addEventListener('popstate', syncFromUrl);
    return () => window.removeEventListener('popstate', syncFromUrl);
  }, []);

  const runWorkspaceTransition = useCallback((nextContext: WorkspaceLaunchContext, complete: () => void) => {
    setPreloaderPhase('transition');
    setShowPreloader(true);
    window.setTimeout(() => {
      complete();
      setShowPreloader(false);
      setPreloaderPhase('lazy-load');
    }, WORKSPACE_TRANSITION_MS);
  }, []);

  const handleEnterCad = useCallback((options?: { caseId?: string; moduleId?: string }) => {
    const nextContext: WorkspaceLaunchContext = {
      ...workspaceContext,
      workspace: 'tlanticad',
      caseId: options?.caseId ?? workspaceContext.caseId,
      module: options?.moduleId,
      title: undefined,
    };

    window.history.replaceState({}, '', buildWorkspaceUrl(nextContext));
    runWorkspaceTransition(nextContext, () => {
      setWorkspaceContext(nextContext);
      setActiveWorkspace('tlanticad');
    });
  }, [runWorkspaceTransition, workspaceContext]);

  const handleBackToDb = useCallback(() => {
    const nextContext: WorkspaceLaunchContext = {
      ...workspaceContext,
      workspace: 'tlantidb',
      module: undefined,
      title: undefined,
    };

    window.history.replaceState({}, '', buildWorkspaceUrl(nextContext));
    runWorkspaceTransition(nextContext, () => {
      setWorkspaceContext(nextContext);
      setActiveWorkspace('tlantidb');
    });
  }, [runWorkspaceTransition, workspaceContext]);

  const handleSyncWorkspaceContext = useCallback((request: WorkspaceContextSyncRequest) => {
    const nextContext: WorkspaceLaunchContext = {
      ...workspaceContext,
      workspace: 'tlantidb',
      caseId: request.caseId ?? workspaceContext.caseId,
      module: undefined,
      title: undefined,
    };

    window.history.replaceState({}, '', buildWorkspaceUrl(nextContext));
    setPreloaderPhase('sync');
    setWorkspaceContext(nextContext);
    setActiveWorkspace('tlantidb');
    window.setTimeout(() => setPreloaderPhase('lazy-load'), 160);
  }, [workspaceContext]);

  useEffect(() => {
    const disposers = [
      hotkeyRegistry.register({
        chord: 'ctrl+1',
        label: 'Abrir TlantiDB',
        context: 'global',
        run: () => handleBackToDb(),
      }),
      hotkeyRegistry.register({
        chord: 'ctrl+2',
        label: 'Abrir TlantiCAD',
        context: 'global',
        run: () => handleEnterCad(),
      }),
      hotkeyRegistry.register({
        chord: 'ctrl+,',
        label: 'Cambiar tema',
        context: 'global',
        run: () => setThemeMode((mode) => (mode === 'light' ? 'dark' : 'light')),
      }),
    ];

    return () => {
      for (const dispose of disposers) dispose();
    };
  }, [handleBackToDb, handleEnterCad]);

  useEffect(() => commandRegistry.registerAll([
    {
      id: 'app.workspace.tlantidb',
      label: 'Abrir TlantiDB',
      kind: 'navigation',
      keywords: ['workspace', 'patients', 'cases', 'launcher', 'db'],
      run: () => handleBackToDb(),
    },
    {
      id: 'app.workspace.tlanticad',
      label: 'Abrir TlantiCAD',
      kind: 'navigation',
      keywords: ['cad', 'design', 'wizard'],
      run: () => handleEnterCad(),
    },
    {
      id: 'app.theme.toggle',
      label: themeMode === 'light' ? 'Modo oscuro' : 'Modo claro',
      kind: 'toggle',
      keywords: ['theme', 'dark', 'light'],
      run: () => setThemeMode((mode) => (mode === 'light' ? 'dark' : 'light')),
    },
  ]), [handleBackToDb, handleEnterCad, themeMode]);

  const preloaderState = buildWorkspacePreloaderState(workspaceContext, preloaderPhase);
  const suspensePreloaderState = buildWorkspacePreloaderState(
    { ...workspaceContext, workspace: activeWorkspace },
    'lazy-load',
  );

  return (
    <AppQueryProvider>
      <I18nProvider>
        <MotionConfig reducedMotion="user">
          <ToastProvider>
            <CommandPalette />
            <HotkeyHelpOverlay />
            <LocalRuntimeBridge />
            <TlantiWorkspacePreloader
              visible={showPreloader}
              title={WORKSPACE_TITLES.app}
              subtitle={preloaderState.subtitle}
              themeMode={themeMode}
              phase={preloaderState.phase}
              workspace={preloaderState.workspace}
              caseId={preloaderState.caseId}
              moduleId={preloaderState.moduleId}
            />

            {bootReady && (
              <Suspense
                fallback={(
                  <TlantiWorkspacePreloader
                    visible
                    title={WORKSPACE_TITLES.app}
                    subtitle={suspensePreloaderState.subtitle}
                    themeMode={themeMode}
                    phase={suspensePreloaderState.phase}
                    workspace={suspensePreloaderState.workspace}
                    caseId={suspensePreloaderState.caseId}
                    moduleId={suspensePreloaderState.moduleId}
                  />
                )}
              >
                <WorkspaceErrorBoundary activeWorkspace={activeWorkspace} themeMode={themeMode} onBackToDb={handleBackToDb}>
                  {activeWorkspace === 'tlantidb' ? (
                    <TlantiDbWorkspace
                      onEnter={handleEnterCad}
                      onSyncWorkspaceContext={handleSyncWorkspaceContext}
                      language={language}
                      setLanguage={setLanguage}
                      themeMode={themeMode}
                      setThemeMode={setThemeMode}
                      caseId={workspaceContext.caseId}
                      moduleId={workspaceContext.module}
                    />
                  ) : (
                    <TlantiCadWorkspace
                      language={language}
                      setLanguage={setLanguage}
                      themeMode={themeMode}
                      setThemeMode={setThemeMode}
                      caseId={workspaceContext.caseId}
                      moduleId={workspaceContext.module}
                      onBackToDb={handleBackToDb}
                    />
                  )}
                </WorkspaceErrorBoundary>
              </Suspense>
            )}
          </ToastProvider>
        </MotionConfig>
      </I18nProvider>
    </AppQueryProvider>
  );
}

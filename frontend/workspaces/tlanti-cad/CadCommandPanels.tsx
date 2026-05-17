import React, { lazy, Suspense } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

import type { Action } from '@/components/ui/action-search-bar';
import type { ThemeMode } from '@/types';
import type { TlantiDbPreferences } from '@/stores/tlantidb-case-store';
import type {
  TlantiCadModuleRoadmapDefinition,
  TlantiCadModuleWorkflowPhase,
  TlantiCadProductModuleDefinition,
} from '@/core';

const ShortcutsPanel = lazy(() => import('@/components/ShortcutsPanel').then((module) => ({ default: module.ShortcutsPanel })));
const ActionSearchBar = lazy(() => import('@/components/ui/action-search-bar').then((module) => ({ default: module.ActionSearchBar })));

export interface CadCommandPanelsProps {
  shortcutsOpen: boolean;
  onCloseShortcuts: () => void;
  themeMode: ThemeMode;
  controlScheme: TlantiDbPreferences['controlScheme'];
  onControlSchemeChange: (value: TlantiDbPreferences['controlScheme']) => void;
  commandPaletteOpen: boolean;
  onCloseCommandPalette: () => void;
  commandPaletteActions: Action[];
  activeProductModule: TlantiCadProductModuleDefinition;
  activeRoadmap: TlantiCadModuleRoadmapDefinition;
  activeWorkflowPhase: TlantiCadModuleWorkflowPhase;
}

export function CadCommandPanels({
  shortcutsOpen,
  onCloseShortcuts,
  themeMode,
  controlScheme,
  onControlSchemeChange,
  commandPaletteOpen,
  onCloseCommandPalette,
  commandPaletteActions,
  activeProductModule,
  activeRoadmap,
  activeWorkflowPhase,
}: CadCommandPanelsProps) {
  const activeJobs = activeWorkflowPhase.jobs.length > 0
    ? activeWorkflowPhase.jobs.join(' / ')
    : 'ui-only';

  return (
    <>
      <Suspense fallback={null}>
        <ShortcutsPanel
          isOpen={shortcutsOpen}
          onClose={onCloseShortcuts}
          themeMode={themeMode}
          controlScheme={controlScheme}
          onControlSchemeChange={onControlSchemeChange}
        />
      </Suspense>

      <AnimatePresence>
        {commandPaletteOpen && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-[100] flex items-start justify-center bg-black/50 pt-[20vh] backdrop-blur-sm"
            onClick={onCloseCommandPalette}
          >
            <div onClick={(event) => event.stopPropagation()} className="w-full max-w-2xl">
              <div className="mb-3 rounded-md border border-white/10 bg-[#111318]/95 px-4 py-3 text-white shadow-2xl backdrop-blur-xl">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <p className="text-[11px] uppercase tracking-[0.18em] text-white/45">Active CAD module</p>
                    <h2 className="mt-1 text-base font-semibold">{activeProductModule.label}</h2>
                  </div>
                  <span className="rounded-md border border-white/10 px-2 py-1 text-[11px] uppercase tracking-[0.14em] text-white/60">
                    {activeProductModule.shortLabel}
                  </span>
                </div>
                <div className="mt-3 grid gap-2 text-xs text-white/68 sm:grid-cols-[1.2fr_0.8fr]">
                  <p>
                    <span className="text-white/90">{activeWorkflowPhase.label}</span>
                    {' · '}
                    {activeWorkflowPhase.userGoal}
                  </p>
                  <p className="sm:text-right">
                    Owner {activeWorkflowPhase.logicOwner} · Jobs {activeJobs}
                  </p>
                </div>
                <p className="mt-2 text-[11px] text-white/45">
                  {activeRoadmap.differentiators[0]}
                </p>
              </div>
              <Suspense fallback={null}>
                <ActionSearchBar actions={commandPaletteActions} />
              </Suspense>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

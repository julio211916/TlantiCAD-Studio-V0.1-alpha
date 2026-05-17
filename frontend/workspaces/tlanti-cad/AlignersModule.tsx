import React, { useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import { Braces, ChevronRight, ScanSearch, Sparkles, Workflow, X } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import type { BrowserCapabilityState } from '@/lib/browser-capability-state';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { useViewportProfile } from '../hooks/useViewportProfile';
import type { ThemeMode } from '../types';
import { commandRegistry } from '@/features/command-palette';

interface AlignersModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  activeCase?: TlantiCase | null;
  capabilities: BrowserCapabilityState;
}

const ALIGNER_STEPS = [
  'Tooth segmentation',
  'Stage timeline',
  'Attachments',
  'IPR',
  'Collision review',
  'Export',
];

export function AlignersModule({ themeMode, onClose, activeCase, capabilities }: AlignersModuleProps) {
  const viewport = useViewportProfile();
  const selectedTeeth = useMemo(
    () => Object.values(activeCase?.toothMap ?? {}).filter((tooth) => tooth.selected).length,
    [activeCase?.toothMap],
  );

  // Aligners is currently a read-only stage shell — only the close action has a
  // real runnable callback, so contextual stage navigation stays out of the
  // palette until the segmentation/IPR backend lands.
  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.aligners.close',
        label: 'Cerrar módulo Aligners',
        kind: 'navigation',
        keywords: ['aligners', 'close', 'cerrar'],
        run: onClose,
      },
    ]);
  }, [onClose]);

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className={cn(
        'absolute z-40 flex flex-col overflow-hidden rounded-lg border shadow-2xl',
        viewport.isCompact ? 'inset-3 top-[8rem]' : 'left-16 top-24 bottom-24 w-[22rem]',
        'bg-surface border-border text-text-primary',
        themeMode === 'light' && 'bg-surface',
      )}
    >
      <div className="flex items-center justify-between border-b border-border bg-surface-raised p-4">
        <div className="flex items-center gap-2">
          <Braces size={18} className="text-text-display" />
          <h3 className="font-display font-semibold tracking-tight text-text-primary">Aligners</h3>
        </div>
        <button onClick={onClose} aria-label="Close Aligners module" className="text-text-secondary transition-colors hover:text-text-primary">
          <X size={16} aria-hidden />
        </button>
      </div>

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto bg-surface p-4">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase text-text-secondary">Case</p>
              <h4 className="mt-1 text-base font-semibold text-text-display">
                {activeCase ? `${activeCase.caseNumber} · ${activeCase.name}` : 'No active case'}
              </h4>
              <p className="mt-1 text-xs text-text-secondary">
                {capabilities.runtime === 'desktop'
                  ? 'Stage planning can use local assets and native storage.'
                  : 'Browser fallback keeps the workflow visible and blocks only native-only operations.'}
              </p>
            </div>
            <Badge variant="outline" className="border-border bg-card text-text-primary">
              {selectedTeeth} teeth
            </Badge>
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex items-center gap-2">
            <Workflow size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Workflow</h4>
          </div>
          <div className="mt-3 space-y-2">
            {ALIGNER_STEPS.map((step, index) => (
              <div key={step} className="flex items-center justify-between rounded-xl border border-border bg-card px-3 py-2">
                <div className="flex items-center gap-2">
                  <span className="inline-flex size-5 items-center justify-center rounded-full bg-surface text-[10px] font-semibold text-text-secondary">
                    {index + 1}
                  </span>
                  <span className="text-sm text-text-primary">{step}</span>
                </div>
                <ChevronRight size={14} className="text-text-secondary" />
              </div>
            ))}
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
          <div className="flex items-center gap-2">
            <ScanSearch size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Fallback policy</h4>
          </div>
          <p className="mt-2">
            This module no longer depends on the odontogram preloader to appear. If segmentation data is missing, it stays honest and keeps the operator inside a recoverable stage shell.
          </p>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
          <div className="flex items-center gap-2">
            <Sparkles size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Next tool boundary</h4>
          </div>
          <p className="mt-2">
            The next tranche should replace these stage cards with real segmentation, attachment and IPR surfaces backed by the local AI/runtime pipeline.
          </p>
        </section>
      </div>
    </motion.div>
  );
}

import React, { useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import { BrainCircuit, FolderOpen, Layers3, ScanSearch, Workflow, X } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import type { BrowserCapabilityState } from '@/lib/browser-capability-state';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { useViewportProfile } from '../hooks/useViewportProfile';
import type { ThemeMode } from '../types';
import { commandRegistry } from '@/features/command-palette';

interface DicomViewerModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  onImportDicom?: () => void;
  activeCase?: TlantiCase | null;
  capabilities: BrowserCapabilityState;
}

export function DicomViewerModule({
  themeMode,
  onClose,
  onImportDicom,
  activeCase,
  capabilities,
}: DicomViewerModuleProps) {
  const viewport = useViewportProfile();
  const dicomAssets = useMemo(
    () =>
      (activeCase?.assets ?? []).filter((asset) =>
        ['dicom-study', 'dicom-series', 'dicom-zip', 'dicom-slice'].includes(asset.role),
      ),
    [activeCase?.assets],
  );

  useEffect(() => {
    const actions = [
      {
        id: 'cad.module.dicom.close',
        label: 'Cerrar módulo DICOM Viewer',
        kind: 'navigation' as const,
        keywords: ['dicom', 'close', 'cerrar'],
        run: onClose,
      },
    ];
    if (onImportDicom) {
      actions.push({
        id: 'cad.module.dicom.import',
        label: 'DICOM: importar estudio',
        kind: 'navigation' as const,
        keywords: ['dicom', 'import', 'study', 'importar'],
        run: onImportDicom,
      });
    }
    return commandRegistry.registerAll(actions);
  }, [onClose, onImportDicom]);

  const sections = [
    {
      title: 'Study Browser',
      description: 'Series, tags and local case-linked studies.',
      ready: dicomAssets.length > 0,
      icon: FolderOpen,
    },
    {
      title: 'MPR / VOI',
      description: 'Axial, sagittal, coronal and volume presets.',
      ready: dicomAssets.length > 0,
      icon: Layers3,
    },
    {
      title: 'Metadata',
      description: 'Patient, study and acquisition context.',
      ready: true,
      icon: ScanSearch,
    },
    {
      title: 'Segmentation Handoff',
      description: 'Bridge into implant, guide and ceph workflows.',
      ready: dicomAssets.length > 0,
      icon: Workflow,
    },
  ];

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
          <ScanSearch size={18} className="text-text-display" />
          <h3 className="font-display font-semibold tracking-tight text-text-primary">DICOM Viewer</h3>
        </div>
        <button onClick={onClose} aria-label="Close DICOM Viewer" className="text-text-secondary transition-colors hover:text-text-primary">
          <X size={16} aria-hidden />
        </button>
      </div>

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto bg-surface p-4">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase text-text-secondary">Runtime</p>
              <h4 className="mt-1 text-base font-semibold text-text-display">
                {capabilities.runtime === 'desktop' ? 'Desktop runtime' : 'Browser fallback'}
              </h4>
              <p className="mt-1 text-xs text-text-secondary">
                {capabilities.runtime === 'desktop'
                  ? 'Local filesystem and native viewer handoff available.'
                  : 'Usable for review and intake, with limited native filesystem integration.'}
              </p>
            </div>
            <Badge variant="outline" className="border-border bg-card text-text-primary">
              {dicomAssets.length} studies
            </Badge>
          </div>
        </section>

        <section className="grid gap-2">
          {sections.map((section) => {
            const Icon = section.icon;
            return (
              <div key={section.title} className="rounded-2xl border border-border bg-surface-raised p-4">
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-start gap-3">
                    <Icon size={16} className="mt-0.5 shrink-0 text-text-display" />
                    <div>
                      <p className="text-sm font-semibold text-text-display">{section.title}</p>
                      <p className="mt-1 text-xs text-text-secondary">{section.description}</p>
                    </div>
                  </div>
                  <Badge variant="outline" className="border-border bg-card text-text-secondary">
                    {section.ready ? 'Ready' : 'Pending data'}
                  </Badge>
                </div>
              </div>
            );
          })}
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
          <div className="flex items-center gap-2">
            <BrainCircuit size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Clinical handoff</h4>
          </div>
          <p className="mt-2">
            This module owns DICOM intake and review. Implant, Surgical Guide and Ceph should reuse this study context instead of opening a generic CAD shell.
          </p>
        </section>

        <button
          type="button"
          onClick={onImportDicom}
          className="mt-auto rounded bg-text-display px-4 py-3 text-xs font-bold uppercase tracking-widest text-black transition-colors hover:bg-white"
        >
          Import DICOM study
        </button>
      </div>
    </motion.div>
  );
}

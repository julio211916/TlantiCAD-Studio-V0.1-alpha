import React, { useEffect, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { AlertCircle, Check, Download, Eye, Layers, Settings, Target } from 'lucide-react';
import clsx from 'clsx';
import { ThemeMode } from '../types';
import { useToast } from './ui/use-toast';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { Badge } from '@/components/ui/badge';
import { downloadJsonBrief, getClinicalAssetReadiness, getImplantTargets } from '@/lib/clinical-module-briefs';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { commandRegistry } from '@/features/command-palette';

interface SurgicalGuideModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  activeCase?: TlantiCase | null;
}

export const SurgicalGuideModule: React.FC<SurgicalGuideModuleProps> = ({ themeMode, onClose, activeCase }) => {
  const { toast } = useToast();
  const viewport = useViewportProfile();
  const [thickness, setThickness] = useState(2.0);
  const [offset, setOffset] = useState(0.1);
  const [sleeveDiameter, setSleeveDiameter] = useState(5.0);
  const [showDicom, setShowDicom] = useState(true);
  const implantTargets = useMemo(() => getImplantTargets(activeCase), [activeCase]);
  const assetReadiness = useMemo(() => getClinicalAssetReadiness(activeCase?.assets ?? []), [activeCase?.assets]);
  const canGenerateGuide = Boolean(activeCase && implantTargets.length && assetReadiness.hasDicomStudy && (assetReadiness.hasPrepScan || assetReadiness.hasGingivaScan));

  const handleGenerateGuide = () => {
    if (!canGenerateGuide) {
      toast('Guide generation needs an active implant case plus DICOM and scan records.', 'error');
      return;
    }

    toast('Guide parameters validated for the active clinical case.', 'success');
  };

  const handleExportGuide = () => {
    if (!activeCase) {
      toast('Open a clinical case before exporting guide data.', 'error');
      return;
    }

    if (!canGenerateGuide) {
      toast('Complete the DICOM / scan prerequisites before exporting guide data.', 'error');
      return;
    }

    const payload = {
      kind: 'surgical-guide-brief',
      createdAt: new Date().toISOString(),
      caseId: activeCase.id,
      caseNumber: activeCase.caseNumber,
      caseName: activeCase.name,
      targetTeeth: implantTargets.map((target) => target.toothNumber),
      guideParameters: {
        thickness,
        offset,
        sleeveDiameter,
        showDicom,
      },
      readiness: {
        dicomStudy: assetReadiness.hasDicomStudy,
        prepScan: assetReadiness.hasPrepScan,
        gingivaScan: assetReadiness.hasGingivaScan,
        manufacturingReport: assetReadiness.hasManufacturingReport,
      },
      assets: activeCase.assets ?? [],
    };

    downloadJsonBrief(`${activeCase.caseNumber}-guide-brief.json`, payload);
    toast('Guide brief exported for manufacturing review.', 'success');
  };

  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.guide.close',
        label: 'Cerrar módulo Surgical Guide',
        kind: 'navigation',
        keywords: ['guide', 'close', 'cerrar'],
        run: onClose,
      },
      {
        id: 'cad.module.guide.validate',
        label: 'Surgical Guide: validar setup',
        kind: 'tool',
        keywords: ['guide', 'validate', 'setup'],
        available: () => canGenerateGuide,
        run: handleGenerateGuide,
      },
      {
        id: 'cad.module.guide.export',
        label: 'Surgical Guide: exportar brief',
        kind: 'tool',
        keywords: ['guide', 'export', 'brief'],
        available: () => canGenerateGuide,
        run: handleExportGuide,
      },
    ]);
  }, [onClose, canGenerateGuide, handleGenerateGuide, handleExportGuide]);

  return (
    <motion.div 
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className={clsx(
        "absolute z-40 flex flex-col overflow-hidden rounded-lg border shadow-2xl",
        viewport.isCompact ? 'inset-3 top-[8rem]' : 'left-16 top-24 bottom-24 w-80',
        themeMode === 'dark' ? "bg-surface border-border text-text-primary" : "bg-surface border-border text-text-primary"
      )}
    >
      <div className="p-4 border-b border-border flex items-center justify-between bg-surface-raised">
        <div className="flex items-center gap-2">
          <Target size={18} className="text-text-display" />
          <h3 className="font-display tracking-tight font-semibold text-text-primary">Surgical Guide</h3>
        </div>
        <button onClick={onClose} title="Close surgical guide" aria-label="Close surgical guide" className="text-text-secondary hover:text-text-primary transition-colors">✕</button>
      </div>

      <div className="p-4 flex-1 overflow-y-auto flex flex-col gap-6 bg-surface">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Guide context</p>
              <h4 className="mt-1 text-balance text-lg font-semibold text-text-display">
                {activeCase ? `${activeCase.caseNumber} · ${activeCase.name}` : 'No active case in CAD'}
              </h4>
              <p className="mt-1 text-sm text-text-secondary">
                {activeCase
                  ? 'Guide generation now uses the active case, implant teeth and attached records to export a manufacturing brief.'
                  : 'Open this module from the TlantiCAD Workspace or CAD with an active case to generate a guide brief.'}
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline" className="border-border bg-card text-text-primary">{implantTargets.length} implant teeth</Badge>
              <Badge variant="outline" className="border-border bg-card text-text-secondary">{assetReadiness.totalAssets} assets</Badge>
            </div>
          </div>

          <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
            <div className="rounded-2xl border border-border bg-card px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">DICOM</p>
              <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{assetReadiness.counts['dicom-study'] ?? 0}</p>
            </div>
            <div className="rounded-2xl border border-border bg-card px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Prep scans</p>
              <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{assetReadiness.counts['prep-scan'] ?? 0}</p>
            </div>
            <div className="rounded-2xl border border-border bg-card px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Gingiva</p>
              <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{assetReadiness.counts['gingiva-scan'] ?? 0}</p>
            </div>
            <div className="rounded-2xl border border-border bg-card px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Reports</p>
              <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{assetReadiness.counts['manufacturing-report'] ?? 0}</p>
            </div>
          </div>
        </section>
        
        <div className="flex items-center justify-between p-3 rounded bg-surface-raised border border-border-visible">
          <span className="text-[11px] font-mono uppercase tracking-widest text-text-primary flex items-center gap-2"><Layers size={14} /> DICOM Overlay</span>
          <button 
            onClick={() => setShowDicom(!showDicom)}
            title={showDicom ? 'Hide DICOM overlay' : 'Show DICOM overlay'}
            aria-label={showDicom ? 'Hide DICOM overlay' : 'Show DICOM overlay'}
            className={clsx("p-1 rounded transition-colors", showDicom ? "text-text-display" : "text-text-secondary hover:text-text-primary")}
          >
            <Eye size={18} />
          </button>
        </div>

        {implantTargets.length ? (
          <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
            <div className="mb-3 flex items-center gap-2">
              <Target size={14} className="text-text-display" />
              <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Guide targets</h4>
            </div>
            <div className="flex flex-wrap gap-2">
              {implantTargets.map((target) => (
                <Badge key={target.toothNumber} variant="outline" className="border-border bg-card text-text-primary">
                  Tooth {target.toothNumber} · {target.implantMode}
                </Badge>
              ))}
            </div>
          </section>
        ) : (
          <section className="rounded-2xl border border-dashed border-border bg-surface-raised p-4 text-sm text-text-secondary">
            No implant-marked teeth found in the active case. Define implant targets in the TlantiCAD Workspace first so the guide brief has a real clinical scope.
          </section>
        )}

        <div className="flex flex-col gap-4 border-t border-border pt-4">
          <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary flex items-center gap-2"><Settings size={14} /> Guide Parameters</h4>
          
          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
              <span>Guide Thickness</span>
              <span>{thickness.toFixed(1)} mm</span>
            </div>
            <input 
              title="Guide thickness"
              aria-label="Guide thickness"
              type="range" 
              min="1.0" max="4.0" step="0.1"
              value={thickness}
              onChange={(e) => setThickness(parseFloat(e.target.value))}
              className="w-full accent-text-display"
            />
          </div>

          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
              <span>Tooth Offset (Gap)</span>
              <span>{offset.toFixed(2)} mm</span>
            </div>
            <input 
              title="Guide tooth offset"
              aria-label="Guide tooth offset"
              type="range" 
              min="0.0" max="0.5" step="0.01"
              value={offset}
              onChange={(e) => setOffset(parseFloat(e.target.value))}
              className="w-full accent-text-display"
            />
          </div>

          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
              <span>Sleeve Hole Diameter</span>
              <span>{sleeveDiameter.toFixed(1)} mm</span>
            </div>
            <input 
              title="Guide sleeve diameter"
              aria-label="Guide sleeve diameter"
              type="range" 
              min="3.0" max="8.0" step="0.1"
              value={sleeveDiameter}
              onChange={(e) => setSleeveDiameter(parseFloat(e.target.value))}
              className="w-full accent-text-display"
            />
          </div>
        </div>

        <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
          <div className="mb-3 flex items-center gap-2">
            <AlertCircle size={14} className="text-text-secondary" />
            <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Guide gates</h4>
          </div>
          <div className="grid gap-2">
            <p>DICOM / CBCT study: <span className="text-text-primary">{assetReadiness.hasDicomStudy ? 'ready' : 'missing'}</span></p>
            <p>Prep or gingiva scan: <span className="text-text-primary">{assetReadiness.hasPrepScan || assetReadiness.hasGingivaScan ? 'ready' : 'missing'}</span></p>
            <p>Implant targets: <span className="text-text-primary">{implantTargets.length ? `${implantTargets.length} defined` : 'missing'}</span></p>
            <p>Manufacturing report: <span className="text-text-primary">{assetReadiness.hasManufacturingReport ? 'attached' : 'optional in this phase'}</span></p>
          </div>
        </section>

        <div className="mt-auto flex flex-col gap-2">
          <button 
            onClick={handleGenerateGuide}
            disabled={!canGenerateGuide}
            className="w-full py-3 bg-surface-raised border border-border-visible text-text-primary font-mono text-xs uppercase tracking-widest font-bold rounded hover:bg-surface transition-colors flex items-center justify-center gap-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Check size={16} /> Validate guide setup
          </button>
          <button 
            onClick={handleExportGuide}
            disabled={!canGenerateGuide}
            className="w-full py-3 bg-text-display text-black font-mono text-xs uppercase tracking-widest font-bold rounded hover:bg-white transition-colors flex items-center justify-center gap-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Download size={16} /> Export guide brief
          </button>
        </div>
      </div>
    </motion.div>
  );
};

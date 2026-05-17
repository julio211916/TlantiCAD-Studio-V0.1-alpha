import React, { useEffect, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { AlertCircle, Check, CircleDot, Files, ScanSearch, Syringe } from 'lucide-react';
import clsx from 'clsx';
import { ThemeMode } from '../types';
import { useToast } from './ui/use-toast';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { Badge } from '@/components/ui/badge';
import { downloadJsonBrief, getClinicalAssetReadiness, getImplantTargets } from '@/lib/clinical-module-briefs';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { commandRegistry } from '@/features/command-palette';

interface ImplantModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  activeCase?: TlantiCase | null;
}

const IMPLANT_TYPES = [
  { value: 'bone_level', label: 'Bone level tapered' },
  { value: 'tissue_level', label: 'Tissue level' },
  { value: 'zygomatic', label: 'Zygomatic' },
  { value: 'mini', label: 'Mini implant' },
] as const;

const RESTORATIVE_WORKFLOWS = [
  { value: 'implant-planning', label: 'Implant planning' },
  { value: 'post-and-core', label: 'Post and Core' },
] as const;

const RESTORATION_MATERIALS = ['Zirconia', 'Composite', 'Titanium', 'NP Metal', 'Lithium Disilicate'] as const;
const POST_CORE_MATERIALS = ['Titanium', 'NP Metal', 'Composite', 'Fiber post hybrid'] as const;
const POST_CORE_STEPS_WITH_SCAN_BODY = [
  'Detect marker position',
  'Detect inner margin line',
  'Place model tooth',
  'Define insertion direction',
  'Adjust post bottom',
  'Design core',
] as const;
const POST_CORE_STEPS_NO_SCAN_BODY = [
  'Detect inner margin line',
  'Place model tooth',
  'Define insertion direction',
  'Adjust post bottom',
  'Design core',
] as const;

export const ImplantModule: React.FC<ImplantModuleProps> = ({ themeMode, onClose, activeCase }) => {
  const { toast } = useToast();
  const viewport = useViewportProfile();
  const [workflowType, setWorkflowType] = useState<(typeof RESTORATIVE_WORKFLOWS)[number]['value']>('implant-planning');
  const [implantType, setImplantType] = useState('bone_level');
  const [diameter, setDiameter] = useState(4.1);
  const [length, setLength] = useState(10);
  const [angulation, setAngulation] = useState(0);
  const [depth, setDepth] = useState(0);
  const [restorationMaterial, setRestorationMaterial] = useState<(typeof RESTORATION_MATERIALS)[number]>('Zirconia');
  const [postCoreMaterial, setPostCoreMaterial] = useState<(typeof POST_CORE_MATERIALS)[number]>('Titanium');
  const [useScanBody, setUseScanBody] = useState(true);
  const [postBottom, setPostBottom] = useState(0.6);
  const [coreMinAngle, setCoreMinAngle] = useState(6);
  const [coreSpacing, setCoreSpacing] = useState(0.08);
  const [autoAdaptOcclusal, setAutoAdaptOcclusal] = useState(true);
  const implantTargets = useMemo(() => getImplantTargets(activeCase), [activeCase]);
  const assetReadiness = useMemo(() => getClinicalAssetReadiness(activeCase?.assets ?? []), [activeCase?.assets]);
  const [selectedTooth, setSelectedTooth] = useState('');

  useEffect(() => {
    if (!implantTargets.length) {
      setSelectedTooth('');
      return;
    }

    setSelectedTooth((current) => current && implantTargets.some((target) => target.toothNumber === current)
      ? current
      : implantTargets[0]?.toothNumber ?? '');
  }, [implantTargets]);

  const selectedTarget = implantTargets.find((target) => target.toothNumber === selectedTooth) ?? implantTargets[0] ?? null;
  const requiresSoftTissue = Boolean(selectedTarget?.needsExtraGingivaScan);
  const postCoreSteps = useMemo(() => (useScanBody ? POST_CORE_STEPS_WITH_SCAN_BODY : POST_CORE_STEPS_NO_SCAN_BODY), [useScanBody]);
  const hasPlanningGeometry = assetReadiness.hasPrepScan || assetReadiness.hasGingivaScan;
  const canGeneratePlanningBrief = Boolean(
    activeCase
    && selectedTarget
    && (workflowType === 'post-and-core'
      ? hasPlanningGeometry
      : assetReadiness.hasDicomStudy && hasPlanningGeometry),
  );

  const handleGeneratePlanningBrief = () => {
    if (!activeCase || !selectedTarget) {
      toast('Open a clinical case with implant or post/core teeth before generating a planning brief.', 'error');
      return;
    }

    if (workflowType !== 'post-and-core' && !assetReadiness.hasDicomStudy) {
      toast('A DICOM / CBCT study is required before implant planning can be exported.', 'error');
      return;
    }

    if (!hasPlanningGeometry) {
      toast('Attach a prep or gingiva scan before exporting implant or post/core planning data.', 'error');
      return;
    }

    if (workflowType === 'post-and-core' && useScanBody && !assetReadiness.hasGingivaScan) {
      toast('For scan-body guided Post and Core, attach the extra isolated scan-body scan before exporting the brief.', 'error');
      return;
    }

    const payload = {
      kind: workflowType === 'post-and-core' ? 'post-core-planning-brief' : 'implant-planning-brief',
      createdAt: new Date().toISOString(),
      caseId: activeCase.id,
      caseNumber: activeCase.caseNumber,
      caseName: activeCase.name,
      tooth: selectedTarget.toothNumber,
      implantMode: selectedTarget.implantMode,
      restorationLabel: selectedTarget.restorationLabel,
      workflowType,
      implantType,
      dimensions: {
        diameter,
        length,
      },
      placement: {
        angulation,
        depth,
      },
      postAndCore: workflowType === 'post-and-core' ? {
        workType: 'post-and-core',
        restorationMaterial,
        postCoreMaterial,
        implantBasedMode: 'post-and-core',
        useScanBody,
        wizardSteps: postCoreSteps,
        postBottom,
        coreMinAngle,
        coreSpacing,
        autoAdaptOcclusal,
      } : null,
      readiness: {
        dicomStudy: assetReadiness.hasDicomStudy,
        prepScan: assetReadiness.hasPrepScan,
        gingivaScan: assetReadiness.hasGingivaScan,
        opposingRecord: assetReadiness.hasOpposingRecord,
        labPrescription: assetReadiness.hasLabPrescription,
        requiresSoftTissue,
      },
      assets: activeCase.assets ?? [],
    };

    downloadJsonBrief(`${activeCase.caseNumber}-${workflowType === 'post-and-core' ? 'post-core' : 'implant'}-${selectedTarget.toothNumber}.json`, payload);
    toast(`${workflowType === 'post-and-core' ? 'Post and Core' : 'Implant'} planning brief exported for tooth ${selectedTarget.toothNumber}.`, 'success');
  };

  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.implant.close',
        label: 'Cerrar módulo Implant',
        kind: 'navigation',
        keywords: ['implant', 'close', 'cerrar'],
        run: onClose,
      },
      {
        id: 'cad.module.implant.export-brief',
        label: 'Implant: exportar planning brief',
        kind: 'tool',
        keywords: ['implant', 'brief', 'export', 'planning'],
        available: () => canGeneratePlanningBrief,
        run: handleGeneratePlanningBrief,
      },
    ]);
  }, [onClose, canGeneratePlanningBrief, handleGeneratePlanningBrief]);

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
          <Syringe size={18} className="text-text-display" />
          <h3 className="font-display tracking-tight font-semibold text-text-primary">Implant Planning</h3>
        </div>
        <button onClick={onClose} title="Close implant planning" aria-label="Close implant planning" className="text-text-secondary hover:text-text-primary transition-colors">✕</button>
      </div>

      <div className="p-4 flex-1 overflow-y-auto flex flex-col gap-6 bg-surface">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Case context</p>
              <h4 className="mt-1 text-balance text-lg font-semibold text-text-display">
                {activeCase ? `${activeCase.caseNumber} · ${activeCase.name}` : 'No active case in CAD'}
              </h4>
              <p className="mt-1 text-sm text-text-secondary">
                {activeCase
                  ? 'This panel now reads the active TlantiCAD Workspace case, target teeth and available clinical records before generating a planning brief.'
                  : 'Open this module from a TlantiCAD Workspace case to prefill implant targets, assets and workflow requirements.'}
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
              <p className="text-[11px] uppercase text-text-secondary">Opposing</p>
              <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{(assetReadiness.counts['antagonist-scan'] ?? 0) + (assetReadiness.counts['bite-registration'] ?? 0)}</p>
            </div>
          </div>
        </section>

        {implantTargets.length ? (
          <section className="rounded-2xl border border-border bg-surface-raised p-4">
            <div className="mb-3 flex items-center gap-2">
              <CircleDot size={14} className="text-text-display" />
              <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Target tooth</h4>
            </div>
            <label className="flex flex-col gap-2">
              <span className="text-sm text-text-primary">Select implant target</span>
              <select
                id="implant-target-tooth"
                aria-label="Implant target tooth"
                title="Implant target tooth"
                value={selectedTooth}
                onChange={(event) => setSelectedTooth(event.target.value)}
                className="w-full rounded bg-card border border-border-visible p-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-display"
              >
                {implantTargets.map((target) => (
                  <option key={target.toothNumber} value={target.toothNumber}>
                    Tooth {target.toothNumber} · {target.implantMode} · {target.restorationLabel}
                  </option>
                ))}
              </select>
            </label>
            {selectedTarget ? (
              <div className="mt-3 grid gap-2 text-sm text-text-secondary">
                <p>Restoration: <span className="text-text-primary">{selectedTarget.restorationLabel}</span></p>
                <p>Implant mode: <span className="text-text-primary">{selectedTarget.implantMode}</span></p>
                <p>Pre-op model: <span className="text-text-primary">{selectedTarget.usesPreOpModel ? 'Required' : 'Optional'}</span></p>
                <p>Extra gingiva scan: <span className="text-text-primary">{selectedTarget.needsExtraGingivaScan ? 'Required' : 'Optional'}</span></p>
              </div>
            ) : null}
          </section>
        ) : (
          <section className="rounded-2xl border border-dashed border-border bg-surface-raised p-4 text-sm text-text-secondary">
            Mark at least one tooth as <span className="text-text-primary">implant restoration</span> or set an implant mode in the TlantiCAD Workspace to unlock case-aware implant planning.
          </section>
        )}

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="mb-3 flex items-center gap-2">
            <CircleDot size={14} className="text-text-display" />
            <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Restorative workflow</h4>
          </div>
          <div className="grid gap-3 sm:grid-cols-2">
            {RESTORATIVE_WORKFLOWS.map((option) => (
              <button
                key={option.value}
                type="button"
                onClick={() => setWorkflowType(option.value)}
                className={clsx(
                  'rounded-2xl border px-4 py-3 text-left transition-colors',
                  workflowType === option.value
                    ? 'border-text-display bg-card text-text-primary'
                    : 'border-border bg-surface text-text-secondary hover:bg-card hover:text-text-primary',
                )}
              >
                <p className="text-sm font-semibold text-current">{option.label}</p>
                <p className="mt-1 text-xs text-text-secondary text-pretty">
                  {option.value === 'post-and-core'
                    ? 'Material config, post/core material, scan body branch, post bottom and core spacing.'
                    : 'CBCT + surface scan, implant library, safety zones, sleeve preview and planning report.'}
                </p>
              </button>
            ))}
          </div>
        </section>

        <div className="flex flex-col gap-2">
          <label htmlFor="implant-type" className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Implant Type</label>
          <select 
            id="implant-type"
            aria-label="Implant type"
            title="Implant type"
            value={implantType}
            onChange={(e) => setImplantType(e.target.value)}
            className="w-full p-2 rounded bg-surface-raised border border-border-visible outline-none text-sm font-mono text-text-primary focus:border-text-display transition-colors"
          >
            {IMPLANT_TYPES.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
        </div>

        {workflowType === 'post-and-core' ? (
          <section className="rounded-2xl border border-border bg-surface-raised p-4">
            <div className="mb-3 flex items-center gap-2">
              <Files size={14} className="text-text-secondary" />
              <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Post and Core setup</h4>
            </div>

            <div className="grid gap-4 sm:grid-cols-2">
              <label className="flex flex-col gap-2">
                <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Restoration material</span>
                <select
                  value={restorationMaterial}
                  onChange={(event) => setRestorationMaterial(event.target.value as (typeof RESTORATION_MATERIALS)[number])}
                  className="w-full rounded bg-card border border-border-visible p-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-display"
                >
                  {RESTORATION_MATERIALS.map((material) => (
                    <option key={material} value={material}>{material}</option>
                  ))}
                </select>
              </label>

              <label className="flex flex-col gap-2">
                <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Post and Core material</span>
                <select
                  value={postCoreMaterial}
                  onChange={(event) => setPostCoreMaterial(event.target.value as (typeof POST_CORE_MATERIALS)[number])}
                  className="w-full rounded bg-card border border-border-visible p-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-display"
                >
                  {POST_CORE_MATERIALS.map((material) => (
                    <option key={material} value={material}>{material}</option>
                  ))}
                </select>
              </label>
            </div>

            <div className="mt-4 grid gap-3 sm:grid-cols-2">
              <button
                type="button"
                onClick={() => setUseScanBody(true)}
                className={clsx(
                  'rounded-2xl border px-4 py-3 text-left transition-colors',
                  useScanBody ? 'border-text-display bg-card text-text-primary' : 'border-border bg-surface text-text-secondary hover:bg-card hover:text-text-primary',
                )}
              >
                <p className="text-sm font-semibold">Use Scan Body</p>
                <p className="mt-1 text-xs text-text-secondary text-pretty">Wizard starts with marker detection. The extra scan must contain the scan body only.</p>
              </button>
              <button
                type="button"
                onClick={() => setUseScanBody(false)}
                className={clsx(
                  'rounded-2xl border px-4 py-3 text-left transition-colors',
                  !useScanBody ? 'border-text-display bg-card text-text-primary' : 'border-border bg-surface text-text-secondary hover:bg-card hover:text-text-primary',
                )}
              >
                <p className="text-sm font-semibold">Don’t use Scan Body</p>
                <p className="mt-1 text-xs text-text-secondary text-pretty">Wizard starts directly on the inner margin line and skips marker matching.</p>
              </button>
            </div>

            <div className="mt-4 rounded-2xl border border-border bg-card px-4 py-4">
              <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Wizard sequence</p>
              <ol className="mt-3 space-y-2 text-sm text-text-primary">
                {postCoreSteps.map((step, index) => (
                  <li key={step} className="flex items-start gap-3 rounded-xl border border-border px-3 py-2">
                    <span className="inline-flex size-6 shrink-0 items-center justify-center rounded-full border border-border-visible bg-surface text-[11px] tabular-nums text-text-secondary">{index + 1}</span>
                    <span className="text-pretty">{step}</span>
                  </li>
                ))}
              </ol>
              {useScanBody ? (
                <p className="mt-3 text-xs text-amber-300 text-pretty">Warning: the scan-body matching scan must not include the patient’s teeth; keep it isolated to the marker/scan body scan.</p>
              ) : null}
            </div>
          </section>
        ) : null}

        <div className="grid grid-cols-2 gap-4">
          <div className="flex flex-col gap-2">
            <label htmlFor="implant-diameter" className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Diameter (mm)</label>
            <input 
              id="implant-diameter"
              title="Implant diameter in millimeters"
              placeholder="4.1"
              type="number" 
              step="0.1"
              value={diameter}
              onChange={(e) => setDiameter(parseFloat(e.target.value))}
              className="w-full p-2 rounded bg-surface-raised border border-border-visible outline-none text-sm font-mono text-text-primary focus:border-text-display transition-colors"
            />
          </div>
          <div className="flex flex-col gap-2">
            <label htmlFor="implant-length" className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Length (mm)</label>
            <input 
              id="implant-length"
              title="Implant length in millimeters"
              placeholder="10"
              type="number" 
              step="0.5"
              value={length}
              onChange={(e) => setLength(parseFloat(e.target.value))}
              className="w-full p-2 rounded bg-surface-raised border border-border-visible outline-none text-sm font-mono text-text-primary focus:border-text-display transition-colors"
            />
          </div>
        </div>

        <div className="flex flex-col gap-4 border-t border-border pt-4">
          <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary flex items-center gap-2"><Syringe size={14} /> Placement</h4>
          
          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
              <span>Angulation</span>
              <span>{angulation}°</span>
            </div>
            <input 
              title="Implant angulation"
              aria-label="Implant angulation"
              type="range" 
              min="-45" max="45" 
              value={angulation}
              onChange={(e) => setAngulation(parseFloat(e.target.value))}
              className="w-full accent-text-display"
            />
          </div>

          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
              <span>Depth Offset</span>
              <span>{depth} mm</span>
            </div>
            <input 
              title="Implant depth offset"
              aria-label="Implant depth offset"
              type="range" 
              min="-5" max="5" step="0.1"
              value={depth}
              onChange={(e) => setDepth(parseFloat(e.target.value))}
              className="w-full accent-text-display"
            />
          </div>
        </div>

        {workflowType === 'post-and-core' ? (
          <section className="rounded-2xl border border-border bg-surface-raised p-4">
            <div className="mb-3 flex items-center gap-2">
              <CircleDot size={14} className="text-text-display" />
              <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Design Core</h4>
            </div>

            <div className="space-y-4">
              <div className="flex flex-col gap-2">
                <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
                  <span>Post Bottom</span>
                  <span>{postBottom.toFixed(1)} mm</span>
                </div>
                <input type="range" min="0" max="3" step="0.1" value={postBottom} onChange={(e) => setPostBottom(parseFloat(e.target.value))} className="w-full accent-text-display" aria-label="Post bottom" />
              </div>

              <div className="flex flex-col gap-2">
                <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
                  <span>Core min angle</span>
                  <span>{coreMinAngle.toFixed(0)}°</span>
                </div>
                <input type="range" min="2" max="20" step="1" value={coreMinAngle} onChange={(e) => setCoreMinAngle(parseFloat(e.target.value))} className="w-full accent-text-display" aria-label="Core minimum angle" />
              </div>

              <div className="flex flex-col gap-2">
                <div className="flex justify-between text-[11px] font-mono uppercase tracking-widest text-text-primary">
                  <span>Core spacing</span>
                  <span>{coreSpacing.toFixed(2)} mm</span>
                </div>
                <input type="range" min="0" max="0.3" step="0.01" value={coreSpacing} onChange={(e) => setCoreSpacing(parseFloat(e.target.value))} className="w-full accent-text-display" aria-label="Core spacing" />
              </div>

              <label className="flex items-center gap-3 rounded-xl border border-border bg-card px-3 py-3 text-sm text-text-primary">
                <input type="checkbox" checked={autoAdaptOcclusal} onChange={(event) => setAutoAdaptOcclusal(event.target.checked)} className="accent-text-display" />
                Auto adapt occlusal area to keep defined spacing
              </label>
            </div>
          </section>
        ) : null}

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="mb-3 flex items-center gap-2">
            <Files size={14} className="text-text-secondary" />
            <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Clinical readiness</h4>
          </div>
          <div className="grid gap-2 text-sm text-text-secondary">
            <p className="flex items-center gap-2"><ScanSearch size={14} className={workflowType === 'post-and-core' ? 'text-text-secondary' : assetReadiness.hasDicomStudy ? 'text-green-400' : 'text-red-400'} /> DICOM / CBCT study {workflowType === 'post-and-core' ? 'optional for restorative branch' : assetReadiness.hasDicomStudy ? 'ready' : 'missing'}</p>
            <p className="flex items-center gap-2"><ScanSearch size={14} className={assetReadiness.hasPrepScan || assetReadiness.hasGingivaScan ? 'text-green-400' : 'text-red-400'} /> Prep or gingiva scan {assetReadiness.hasPrepScan || assetReadiness.hasGingivaScan ? 'ready' : 'missing'}</p>
            <p className="flex items-center gap-2"><ScanSearch size={14} className={assetReadiness.hasOpposingRecord ? 'text-green-400' : 'text-text-secondary'} /> Opposing / bite record {assetReadiness.hasOpposingRecord ? 'attached' : 'recommended'}</p>
            {workflowType === 'post-and-core' ? (
              <p className="flex items-center gap-2"><AlertCircle size={14} className={useScanBody && !assetReadiness.hasGingivaScan ? 'text-amber-400' : 'text-text-secondary'} /> Scan body branch {useScanBody ? (assetReadiness.hasGingivaScan ? 'extra isolated scan attached' : 'extra isolated scan required') : 'disabled'}</p>
            ) : null}
            <p className="flex items-center gap-2"><AlertCircle size={14} className={requiresSoftTissue && !assetReadiness.hasGingivaScan ? 'text-amber-400' : 'text-text-secondary'} /> Soft tissue scan {requiresSoftTissue ? (assetReadiness.hasGingivaScan ? 'ready' : 'required for selected tooth') : 'optional for selected tooth'}</p>
          </div>
        </section>

        <button 
          onClick={handleGeneratePlanningBrief}
          disabled={!canGeneratePlanningBrief}
          className="mt-4 w-full py-3 bg-text-display text-black font-mono text-xs uppercase tracking-widest font-bold rounded hover:bg-white transition-colors flex items-center justify-center gap-2 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Check size={16} /> Generate {workflowType === 'post-and-core' ? 'Post and Core' : 'implant'} brief
        </button>
      </div>
    </motion.div>
  );
};

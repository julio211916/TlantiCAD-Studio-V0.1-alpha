import React, { useEffect, useMemo, useState } from 'react'
import { motion } from 'framer-motion'
import { AlertCircle, Check, Download, MoonStar, ScanSearch } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { downloadJsonBrief, getClinicalAssetReadiness, getRecommendedSplintIndication, getSplintTargets, type SplintIndication } from '@/lib/clinical-module-briefs'
import { cn } from '@/lib/utils'
import type { TlantiCase } from '@/stores/tlantidb-case-store'
import { useViewportProfile } from '../hooks/useViewportProfile'
import type { ThemeMode } from '../types'
import { useToast } from './ui/use-toast'
import { commandRegistry } from '@/features/command-palette'

interface SplintModuleProps {
  themeMode: ThemeMode
  onClose: () => void
  activeCase?: TlantiCase | null
}

const SPLINT_INDICATIONS: Array<{ value: SplintIndication; label: string; description: string }> = [
  { value: 'stabilization', label: 'Stabilization splint', description: 'General full-arch stabilization with balanced support.' },
  { value: 'deprogrammer', label: 'Deprogrammer', description: 'Anterior guidance and occlusal deprogramming focus.' },
  { value: 'night-guard', label: 'Night guard', description: 'Protective overnight splint for parafunction workflows.' },
  { value: 'surgical-provisional', label: 'Surgical provisional', description: 'Provisional support around implant / guided workflows.' },
]

const INSERTION_PATHS = ['neutral', 'anterior', 'posterior', 'implant-guided'] as const

export function SplintModule({ themeMode, onClose, activeCase }: SplintModuleProps) {
  const { toast } = useToast()
  const viewport = useViewportProfile()
  const readiness = useMemo(() => getClinicalAssetReadiness(activeCase?.assets ?? []), [activeCase?.assets])
  const splintTargets = useMemo(() => getSplintTargets(activeCase), [activeCase])
  const [indication, setIndication] = useState<SplintIndication>(() => getRecommendedSplintIndication(activeCase, readiness))
  const [thickness, setThickness] = useState(1.8)
  const [relief, setRelief] = useState(0.12)
  const [occlusalOffset, setOcclusalOffset] = useState(0.4)
  const [insertionPath, setInsertionPath] = useState<(typeof INSERTION_PATHS)[number]>('neutral')

  const requiresOpposing = indication !== 'surgical-provisional'
  const canExport = Boolean(activeCase && splintTargets.length && readiness.hasPrepScan && (!requiresOpposing || readiness.hasOpposingRecord))

  const exportBrief = () => {
    if (!activeCase) {
      toast('Open an active case before exporting splint planning data.', 'error')
      return
    }

    if (!splintTargets.length) {
      toast('Select clinical teeth in the TlantiCAD Workspace before preparing a splint brief.', 'error')
      return
    }

    if (!readiness.hasPrepScan) {
      toast('A prep / restorative scan is required before splint export.', 'error')
      return
    }

    if (requiresOpposing && !readiness.hasOpposingRecord) {
      toast('Splint workflows need antagonist or bite registration data.', 'error')
      return
    }

    downloadJsonBrief(`${activeCase.caseNumber}-splint-brief.json`, {
      kind: 'splint-planning-brief',
      createdAt: new Date().toISOString(),
      caseId: activeCase.id,
      caseNumber: activeCase.caseNumber,
      caseName: activeCase.name,
      activeJaw: activeCase.activeJaw,
      occlusionScanType: activeCase.occlusionScanType,
      indication,
      parameters: {
        thickness,
        relief,
        occlusalOffset,
        insertionPath,
      },
      readiness: {
        prepScan: readiness.hasPrepScan,
        opposingRecord: readiness.hasOpposingRecord,
        gingivaScan: readiness.hasGingivaScan,
        manufacturingReport: readiness.hasManufacturingReport,
      },
      targets: splintTargets,
      assets: activeCase.assets ?? [],
    })
    toast('Splint brief exported for clinical review.', 'success')
  }

  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.splint.close',
        label: 'Cerrar módulo Splint',
        kind: 'navigation',
        keywords: ['splint', 'close', 'cerrar'],
        run: onClose,
      },
      {
        id: 'cad.module.splint.export',
        label: 'Splint: exportar brief',
        kind: 'tool',
        keywords: ['splint', 'export', 'brief'],
        available: () => canExport,
        run: exportBrief,
      },
    ])
  }, [onClose, canExport, exportBrief])

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className={cn(
        'absolute z-40 flex flex-col overflow-hidden rounded-lg border shadow-2xl',
        viewport.isCompact ? 'inset-3 top-[8rem]' : 'left-16 top-24 bottom-24 w-80',
        themeMode === 'dark' ? 'bg-surface border-border text-text-primary' : 'bg-surface border-border text-text-primary',
      )}
    >
      <div className="flex items-center justify-between border-b border-border bg-surface-raised p-4">
        <div className="flex items-center gap-2">
          <MoonStar size={18} className="text-text-display" />
          <h3 className="font-display font-semibold tracking-tight text-text-primary">Splint Workflow</h3>
        </div>
        <button onClick={onClose} title="Close splint workflow" aria-label="Close splint workflow" className="text-text-secondary transition-colors hover:text-text-primary">✕</button>
      </div>

      <div className="flex flex-1 flex-col gap-6 overflow-y-auto bg-surface p-4">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase text-text-secondary">Splint case</p>
              <h4 className="mt-1 text-balance text-lg font-semibold text-text-display">{activeCase ? `${activeCase.caseNumber} · ${activeCase.name}` : 'No active case in CAD'}</h4>
              <p className="mt-1 text-pretty text-sm text-text-secondary">Full-arch splint planning with scan readiness, occlusal context and exportable brief.</p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline" className="border-border bg-card text-text-primary">{splintTargets.length} target teeth</Badge>
              <Badge variant="outline" className="border-border bg-card text-text-secondary">{activeCase?.activeJaw ?? 'jaw n/a'}</Badge>
            </div>
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <label htmlFor="splint-indication" className="text-[11px] uppercase text-text-secondary">Indication</label>
          <select
            id="splint-indication"
            title="Splint indication"
            aria-label="Splint indication"
            value={indication}
            onChange={(event) => setIndication(event.target.value as SplintIndication)}
            className="mt-2 w-full rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary"
          >
            {SPLINT_INDICATIONS.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
          <p className="mt-2 text-sm text-text-secondary">{SPLINT_INDICATIONS.find((option) => option.value === indication)?.description}</p>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="grid gap-4">
            <div className="flex flex-col gap-2">
              <div className="flex justify-between text-[11px] uppercase text-text-primary"><span>Thickness</span><span>{thickness.toFixed(1)} mm</span></div>
              <input title="Splint thickness" aria-label="Splint thickness" type="range" min="1.0" max="4.0" step="0.1" value={thickness} onChange={(event) => setThickness(parseFloat(event.target.value))} className="w-full accent-text-display" />
            </div>
            <div className="flex flex-col gap-2">
              <div className="flex justify-between text-[11px] uppercase text-text-primary"><span>Selective relief</span><span>{relief.toFixed(2)} mm</span></div>
              <input title="Splint selective relief" aria-label="Splint selective relief" type="range" min="0.0" max="0.4" step="0.01" value={relief} onChange={(event) => setRelief(parseFloat(event.target.value))} className="w-full accent-text-display" />
            </div>
            <div className="flex flex-col gap-2">
              <div className="flex justify-between text-[11px] uppercase text-text-primary"><span>Occlusal offset</span><span>{occlusalOffset.toFixed(2)} mm</span></div>
              <input title="Splint occlusal offset" aria-label="Splint occlusal offset" type="range" min="0.0" max="1.0" step="0.05" value={occlusalOffset} onChange={(event) => setOcclusalOffset(parseFloat(event.target.value))} className="w-full accent-text-display" />
            </div>
            <label className="grid gap-2">
              <span className="text-[11px] uppercase text-text-secondary">Insertion path</span>
              <select value={insertionPath} onChange={(event) => setInsertionPath(event.target.value as (typeof INSERTION_PATHS)[number])} title="Splint insertion path" aria-label="Splint insertion path" className="rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary">
                {INSERTION_PATHS.map((path) => (
                  <option key={path} value={path}>{path}</option>
                ))}
              </select>
            </label>
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
          <div className="mb-3 flex items-center gap-2">
            <ScanSearch size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Clinical gates</h4>
          </div>
          <div className="grid gap-2">
            <p className="flex items-center gap-2"><ScanSearch size={14} className={readiness.hasPrepScan ? 'text-green-400' : 'text-red-400'} /> Prep / restorative scan {readiness.hasPrepScan ? 'ready' : 'missing'}</p>
            <p className="flex items-center gap-2"><ScanSearch size={14} className={!requiresOpposing || readiness.hasOpposingRecord ? 'text-green-400' : 'text-amber-400'} /> Opposing / bite registration {!requiresOpposing ? 'optional for this indication' : readiness.hasOpposingRecord ? 'ready' : 'required'}</p>
            <p className="flex items-center gap-2"><AlertCircle size={14} className={splintTargets.some((item) => item.usesPreOpModel) ? 'text-text-primary' : 'text-text-secondary'} /> Pre-op model {splintTargets.some((item) => item.usesPreOpModel) ? 'requested on selected teeth' : 'optional'}</p>
          </div>
        </section>

        {splintTargets.length ? (
          <section className="rounded-2xl border border-border bg-surface-raised p-4 text-sm text-text-secondary">
            <div className="mb-3 flex items-center gap-2"><MoonStar size={14} className="text-text-display" /><h4 className="text-[11px] uppercase text-text-secondary">Target teeth</h4></div>
            <div className="flex flex-wrap gap-2">
              {splintTargets.map((target) => (
                <Badge key={target.toothNumber} variant="outline" className="border-border bg-card text-text-primary">Tooth {target.toothNumber} · {target.restorationLabel}</Badge>
              ))}
            </div>
          </section>
        ) : null}

        <div className="mt-auto flex flex-col gap-2">
          <button type="button" onClick={exportBrief} disabled={!canExport} className="flex w-full items-center justify-center gap-2 rounded bg-text-display py-3 text-xs font-bold uppercase tracking-widest text-black transition-colors hover:bg-white disabled:cursor-not-allowed disabled:opacity-50">
            <Download size={16} /> Export splint brief
          </button>
          <button type="button" onClick={onClose} className="flex w-full items-center justify-center gap-2 rounded border border-border-visible bg-surface-raised py-3 text-xs font-bold uppercase tracking-widest text-text-primary transition-colors hover:bg-surface">
            <Check size={16} /> Keep reviewing in CAD
          </button>
        </div>
      </div>
    </motion.div>
  )
}
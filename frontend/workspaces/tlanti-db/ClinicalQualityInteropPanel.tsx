import React, { useMemo } from 'react'
import { FileJson2, RefreshCw, ShieldCheck, ShieldAlert, Workflow, Database, FolderSync } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import {
  buildClinicalQualityGateReport,
  buildInteropReadinessManifest,
  exportClinicalQualityGateReport,
  exportInteropReadinessManifest,
  type QualityGateRisk,
} from '@/lib/clinical-quality-interop'
import type { BackendCatalog, PythonBridgeStatus, SystemRuntimeReport } from '@/lib/backend-integrations'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import type { TlantiCase, TlantiDbState } from '@/stores/tlantidb-case-store'
import { cn } from '@/lib/utils'

interface ClinicalQualityInteropPanelProps {
  activeCase: TlantiCase
  state: TlantiDbState
  systemRuntimeReport: SystemRuntimeReport | null
  pythonBridge: PythonBridgeStatus | null
  backendCatalog: BackendCatalog | null
  runtimeDiagnostics: RuntimeDiagnostics | null
  runtimeDiagnosticsLoading: boolean
  onRefreshRuntime: () => void
}

function MetricCard({ label, value, helper }: { label: string; value: string; helper?: string }) {
  return (
    <div className="rounded-2xl border border-border bg-surface px-3 py-3">
      <p className="text-[11px] uppercase text-text-secondary">{label}</p>
      <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{value}</p>
      {helper ? <p className="mt-1 text-xs text-text-secondary text-pretty">{helper}</p> : null}
    </div>
  )
}

function riskTone(risk: QualityGateRisk) {
  if (risk.severity === 'high') return 'text-rose-300'
  if (risk.severity === 'medium') return 'text-amber-300'
  return 'text-sky-300'
}

export function ClinicalQualityInteropPanel({
  activeCase,
  state,
  systemRuntimeReport,
  pythonBridge,
  backendCatalog,
  runtimeDiagnostics,
  runtimeDiagnosticsLoading,
  onRefreshRuntime,
}: ClinicalQualityInteropPanelProps) {
  const report = useMemo(() => buildClinicalQualityGateReport({
    activeCase,
    state,
    systemRuntimeReport,
    pythonBridge,
    backendCatalog,
    runtimeDiagnostics,
  }), [activeCase, backendCatalog, pythonBridge, runtimeDiagnostics, state, systemRuntimeReport])

  const interopManifest = useMemo(() => buildInteropReadinessManifest(report, state, activeCase), [activeCase, report, state])

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <ShieldCheck className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary text-balance">Quality gate + interoperability readiness</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Slice operativo de los sprints <span className="font-medium text-text-primary">101–120</span>: calidad clínica, reproducibilidad, export readiness y preparación para interoperabilidad.
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">101–110 quality gate</Badge>
          <Badge variant="outline">111–120 interop</Badge>
          <Badge variant="outline">{report.releaseRecommendation === 'ready' ? 'Ready' : 'Hold'}</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard label="Clinical score" value={`${report.scorecard.clinical}/100`} helper={`${report.checklistSummary.requiredComplete}/${report.checklistSummary.requiredTotal} required checklist items complete`} />
        <MetricCard label="Reproducibility" value={`${report.scorecard.reproducibility}/100`} helper={report.storagePath ?? 'Case folder not persisted yet'} />
        <MetricCard label="Interop" value={`${report.scorecard.interop}/100`} helper={report.interopXmlPath ?? 'No saved XML interop yet'} />
        <MetricCard label="Security" value={`${report.scorecard.security}/100`} helper={report.runtimeSummary.diagnosticsLoaded ? 'Runtime diagnostics captured' : 'Diagnostics still pending'} />
      </div>

      <div className="mt-4 rounded-2xl border border-border bg-surface px-4 py-4">
        <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div>
            <p className="text-xs uppercase text-text-secondary">Overall gate</p>
            <p className="mt-1 text-2xl font-semibold text-text-display tabular-nums">{report.scorecard.overall}/100</p>
            <p className="mt-1 text-xs text-text-secondary text-pretty">
              Recommendation: <span className="font-medium text-text-primary">{report.releaseRecommendation === 'ready' ? 'ready to continue' : 'hold and remediate blockers'}</span>
            </p>
          </div>

          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => exportClinicalQualityGateReport(report)}
              className="inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
            >
              <FileJson2 className="size-4" />
              Export quality gate JSON
            </button>
            <button
              type="button"
              onClick={() => exportInteropReadinessManifest(interopManifest)}
              className="inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
            >
              <FolderSync className="size-4" />
              Export interop manifest
            </button>
            <button
              type="button"
              onClick={onRefreshRuntime}
              className="inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
            >
              <RefreshCw className={cn('size-4', runtimeDiagnosticsLoading && 'animate-spin')} />
              Refresh runtime inputs
            </button>
          </div>
        </div>
      </div>

      <div className="mt-4 grid gap-4 xl:grid-cols-[1.05fr_0.95fr]">
        <div className="rounded-2xl border border-border bg-surface px-4 py-4">
          <div className="mb-3 flex items-center gap-2">
            <ShieldAlert className="size-4 text-text-secondary" />
            <p className="text-xs uppercase text-text-secondary">Active risk register</p>
          </div>

          <div className="space-y-2">
            {report.risks.map((risk) => (
              <div key={risk.id} className="rounded-xl border border-border bg-card px-3 py-3">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className={cn('text-sm font-semibold', riskTone(risk))}>{risk.title}</p>
                    <p className="mt-1 text-xs text-text-secondary text-pretty">{risk.detail}</p>
                  </div>
                  <Badge variant="outline">{risk.severity}</Badge>
                </div>
                <p className="mt-2 text-[11px] uppercase text-text-secondary">{risk.category} · {risk.blocked ? 'blocked' : 'watch'}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-2xl border border-border bg-surface px-4 py-4">
          <div className="mb-3 flex items-center gap-2">
            <Workflow className="size-4 text-text-secondary" />
            <p className="text-xs uppercase text-text-secondary">Interop readiness map</p>
          </div>

          <div className="space-y-2">
            {report.interopCapabilities.map((capability) => (
              <div key={capability.id} className="rounded-xl border border-border bg-card px-3 py-3">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-semibold text-text-display">{capability.label}</p>
                  <Badge variant="outline">{capability.status}</Badge>
                </div>
                <p className="mt-2 text-xs text-text-secondary text-pretty">{capability.detail}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="mt-4 grid gap-4 xl:grid-cols-2">
        <div className="rounded-2xl border border-border bg-surface px-4 py-4">
          <div className="mb-3 flex items-center gap-2">
            <Database className="size-4 text-text-secondary" />
            <p className="text-xs uppercase text-text-secondary">Checklist + pipeline evidence</p>
          </div>
          <div className="grid gap-3 md:grid-cols-2">
            <MetricCard label="Pipeline" value={Object.entries(report.pipeline).filter(([, active]) => active).map(([key]) => key).join(' · ') || 'none'} />
            <MetricCard label="Optional items" value={`${report.checklistSummary.optionalComplete}/${report.checklistSummary.optionalTotal}`} helper="Optional clinical signals completed" />
            <MetricCard label="Runtime workflows" value={`${report.runtimeSummary.readyWorkflows}/${report.runtimeSummary.totalWorkflows}`} helper={report.runtimeSummary.recommendedRenderQuality ?? 'No runtime report yet'} />
            <MetricCard label="Platform" value={report.runtimeSummary.platform ?? 'pending'} helper={runtimeDiagnostics?.caseLocation ?? 'No diagnostics captured'} />
          </div>
        </div>

        <div className="rounded-2xl border border-border bg-surface px-4 py-4">
          <div className="mb-3 flex items-center gap-2">
            <FileJson2 className="size-4 text-text-secondary" />
            <p className="text-xs uppercase text-text-secondary">Interop manifest preview</p>
          </div>
          <div className="space-y-2 rounded-2xl border border-border bg-card px-3 py-3 text-xs text-text-secondary">
            <p><span className="text-text-primary">Case:</span> {interopManifest.caseNumber}</p>
            <p><span className="text-text-primary">Operator:</span> {interopManifest.operatorAlias}</p>
            <p><span className="text-text-primary">Numbering:</span> {interopManifest.numberingSystem}</p>
            <p><span className="text-text-primary">Client alias:</span> {interopManifest.clientAlias || 'missing'}</p>
            <p><span className="text-text-primary">Direct identifiers:</span> {interopManifest.directIdentifiersPresent ? 'present' : 'not present'}</p>
            <p><span className="text-text-primary">Blocked lanes:</span> {interopManifest.blockers.length ? interopManifest.blockers.join(' · ') : 'none'}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
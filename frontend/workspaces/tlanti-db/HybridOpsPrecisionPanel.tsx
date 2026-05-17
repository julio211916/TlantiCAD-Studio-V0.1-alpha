import React, { useMemo, useState } from 'react'
import { Cloud, CloudOff, Cpu, FileJson2, Gauge, Layers3, RefreshCw, Shield, Shuffle, Wallet } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import {
  buildHybridOpsReadinessReport,
  buildPrecisionKernelAlphaReport,
  createRemoteJobDraft,
  exportHybridOpsReadiness,
  exportPrecisionKernelAlpha,
  updateRemoteJobStatus,
} from '@/lib/hybrid-ops-precision'
import type { BackendCatalog, BackendWorkspaceCatalog, SystemRuntimeReport } from '@/lib/backend-integrations'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import { cn } from '@/lib/utils'
import type {
  TlantiCadKernel,
  TlantiCadKernelPreference,
  TlantiCase,
  TlantiCaseOperationsState,
  TlantiCaseRemoteJobTarget,
  TlantiDbState,
} from '@/stores/tlantidb-case-store'

interface HybridOpsPrecisionPanelProps {
  activeCase: TlantiCase
  state: TlantiDbState
  operations: TlantiCaseOperationsState
  runtimeReport: SystemRuntimeReport | null
  backendCatalog: BackendCatalog | null
  workspaceCatalog: BackendWorkspaceCatalog | null
  diagnostics: RuntimeDiagnostics | null
  onPatchOperations: (updater: (current: TlantiCaseOperationsState) => TlantiCaseOperationsState) => void
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

export function HybridOpsPrecisionPanel({ activeCase, state, operations, runtimeReport, backendCatalog, workspaceCatalog, diagnostics, onPatchOperations }: HybridOpsPrecisionPanelProps) {
  const [jobLabel, setJobLabel] = useState('Distributed mesh boolean')
  const [jobTarget, setJobTarget] = useState<TlantiCaseRemoteJobTarget>('hybrid')

  const hybridReport = useMemo(() => buildHybridOpsReadinessReport({
    activeCase,
    state,
    operations,
    runtimeReport,
    backendCatalog,
    diagnostics,
  }), [activeCase, backendCatalog, diagnostics, operations, runtimeReport, state])

  const kernelReport = useMemo(() => buildPrecisionKernelAlphaReport({
    activeCase,
    operations,
    backendCatalog,
    workspaceCatalog,
  }), [activeCase, backendCatalog, operations, workspaceCatalog])

  const queueJob = () => {
    const label = jobLabel.trim()
    if (!label) return
    const quota = jobTarget === 'cloud' ? 24 : jobTarget === 'hybrid' ? 12 : 6
    const cost = jobTarget === 'cloud' ? 3.4 : jobTarget === 'hybrid' ? 1.6 : 0.4
    onPatchOperations((current) => ({
      ...current,
      remoteJobs: [createRemoteJobDraft(label, jobTarget, quota, cost), ...current.remoteJobs],
    }))
  }

  const mutateJob = (jobId: string, kind: 'running' | 'retrying' | 'offline-fallback' | 'completed' | 'failed') => {
    onPatchOperations((current) => ({
      ...current,
      remoteJobs: current.remoteJobs.map((job) => job.id === jobId
        ? updateRemoteJobStatus(job, kind, kind === 'failed' ? 'Manual failure flag for resilience drill.' : null)
        : job),
    }))
  }

  const updateKernelPreference = (preference: TlantiCadKernelPreference) => {
    onPatchOperations((current) => ({
      ...current,
      kernelTransition: {
        ...current.kernelTransition,
        preference,
        lastPolicyUpdateAt: new Date().toISOString(),
      },
    }))
  }

  const updatePreferredKernel = (preferredKernel: TlantiCadKernel) => {
    onPatchOperations((current) => ({
      ...current,
      kernelTransition: {
        ...current.kernelTransition,
        preferredKernel,
        lastPolicyUpdateAt: new Date().toISOString(),
      },
    }))
  }

  const toggleOfflineFallback = () => {
    onPatchOperations((current) => ({
      ...current,
      kernelTransition: {
        ...current.kernelTransition,
        offlineFallback: !current.kernelTransition.offlineFallback,
        lastPolicyUpdateAt: new Date().toISOString(),
      },
    }))
  }

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <Cloud className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary text-balance">Hybrid ops + precision kernel readiness</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Tranche operativo de los sprints <span className="font-medium text-text-primary">141–160</span>: cola remota/híbrida con fallback offline y transición controlada de kernel mesh-first a precisión CAD.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">141–150 hybrid ops</Badge>
          <Badge variant="outline">151–160 precision alpha</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard label="Queued jobs" value={`${hybridReport.queueSummary.total}`} helper={`${hybridReport.queueSummary.active} active / ${hybridReport.queueSummary.failed} failed`} />
        <MetricCard label="Quota units" value={`${hybridReport.queueSummary.quotaUnits}`} helper={hybridReport.readiness.offlineFallbackEnabled ? 'Offline fallback on' : 'Offline fallback off'} />
        <MetricCard label="Estimated cost" value={`$${hybridReport.queueSummary.estimatedCostUsd.toFixed(2)}`} helper="Exploratory cost envelope" />
        <MetricCard label="Kernel benchmark" value={`${kernelReport.kernelStatus.benchmarkScore}/100`} helper={`${kernelReport.preferredKernel} · ${kernelReport.preference}`} />
      </div>

      <div className="mt-4 grid gap-4 xl:grid-cols-[1.02fr_0.98fr]">
        <div className="space-y-4">
          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <Shuffle className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Remote job queue</p>
            </div>
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_12rem_auto]">
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Job label</span>
                <input value={jobLabel} onChange={(event) => setJobLabel(event.target.value)} className="rounded border border-border-visible bg-card px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Remote job label" />
              </label>
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Target</span>
                <select value={jobTarget} onChange={(event) => setJobTarget(event.target.value as TlantiCaseRemoteJobTarget)} className="rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Remote job target">
                  <option value="local">local</option>
                  <option value="hybrid">hybrid</option>
                  <option value="cloud">cloud</option>
                </select>
              </label>
              <div className="flex items-end">
                <button type="button" onClick={queueJob} className="w-full rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Queue</button>
              </div>
            </div>
            <div className="mt-4 space-y-2">
              {operations.remoteJobs.length ? operations.remoteJobs.map((job) => (
                <div key={job.id} className="rounded-xl border border-border bg-card px-3 py-3">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <p className="text-sm font-semibold text-text-display">{job.label}</p>
                      <p className="mt-1 text-xs text-text-secondary">{job.target} · quota {job.quotaUnits} · ${job.estimatedCostUsd.toFixed(2)}</p>
                    </div>
                    <Badge variant="outline">{job.status}</Badge>
                  </div>
                  <div className="mt-3 flex flex-wrap gap-2">
                    <button type="button" onClick={() => mutateJob(job.id, 'running')} className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-primary transition-colors hover:bg-surface-raised">Run</button>
                    <button type="button" onClick={() => mutateJob(job.id, 'retrying')} className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-primary transition-colors hover:bg-surface-raised">Retry</button>
                    <button type="button" onClick={() => mutateJob(job.id, 'offline-fallback')} className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-primary transition-colors hover:bg-surface-raised">Offline fallback</button>
                    <button type="button" onClick={() => mutateJob(job.id, 'completed')} className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-primary transition-colors hover:bg-surface-raised">Complete</button>
                    <button type="button" onClick={() => mutateJob(job.id, 'failed')} className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-primary transition-colors hover:bg-surface-raised">Fail drill</button>
                  </div>
                </div>
              )) : (
                <div className="rounded-xl border border-dashed border-border bg-card px-3 py-4 text-sm text-text-secondary">No remote jobs yet. Queue one to simulate hybrid/local/cloud distribution with quotas and retries.</div>
              )}
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" onClick={() => exportHybridOpsReadiness(hybridReport)} className="inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">
                <FileJson2 className="size-4" /> Export hybrid ops JSON
              </button>
            </div>
          </section>

          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <Wallet className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Hybrid readiness + quotas</p>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <MetricCard label="Persisted case" value={hybridReport.readiness.persistedCaseFolder ? 'Ready' : 'Pending'} helper={activeCase.storagePath ?? 'No `.tlanticad` folder yet'} />
              <MetricCard label="Asset sync" value={hybridReport.readiness.syncReady ? 'Ready' : 'Blocked'} helper={hybridReport.readiness.assetsAttached ? `${activeCase.assets?.length ?? 0} assets attached` : 'No assets attached'} />
              <MetricCard label="Encrypted transport" value={hybridReport.readiness.transportEncryptionReady ? 'Enabled' : 'Mixed'} />
              <MetricCard label="Runtime evidence" value={hybridReport.readiness.runtimeLoaded ? 'Loaded' : 'Missing'} helper={diagnostics?.platform ?? 'No diagnostics'} />
            </div>
            <div className="mt-3 space-y-1 text-xs text-text-secondary">
              {hybridReport.blockers.map((item) => <p key={item}>• {item}</p>)}
              {hybridReport.recommendations.map((item) => <p key={item}>• {item}</p>)}
            </div>
          </section>
        </div>

        <div className="space-y-4">
          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <Layers3 className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Precision kernel policy</p>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Policy</span>
                <select value={operations.kernelTransition.preference} onChange={(event) => updateKernelPreference(event.target.value as TlantiCadKernelPreference)} className="rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Kernel policy">
                  <option value="mesh-first">mesh-first</option>
                  <option value="precision-cad">precision-cad</option>
                  <option value="hybrid-auto">hybrid-auto</option>
                </select>
              </label>
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Preferred kernel</span>
                <select value={operations.kernelTransition.preferredKernel} onChange={(event) => updatePreferredKernel(event.target.value as TlantiCadKernel)} className="rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Preferred kernel">
                  <option value="auto">auto</option>
                  <option value="manifold">manifold</option>
                  <option value="occt">occt</option>
                </select>
              </label>
            </div>
            <button type="button" onClick={toggleOfflineFallback} className="mt-3 inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">
              {operations.kernelTransition.offlineFallback ? <CloudOff className="size-4" /> : <Cloud className="size-4" />} Toggle offline fallback
            </button>
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              <MetricCard label="Mesh kernel" value={kernelReport.kernelStatus.meshKernelReady ? 'Ready' : 'Missing'} helper="Manifold / CSG evidence" />
              <MetricCard label="OCCT evidence" value={kernelReport.kernelStatus.occtEvidencePresent ? 'Present' : 'Pending'} helper="Workspace precision signals" />
              <MetricCard label="Conversion" value={kernelReport.kernelStatus.conversionReadiness ? 'Testable' : 'Pending'} helper="mesh↔BRep transition" />
              <MetricCard label="Tolerance" value={kernelReport.kernelStatus.toleranceReadiness ? 'Testable' : 'Pending'} helper={`Benchmark ${kernelReport.kernelStatus.benchmarkScore}/100`} />
            </div>
            <div className="mt-3 space-y-1 text-xs text-text-secondary">
              {kernelReport.evidence.map((item) => <p key={item}>• {item}</p>)}
              {kernelReport.constraints.map((item) => <p key={item}>• {item}</p>)}
              {kernelReport.transitionNotes.map((item) => <p key={item}>• {item}</p>)}
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" onClick={() => exportPrecisionKernelAlpha(kernelReport)} className="inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">
                <Cpu className="size-4" /> Export precision alpha JSON
              </button>
            </div>
          </section>

          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <Shield className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Runtime + crate evidence</p>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <MetricCard label="Render quality" value={runtimeReport?.system.capabilities.recommendedRenderQuality ?? 'pending'} helper={runtimeReport?.system.os.name ?? 'No runtime report'} />
              <MetricCard label="Workspace crates" value={`${workspaceCatalog?.crateCount ?? 0}`} helper={`${workspaceCatalog?.publicFunctionCount ?? 0} public functions`} />
              <MetricCard label="Backend routes" value={`${backendCatalog?.activeFeatureCount ?? 0}`} helper={`${backendCatalog?.experimentalFeatureCount ?? 0} experimental`} />
              <MetricCard label="Diagnostics" value={diagnostics ? 'Captured' : 'Pending'} helper={diagnostics?.hostname ?? 'Capture runtime diagnostics'} />
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}
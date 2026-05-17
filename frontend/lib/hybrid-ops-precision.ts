import type { BackendCatalog, BackendWorkspaceCatalog, SystemRuntimeReport } from '@/lib/backend-integrations'
import { downloadJsonBrief } from '@/lib/clinical-module-briefs'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import type {
  TlantiCadKernel,
  TlantiCadKernelPreference,
  TlantiCase,
  TlantiCaseOperationsState,
  TlantiCaseRemoteJob,
  TlantiCaseRemoteJobStatus,
  TlantiCaseRemoteJobTarget,
  TlantiDbState,
} from '@/stores/tlantidb-case-store'

export interface HybridOpsReadinessReport {
  kind: 'hybrid-ops-readiness'
  sprintRange: '141-150'
  generatedAt: string
  caseId: string
  caseNumber: string
  caseName: string
  queueSummary: {
    total: number
    active: number
    failed: number
    offlineFallback: number
    quotaUnits: number
    estimatedCostUsd: number
  }
  readiness: {
    persistedCaseFolder: boolean
    assetsAttached: boolean
    transportEncryptionReady: boolean
    syncReady: boolean
    runtimeLoaded: boolean
    backendLoaded: boolean
    offlineFallbackEnabled: boolean
  }
  blockers: string[]
  recommendations: string[]
  jobs: TlantiCaseRemoteJob[]
}

export interface PrecisionKernelAlphaReport {
  kind: 'precision-kernel-alpha'
  sprintRange: '151-160'
  generatedAt: string
  caseId: string
  caseNumber: string
  preference: TlantiCadKernelPreference
  preferredKernel: TlantiCadKernel
  kernelStatus: {
    meshKernelReady: boolean
    occtEvidencePresent: boolean
    conversionReadiness: boolean
    toleranceReadiness: boolean
    benchmarkScore: number
  }
  evidence: string[]
  constraints: string[]
  transitionNotes: string[]
}

export function createRemoteJobDraft(label: string, target: TlantiCaseRemoteJobTarget, quotaUnits: number, estimatedCostUsd: number): TlantiCaseRemoteJob {
  const now = new Date().toISOString()
  return {
    id: crypto.randomUUID(),
    label,
    target,
    status: 'queued',
    encryptedTransport: true,
    syncAssets: true,
    retryCount: 0,
    maxRetries: 3,
    quotaUnits,
    estimatedCostUsd,
    createdAt: now,
    updatedAt: now,
    lastError: null,
  }
}

export function updateRemoteJobStatus(job: TlantiCaseRemoteJob, status: TlantiCaseRemoteJobStatus, lastError?: string | null): TlantiCaseRemoteJob {
  const retryCount = status === 'retrying' ? job.retryCount + 1 : job.retryCount
  return {
    ...job,
    status,
    retryCount,
    updatedAt: new Date().toISOString(),
    lastError: lastError ?? job.lastError ?? null,
  }
}

export function buildHybridOpsReadinessReport(input: {
  activeCase: TlantiCase
  state: TlantiDbState
  operations: TlantiCaseOperationsState
  runtimeReport?: SystemRuntimeReport | null
  backendCatalog?: BackendCatalog | null
  diagnostics?: RuntimeDiagnostics | null
}): HybridOpsReadinessReport {
  const { activeCase, operations, runtimeReport, backendCatalog, diagnostics } = input
  const jobs = operations.remoteJobs
  const queueSummary = {
    total: jobs.length,
    active: jobs.filter((job) => job.status === 'queued' || job.status === 'running' || job.status === 'retrying').length,
    failed: jobs.filter((job) => job.status === 'failed').length,
    offlineFallback: jobs.filter((job) => job.status === 'offline-fallback').length,
    quotaUnits: jobs.reduce((sum, job) => sum + job.quotaUnits, 0),
    estimatedCostUsd: Math.round(jobs.reduce((sum, job) => sum + job.estimatedCostUsd, 0) * 100) / 100,
  }

  const readiness = {
    persistedCaseFolder: Boolean(activeCase.storagePath),
    assetsAttached: Boolean(activeCase.assets?.length),
    transportEncryptionReady: jobs.every((job) => job.encryptedTransport),
    syncReady: jobs.every((job) => job.syncAssets) && Boolean(activeCase.storagePath),
    runtimeLoaded: Boolean(runtimeReport),
    backendLoaded: Boolean(backendCatalog),
    offlineFallbackEnabled: operations.kernelTransition.offlineFallback,
  }

  const blockers = [
    !readiness.persistedCaseFolder ? 'The active case is not saved to a `.tlanticad` folder yet.' : null,
    !readiness.assetsAttached ? 'Attach assets before queuing hybrid/cloud workloads.' : null,
    !readiness.runtimeLoaded ? 'No runtime report loaded for workload sizing.' : null,
    !readiness.backendLoaded ? 'Backend catalog is missing, so distributed capability evidence is incomplete.' : null,
  ].filter(Boolean) as string[]

  const recommendations = [
    queueSummary.failed > 0 ? 'Retry failed jobs or send them to offline fallback before release.' : 'No failed jobs pending remediation.',
    queueSummary.quotaUnits > 60 ? 'Quota usage is high; review cost and concurrency before enabling cloud-heavy flows.' : 'Quota usage is still within a safe exploratory range.',
    diagnostics ? `Diagnostics captured on ${diagnostics.platform} / ${diagnostics.architecture}.` : 'Capture diagnostics before escalating hybrid incidents.',
  ]

  return {
    kind: 'hybrid-ops-readiness',
    sprintRange: '141-150',
    generatedAt: new Date().toISOString(),
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    caseName: activeCase.name,
    queueSummary,
    readiness,
    blockers,
    recommendations,
    jobs,
  }
}

export function buildPrecisionKernelAlphaReport(input: {
  activeCase: TlantiCase
  operations: TlantiCaseOperationsState
  backendCatalog?: BackendCatalog | null
  workspaceCatalog?: BackendWorkspaceCatalog | null
}): PrecisionKernelAlphaReport {
  const { activeCase, operations, backendCatalog, workspaceCatalog } = input
  const integrationNotes = (backendCatalog?.integrations ?? []).map((item) => `${item.crateName} · ${item.notes}`)
  const crateNames = (workspaceCatalog?.crates ?? []).map((item) => item.packageName.toLowerCase())
  const meshKernelReady = (backendCatalog?.integrations ?? []).some((item) => item.id === 'mesh-csg' && item.enabled)
  const occtEvidencePresent = crateNames.some((name) => name.includes('occt') || name.includes('cascade'))
  const conversionReadiness = meshKernelReady && Boolean(activeCase.assets?.some((asset) => asset.category === 'model'))
  const toleranceReadiness = Boolean(activeCase.pipeline?.design || activeCase.pipeline?.model)
  const benchmarkScore = operations.kernelTransition.geometryBenchmarkScore ?? (meshKernelReady ? 64 : 28) + (occtEvidencePresent ? 18 : 0) + (conversionReadiness ? 10 : 0)

  const evidence = [
    meshKernelReady ? 'Mesh CSG backend integration is available.' : 'Mesh CSG backend integration is still missing.',
    occtEvidencePresent ? 'OCCT/OpenCascade evidence was found in the workspace.' : 'No OCCT/OpenCascade crate evidence was found yet.',
    ...integrationNotes.slice(0, 4),
  ]

  const constraints = [
    !occtEvidencePresent ? 'Precision kernel remains a readiness layer until OCCT integration lands.' : null,
    !conversionReadiness ? 'Attach mesh/model assets to exercise mesh↔precision transitions.' : null,
    !toleranceReadiness ? 'The case has not yet progressed far enough in design/model stages to validate tolerances.' : null,
  ].filter(Boolean) as string[]

  const transitionNotes = [
    `Policy: ${operations.kernelTransition.preference}`,
    `Preferred kernel: ${operations.kernelTransition.preferredKernel}`,
    operations.kernelTransition.offlineFallback ? 'Offline fallback remains enabled for hybrid/precision flows.' : 'Offline fallback is disabled and should be reviewed carefully.',
  ]

  return {
    kind: 'precision-kernel-alpha',
    sprintRange: '151-160',
    generatedAt: new Date().toISOString(),
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    preference: operations.kernelTransition.preference,
    preferredKernel: operations.kernelTransition.preferredKernel,
    kernelStatus: {
      meshKernelReady,
      occtEvidencePresent,
      conversionReadiness,
      toleranceReadiness,
      benchmarkScore,
    },
    evidence,
    constraints,
    transitionNotes,
  }
}

export function exportHybridOpsReadiness(report: HybridOpsReadinessReport) {
  downloadJsonBrief(`${report.caseNumber}-hybrid-ops-141-150.json`, report)
}

export function exportPrecisionKernelAlpha(report: PrecisionKernelAlphaReport) {
  downloadJsonBrief(`${report.caseNumber}-precision-kernel-151-160.json`, report)
}
import type { DentalAiWorkflowHint } from '@/lib/dental-ai-workflows'
import { downloadJsonBrief } from '@/lib/clinical-module-briefs'
import type { DentalModelSegStatus } from '@/lib/dental-model-seg'
import type { FileData } from '@/types'

export interface SegmentationAuditPayload {
  kind: 'ai-segmentation-audit'
  fileId: string
  fileName: string
  fileType: FileData['type']
  sourcePath?: string
  sessionId?: string
  workflowId?: string
  workflowLabel?: string
  recommendedTool?: string
  executableInApp?: boolean
  numberingSystem?: 'FDI' | 'UNIVERSAL'
  status: NonNullable<FileData['aiSegmentation']>['status']
  inputKind: 'dicom' | 'mesh' | 'unknown'
  runtime: {
    ready: boolean
    sidecarRunning: boolean
    pythonPath?: string
    missing: string[]
    notes: string[]
  }
  outputPath?: string
  lastRunAt?: string
  logs?: string
  error?: string
  createdAt: string
}

function inferInputKind(file: FileData): SegmentationAuditPayload['inputKind'] {
  if (file.type === 'DICOM') return 'dicom'
  if (file.type === 'MODEL') return 'mesh'
  return 'unknown'
}

export function buildAiSegmentationAudit(file: FileData | null | undefined, workflow: DentalAiWorkflowHint | null, runtime: DentalModelSegStatus | null): SegmentationAuditPayload | null {
  if (!file) {
    return null
  }

  if (!file.aiSegmentation) {
    return null
  }

  return {
    kind: 'ai-segmentation-audit',
    fileId: file.id,
    fileName: file.name,
    fileType: file.type,
    sourcePath: file.sourcePath,
    sessionId: file.aiSegmentation.sessionId,
    workflowId: file.aiSegmentation.workflowId ?? workflow?.id,
    workflowLabel: workflow?.label,
    recommendedTool: file.aiSegmentation.recommendedTool ?? workflow?.recommendedTool,
    executableInApp: workflow?.executableInApp,
    numberingSystem: file.aiSegmentation.numberingSystem,
    status: file.aiSegmentation.status,
    inputKind: inferInputKind(file),
    runtime: {
      ready: Boolean(runtime?.ready),
      sidecarRunning: Boolean(runtime?.sidecarRunning),
      pythonPath: runtime?.pythonPath,
      missing: runtime?.missing ?? [],
      notes: runtime?.notes ?? [],
    },
    outputPath: file.aiSegmentation.outputPath,
    lastRunAt: file.aiSegmentation.lastRunAt,
    logs: file.aiSegmentation.logs,
    error: file.aiSegmentation.error,
    createdAt: new Date().toISOString(),
  }
}

export function exportAiSegmentationAudit(file: FileData, workflow: DentalAiWorkflowHint | null, runtime: DentalModelSegStatus | null) {
  const audit = buildAiSegmentationAudit(file, workflow, runtime)
  if (!audit) {
    return null
  }

  downloadJsonBrief(`${file.name.replace(/\.[^.]+$/, '')}-ai-audit.json`, audit)
  return audit
}

export function buildAiSegmentationSummary(audit: SegmentationAuditPayload) {
  return [
    `workflow=${audit.workflowId ?? 'unknown'}`,
    `status=${audit.status}`,
    `tool=${audit.recommendedTool ?? 'manual-review'}`,
    `input=${audit.inputKind}`,
    `output=${audit.outputPath ?? 'pending'}`,
    `runtime=${audit.runtime.ready ? 'ready' : 'not-ready'}`,
  ].join(' · ')
}

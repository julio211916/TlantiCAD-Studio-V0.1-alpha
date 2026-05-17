import {
  TLANTI_MODULE_DEFINITIONS,
  type TlantiModuleStage,
  resolveTlantiModuleDefinition,
} from '@/core'
import type { TlantiCasePipeline } from '@/stores/tlantidb-case-store'

export type WorkspaceModuleStage = TlantiModuleStage
export type WorkspaceStatusTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger' | 'accent'

export interface WorkspaceModuleDefinition {
  id: string
  label: string
  description: string
  stage: WorkspaceModuleStage
}

export interface WorkspaceStatusItem {
  id: string
  label: string
  tone: WorkspaceStatusTone
}

export const WORKSPACE_MODULE_DEFINITIONS: Record<string, WorkspaceModuleDefinition> = Object.fromEntries(
  Object.values(TLANTI_MODULE_DEFINITIONS).map((module) => [
    module.id,
    {
      id: module.id,
      label: module.label,
      description: module.description,
      stage: module.stage,
    },
  ]),
)

const FALLBACK_MODULE: WorkspaceModuleDefinition = {
  id: 'cad',
  label: 'CAD Design',
  description: 'Workspace dental principal para diseño clínico y fabricación.',
  stage: 'design',
}

export function resolveWorkspaceModuleDefinition(moduleId?: string | null): WorkspaceModuleDefinition {
  if (!moduleId) return FALLBACK_MODULE
  const module = resolveTlantiModuleDefinition(moduleId)
  if (module.id !== 'cad' || moduleId === 'cad') {
    return {
      id: module.id,
      label: module.label,
      description: module.description,
      stage: module.stage,
    }
  }

  return {
    id: moduleId,
    label: moduleId.replace(/-/g, ' '),
    description: 'Módulo clínico especializado dentro del shell de TlantiCAD.',
    stage: 'design',
  }
}

export function getPipelineStatusMeta(pipeline?: TlantiCasePipeline | null): Pick<WorkspaceStatusItem, 'label' | 'tone'> {
  if (!pipeline) return { label: 'Pending scan', tone: 'danger' }
  if (pipeline.export) return { label: 'Export ready', tone: 'success' }
  if (pipeline.manufacture) return { label: 'Manufacture ready', tone: 'accent' }
  if (pipeline.design || pipeline.model) return { label: 'In design', tone: 'info' }
  if (pipeline.scan) return { label: 'Scan ready', tone: 'warning' }
  return { label: 'Pending scan', tone: 'danger' }
}

export function buildCaseStatusItems({
  moduleId,
  pipeline,
  storagePath,
  interopXmlPath,
}: {
  moduleId?: string | null
  pipeline?: TlantiCasePipeline | null
  /** Kept for future diagnostics; no longer surfaced as a pill. */
  storagePath?: string | null
  /** Kept for future diagnostics; no longer surfaced as a pill. */
  interopXmlPath?: string | null
}): WorkspaceStatusItem[] {
  const module = resolveWorkspaceModuleDefinition(moduleId)
  const pipelineStatus = getPipelineStatusMeta(pipeline)

  // Two pills only — the module and the pipeline stage. "Case not linked"
  // and "XML pending" used to show here but they confused clinicians who
  // don't think in terms of interop XML; file-system linkage is a
  // developer concern.
  void storagePath
  void interopXmlPath

  return [
    {
      id: 'module',
      label: module.label,
      tone: 'info',
    },
    {
      id: 'pipeline',
      label: pipelineStatus.label,
      tone: pipelineStatus.tone,
    },
  ]
}

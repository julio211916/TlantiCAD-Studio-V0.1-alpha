import type { ModuleWorkflowState, TlantiModuleId } from '../domain/entities'
import {
  FILE_ACTION_DEFINITIONS,
  TLANTI_MODULE_DEFINITIONS,
  TLANTI_MODULE_IDS,
  isTlantiModuleId,
  resolveTlantiModuleDefinition,
} from '../domain/module-registry'

interface CreateModuleWorkflowStateInput {
  caseId: string
  moduleId: TlantiModuleId
  now?: Date
}

export function createModuleWorkflowState({
  caseId,
  moduleId,
  now = new Date(),
}: CreateModuleWorkflowStateInput): ModuleWorkflowState {
  return {
    caseId,
    moduleId,
    currentStepIndex: 0,
    completedSteps: [],
    updatedAt: now.toISOString(),
  }
}

export function completeCurrentWorkflowStep(state: ModuleWorkflowState, now = new Date()): ModuleWorkflowState {
  const definition = resolveTlantiModuleDefinition(state.moduleId)
  const currentStep = definition.workflow[state.currentStepIndex]

  if (!currentStep) {
    return { ...state, updatedAt: now.toISOString() }
  }

  const completedSteps = state.completedSteps.includes(currentStep)
    ? state.completedSteps
    : [...state.completedSteps, currentStep]

  return {
    ...state,
    completedSteps,
    currentStepIndex: Math.min(state.currentStepIndex + 1, Math.max(definition.workflow.length - 1, 0)),
    updatedAt: now.toISOString(),
  }
}

export function getModuleWorkflowProgress(state: ModuleWorkflowState): number {
  const definition = resolveTlantiModuleDefinition(state.moduleId)
  if (definition.workflow.length === 0) return 1
  return state.completedSteps.length / definition.workflow.length
}

export function validateTlantiModuleRegistry(): string[] {
  const issues: string[] = []
  const ids = new Set(TLANTI_MODULE_IDS)

  if (ids.has('ai-design' as TlantiModuleId)) {
    issues.push('ai-design must not be a root clinical module; AI tools live inside CAD Design.')
  }

  if (TLANTI_MODULE_DEFINITIONS.dicom.label !== 'DICOM Viewer') {
    issues.push('dicom module must be labeled DICOM Viewer.')
  }

  if (TLANTI_MODULE_DEFINITIONS.partials.label !== 'PartialCAD AI') {
    issues.push('partials module must be labeled PartialCAD AI.')
  }

  Object.keys(FILE_ACTION_DEFINITIONS).forEach((actionId) => {
    if (isTlantiModuleId(actionId)) {
      issues.push(`${actionId} is a file action and must not be registered as a clinical module.`)
    }
  })

  TLANTI_MODULE_IDS.forEach((moduleId) => {
    const definition = TLANTI_MODULE_DEFINITIONS[moduleId]
    const workflow = definition.workflow as readonly string[]
    const tools = definition.tools as readonly string[]
    if (workflow.length < 4) {
      issues.push(`${moduleId} needs a complete operator workflow.`)
    }
    if (tools.length === 0) {
      issues.push(`${moduleId} needs at least one tool.`)
    }
  })

  return issues
}

import type { ModuleWorkflowState } from '../domain/entities'

export interface ModuleWorkflowRepository {
  find(caseId: string, moduleId: string): Promise<ModuleWorkflowState | null>
  save(state: ModuleWorkflowState): Promise<ModuleWorkflowState>
}

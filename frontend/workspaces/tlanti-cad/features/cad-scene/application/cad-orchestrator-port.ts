import type { CadShellBootstrap } from '../domain/cad-scene';

export interface CadOrchestratorPort {
  bootstrapShell(input: { caseId?: string; moduleId?: string }): Promise<CadShellBootstrap>;
}

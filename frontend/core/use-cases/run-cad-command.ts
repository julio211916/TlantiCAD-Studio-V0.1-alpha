import type { CadCommand, CadCommandRunResult } from '../domain/cad-command'
import { runCadCommandInMemory, validateCadCommand } from '../domain/cad-command'

export interface CadCommandRunnerPort {
  run(command: CadCommand): Promise<CadCommandRunResult>
}

export class InMemoryCadCommandRunner implements CadCommandRunnerPort {
  async run(command: CadCommand): Promise<CadCommandRunResult> {
    return runCadCommandInMemory(command)
  }
}

export class CadCommandUseCase {
  constructor(private readonly runner: CadCommandRunnerPort = new InMemoryCadCommandRunner()) {}

  async run(command: CadCommand): Promise<CadCommandRunResult> {
    const validation = validateCadCommand(command)
    if (!validation.ok) {
      return { accepted: false, command, issues: validation.issues }
    }

    return this.runner.run(command)
  }
}

export function createCadCommandUseCase(runner?: CadCommandRunnerPort): CadCommandUseCase {
  return new CadCommandUseCase(runner)
}

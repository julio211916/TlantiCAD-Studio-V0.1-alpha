import { PersistibleClinicalToolUseCase } from '../use-cases/cad-clinical-tool-use-case'
import { TauriClinicalJobRepository } from './tauri-clinical-job-repository'

export function createTauriPersistibleClinicalToolUseCase(): PersistibleClinicalToolUseCase {
  return new PersistibleClinicalToolUseCase(new TauriClinicalJobRepository())
}

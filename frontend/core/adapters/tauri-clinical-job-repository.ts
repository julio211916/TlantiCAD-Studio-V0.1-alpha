import type {
  ClinicalArtifactRecord,
  ClinicalArtifactRecordRequest,
  ClinicalJobRecord,
  ClinicalJobRecordRequest,
  ClinicalJobRepository,
} from '../ports/clinical-job-repository'

type InvokeArgs = Record<string, unknown>

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

export class TauriClinicalJobRepository implements ClinicalJobRepository {
  async record(request: ClinicalJobRecordRequest): Promise<ClinicalJobRecord> {
    return invokeCommand<ClinicalJobRecord>('clinical_job_record', { request })
  }

  async get(jobId: string): Promise<ClinicalJobRecord> {
    return invokeCommand<ClinicalJobRecord>('clinical_job_get', { jobId })
  }

  async list(caseId?: string | null): Promise<readonly ClinicalJobRecord[]> {
    return invokeCommand<ClinicalJobRecord[]>('clinical_job_list', { caseId: caseId ?? null })
  }

  async cancel(jobId: string): Promise<ClinicalJobRecord> {
    return invokeCommand<ClinicalJobRecord>('clinical_job_cancel', { jobId })
  }

  async recordArtifact(request: ClinicalArtifactRecordRequest): Promise<ClinicalArtifactRecord> {
    return invokeCommand<ClinicalArtifactRecord>('clinical_artifact_record', { request })
  }
}

import type { ClinicalAsset } from '../domain/entities'

export type JawMovement = 'protrusive' | 'right' | 'left' | 'linear_drop'

export interface ClinicalJob {
  id: string
  caseId: string
  kind: 'jaw-motion'
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'
  progress: number
  createdAt: string
  completedAt?: string | null
  resultId?: string | null
  error?: string | null
}

export interface JawMotionGenerateRequest {
  caseId: string
  maxillaAssetId: string
  mandibleAssetId: string
  marks: Record<string, readonly number[]>
  movement: JawMovement
  frames?: number
  clearanceMm?: number
  maxillaPath?: string | null
  mandiblePath?: string | null
  movementDistanceMm?: number
  samples?: number
  contactIterations?: number
  outputXmlPath?: string | null
}

export interface JawMotionResult {
  id: string
  caseId: string
  movement: JawMovement
  frameCount: number
  transforms: readonly (readonly (readonly number[])[])[]
  tracks: Record<string, readonly (readonly number[])[]>
  totalDropMm: number
  warnings: readonly string[]
  outputXmlPath?: string | null
}

export interface JawMotionRepository {
  generate(request: JawMotionGenerateRequest): Promise<ClinicalJob>
  getResult(jobId: string): Promise<JawMotionResult | null>
  exportXml(resultId: string): Promise<ClinicalAsset>
}

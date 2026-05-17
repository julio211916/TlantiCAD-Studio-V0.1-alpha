import type { ClinicalAsset } from '../domain/entities'
import type {
  ClinicalJob,
  JawMotionGenerateRequest,
  JawMotionRepository,
  JawMotionResult,
} from '../ports/jaw-motion-repository'

const DEFAULT_SIDECAR_URL = 'http://127.0.0.1:17493'

interface RawJawMotionResult {
  case_id: string
  movement: JawMotionResult['movement']
  frame_count: number
  transforms: number[][][]
  tracks: Record<string, number[][]>
  total_drop_mm: number
  warnings: string[]
  output_xml_path?: string | null
}

function toRawRequest(request: JawMotionGenerateRequest) {
  return {
    caseId: request.caseId,
    maxillaAssetId: request.maxillaAssetId,
    mandibleAssetId: request.mandibleAssetId,
    marks: request.marks,
    movement: request.movement,
    frames: request.frames,
    clearanceMm: request.clearanceMm,
    maxillaPath: request.maxillaPath,
    mandiblePath: request.mandiblePath,
    movementDistanceMm: request.movementDistanceMm,
    samples: request.samples,
    contactIterations: request.contactIterations,
    outputXmlPath: request.outputXmlPath,
  }
}

function toJawMotionResult(resultId: string, raw: RawJawMotionResult): JawMotionResult {
  return {
    id: resultId,
    caseId: raw.case_id,
    movement: raw.movement,
    frameCount: raw.frame_count,
    transforms: raw.transforms,
    tracks: raw.tracks,
    totalDropMm: raw.total_drop_mm,
    warnings: raw.warnings,
    outputXmlPath: raw.output_xml_path,
  }
}

async function parseSidecarError(response: Response): Promise<string> {
  const payload = await response.json().catch(() => null)
  if (typeof payload?.detail === 'string') return payload.detail
  return `JawMotion sidecar HTTP ${response.status}`
}

export class PythonJawMotionRepository implements JawMotionRepository {
  private readonly baseUrl: string
  private readonly resultsByJobId = new Map<string, JawMotionResult>()
  private readonly resultsById = new Map<string, JawMotionResult>()

  constructor(baseUrl = DEFAULT_SIDECAR_URL) {
    this.baseUrl = baseUrl.replace(/\/$/, '')
  }

  async generate(request: JawMotionGenerateRequest): Promise<ClinicalJob> {
    const createdAt = new Date().toISOString()
    const response = await fetch(`${this.baseUrl}/jaw-motion/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(toRawRequest(request)),
    })

    if (!response.ok) {
      throw new Error(await parseSidecarError(response))
    }

    const payload = (await response.json()) as { result: RawJawMotionResult }
    const jobId = `jaw-motion:${request.caseId}:${createdAt}`
    const resultId = `${jobId}:result`
    const result = toJawMotionResult(resultId, payload.result)
    this.resultsByJobId.set(jobId, result)
    this.resultsById.set(resultId, result)

    return {
      id: jobId,
      caseId: request.caseId,
      kind: 'jaw-motion',
      status: 'completed',
      progress: 1,
      createdAt,
      completedAt: new Date().toISOString(),
      resultId,
    }
  }

  async getResult(jobId: string): Promise<JawMotionResult | null> {
    return this.resultsByJobId.get(jobId) ?? null
  }

  async exportXml(resultId: string): Promise<ClinicalAsset> {
    const result = this.resultsById.get(resultId)
    if (!result?.outputXmlPath) {
      throw new Error('JawMotion XML export requires outputXmlPath during generation until Tauri asset export is wired.')
    }

    return {
      id: resultId,
      name: `jaw-motion-${result.caseId}-${result.movement}.xml`,
      role: 'report',
      localPath: result.outputXmlPath,
      moduleId: 'cad',
      tags: ['jaw-motion', 'exocad-xml'],
      createdAt: new Date().toISOString(),
    }
  }
}

import {
  CAD_PRODUCT_MODULE_DEFINITIONS,
  TLANTI_CAD_PRODUCT_MODULE_IDS,
  type TlantiCadProductModuleId,
} from '../domain/cad-product-module-registry'
import type {
  ClinicalJobRecord,
  ClinicalJobRecordRequest,
  ClinicalJobRepository,
  ClinicalJobStatus,
} from '../ports/clinical-job-repository'

export type PersistibleClinicalToolId =
  | 'meshlib-repair'
  | 'meshlib-boolean'
  | 'meshlib-offset'
  | 'dentist-sota-dicom-sanitize'
  | 'dentist-sota-pxi-segmentation'
  | 'dentist-sota-cbct-segmentation'
  | 'jaw-motion-generate'
  | 'clinical-command-record'
  | 'clinical-command-undo'
  | 'clinical-command-redo'

export type ClinicalToolRuntime = 'tauri-rust' | 'python-sidecar'

export interface PersistibleClinicalToolDefinition {
  id: PersistibleClinicalToolId
  label: string
  runtime: ClinicalToolRuntime
  vendor: 'meshlib' | 'dentist-sota' | 'jawmotionai' | 'tlanticad-core'
  jobKind: string
  modules: readonly TlantiCadProductModuleId[]
  inputAssets: readonly string[]
  outputArtifacts: readonly string[]
  paramsSchemaVersion: number
  persistenceRule: string
  webviewRule: string
  wasmRule: string
  defaultStatus: ClinicalJobStatus
}

const ALL_PRODUCT_MODULES: readonly TlantiCadProductModuleId[] = TLANTI_CAD_PRODUCT_MODULE_IDS

export const PERSISTIBLE_CLINICAL_TOOL_DEFINITIONS: readonly PersistibleClinicalToolDefinition[] = [
  {
    id: 'meshlib-repair',
    label: 'MeshLib repair',
    runtime: 'tauri-rust',
    vendor: 'meshlib',
    jobKind: 'mesh-repair',
    modules: ['tlanticad-crown', 'tlanticad-freeform', 'tlanticad-model'],
    inputAssets: ['mesh'],
    outputArtifacts: ['derived-mesh', 'mesh-manifest'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist source asset id, repair params, derived asset checksum, size, parent asset and tool version.',
    webviewRule: 'WebView may dispatch DTOs and show progress only; it must not scan folders or mutate STL files directly.',
    wasmRule: 'WASM may run lightweight preview checks, but persisted mesh repair is owned by the Tauri/Rust job.',
    defaultStatus: 'queued',
  },
  {
    id: 'meshlib-boolean',
    label: 'MeshLib boolean',
    runtime: 'tauri-rust',
    vendor: 'meshlib',
    jobKind: 'mesh-boolean',
    modules: ['tlanticad-bridge', 'tlanticad-freeform'],
    inputAssets: ['mesh-a', 'mesh-b'],
    outputArtifacts: ['derived-mesh', 'boolean-report'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist operands, boolean mode, derived mesh checksum, parent ids and topology validation report.',
    webviewRule: 'WebView sends operands and mode; Three.js only previews selected meshes.',
    wasmRule: 'WASM is allowed for temporary bounding-box/preview evaluation, not final clinical boolean output.',
    defaultStatus: 'queued',
  },
  {
    id: 'meshlib-offset',
    label: 'MeshLib offset',
    runtime: 'tauri-rust',
    vendor: 'meshlib',
    jobKind: 'mesh-offset',
    modules: ['tlanticad-abutment', 'tlanticad-bar', 'tlanticad-bite-splint'],
    inputAssets: ['mesh'],
    outputArtifacts: ['offset-mesh', 'thickness-report'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist offset distance, shell options, derived asset checksum and thickness validation metadata.',
    webviewRule: 'WebView edits numeric parameters and displays preview state only.',
    wasmRule: 'WASM may calculate coarse preview shells; persisted offset belongs to Tauri/Rust.',
    defaultStatus: 'queued',
  },
  {
    id: 'dentist-sota-dicom-sanitize',
    label: 'Dentist-SOTA DICOM sanitize',
    runtime: 'python-sidecar',
    vendor: 'dentist-sota',
    jobKind: 'dicom-sanitize',
    modules: ['tlanticad-implant'],
    inputAssets: ['dicom-series'],
    outputArtifacts: ['sanitized-dicom-series', 'dicom-metadata-json'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist sanitized series manifest, metadata JSON checksum and PHI removal summary.',
    webviewRule: 'WebView never reads DICOM files directly; it asks Tauri/Python for metadata and previews.',
    wasmRule: 'WASM is not used for PHI handling; DICOM privacy work stays in the Python sidecar.',
    defaultStatus: 'queued',
  },
  {
    id: 'dentist-sota-pxi-segmentation',
    label: 'Dentist-SOTA PXI segmentation',
    runtime: 'python-sidecar',
    vendor: 'dentist-sota',
    jobKind: 'dicom-segmentation',
    modules: ['tlanticad-implant'],
    inputAssets: ['panoramic-xray'],
    outputArtifacts: ['segmentation-mask', 'segmentation-review-json'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist model id, checkpoint checksum, inference params, mask checksum and manual review state.',
    webviewRule: 'WebView displays result overlays and review controls only; inference runs in the local Python job.',
    wasmRule: 'WASM may render mask compositing in the WebView, but model inference remains offline Python/ONNX/Torch.',
    defaultStatus: 'manual-review',
  },
  {
    id: 'dentist-sota-cbct-segmentation',
    label: 'Dentist-SOTA CBCT segmentation',
    runtime: 'python-sidecar',
    vendor: 'dentist-sota',
    jobKind: 'dicom-segmentation',
    modules: ['tlanticad-implant'],
    inputAssets: ['cbct-series'],
    outputArtifacts: ['segmentation-volume', 'mesh-proposal', 'segmentation-review-json'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist model id, checkpoint checksum, volume manifest, mesh proposal checksum and review state.',
    webviewRule: 'WebView renders slices/overlays; it must not parse volume data or run segmentation.',
    wasmRule: 'WASM may handle viewport compositing and slice shaders, not final segmentation.',
    defaultStatus: 'manual-review',
  },
  {
    id: 'jaw-motion-generate',
    label: 'JawMotionAI motion generation',
    runtime: 'python-sidecar',
    vendor: 'jawmotionai',
    jobKind: 'jaw-motion',
    modules: ['tlanticad-bite-splint'],
    inputAssets: ['maxilla-scan', 'mandible-scan', 'marks-json'],
    outputArtifacts: ['jaw-motion-result-json', 'exocad-motion-xml'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist marks, movement params, generated transforms, warnings, result JSON and XML checksums.',
    webviewRule: 'WebView collects marks and previews tracks; collision/proximity work stays in Python.',
    wasmRule: 'WASM may interpolate preview tracks only after a persisted job result exists.',
    defaultStatus: 'queued',
  },
  {
    id: 'clinical-command-record',
    label: 'Clinical command record',
    runtime: 'tauri-rust',
    vendor: 'tlanticad-core',
    jobKind: 'clinical-command-record',
    modules: ALL_PRODUCT_MODULES,
    inputAssets: ['command-dto'],
    outputArtifacts: ['clinical-command-event'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist command DTO, target aggregate, user intent, inverse command pointer and affected asset ids.',
    webviewRule: 'WebView emits intent DTOs; command validation and persistence happen through Tauri.',
    wasmRule: 'WASM must not own undo history; it can only supply preview measurements included in command DTOs.',
    defaultStatus: 'queued',
  },
  {
    id: 'clinical-command-undo',
    label: 'Clinical command undo',
    runtime: 'tauri-rust',
    vendor: 'tlanticad-core',
    jobKind: 'clinical-command-undo',
    modules: ALL_PRODUCT_MODULES,
    inputAssets: ['command-event-id'],
    outputArtifacts: ['clinical-command-event'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist undo event linked to the original command and reconstitute state from the command log.',
    webviewRule: 'WebView requests undo; it does not mutate clinical state locally.',
    wasmRule: 'WASM is read-only for undo/redo and only refreshes preview buffers after committed state changes.',
    defaultStatus: 'queued',
  },
  {
    id: 'clinical-command-redo',
    label: 'Clinical command redo',
    runtime: 'tauri-rust',
    vendor: 'tlanticad-core',
    jobKind: 'clinical-command-redo',
    modules: ALL_PRODUCT_MODULES,
    inputAssets: ['command-event-id'],
    outputArtifacts: ['clinical-command-event'],
    paramsSchemaVersion: 1,
    persistenceRule: 'Persist redo event linked to the undo/original command pair and replay via the clinical command log.',
    webviewRule: 'WebView requests redo; it does not reconstruct clinical state from React snapshots.',
    wasmRule: 'WASM receives committed state deltas after redo; it never stores the clinical source of truth.',
    defaultStatus: 'queued',
  },
]

const TOOL_DEFINITIONS_BY_ID = new Map<PersistibleClinicalToolId, PersistibleClinicalToolDefinition>(
  PERSISTIBLE_CLINICAL_TOOL_DEFINITIONS.map((tool) => [tool.id, tool]),
)

export interface RunPersistibleClinicalToolRequest {
  caseId?: string | null
  moduleId: TlantiCadProductModuleId
  toolId: PersistibleClinicalToolId
  assetIds?: readonly string[]
  params?: Record<string, unknown>
  modelId?: string | null
  checkpointSha256?: string | null
  jobId?: string
}

export interface PersistibleClinicalToolJobParams {
  schemaVersion: number
  moduleId: TlantiCadProductModuleId
  toolId: PersistibleClinicalToolId
  assetIds: readonly string[]
  params: Record<string, unknown>
  runtime: ClinicalToolRuntime
  persistenceRule: string
  webviewRule: string
  wasmRule: string
}

export function resolvePersistibleClinicalToolDefinition(
  toolId: PersistibleClinicalToolId,
): PersistibleClinicalToolDefinition {
  const tool = TOOL_DEFINITIONS_BY_ID.get(toolId)
  if (!tool) {
    throw new Error(`Unknown persistible clinical tool: ${toolId}`)
  }

  return tool
}

export function listPersistibleClinicalToolsForModule(
  moduleId: TlantiCadProductModuleId,
): readonly PersistibleClinicalToolDefinition[] {
  return PERSISTIBLE_CLINICAL_TOOL_DEFINITIONS.filter((tool) => tool.modules.includes(moduleId))
}

export function buildPersistibleClinicalToolJobRequest(
  request: RunPersistibleClinicalToolRequest,
): ClinicalJobRecordRequest {
  const productModule = CAD_PRODUCT_MODULE_DEFINITIONS[request.moduleId]
  const tool = resolvePersistibleClinicalToolDefinition(request.toolId)

  if (!tool.modules.includes(request.moduleId)) {
    throw new Error(`${tool.id} is not available in ${request.moduleId}`)
  }

  const isClinicalCommand = tool.jobKind.startsWith('clinical-command-')
  if (!isClinicalCommand && !(productModule.jobTypes as readonly string[]).includes(tool.jobKind)) {
    throw new Error(`${tool.jobKind} is not registered for ${request.moduleId}`)
  }

  const jobParams: PersistibleClinicalToolJobParams = {
    schemaVersion: tool.paramsSchemaVersion,
    moduleId: request.moduleId,
    toolId: tool.id,
    assetIds: request.assetIds ?? [],
    params: request.params ?? {},
    runtime: tool.runtime,
    persistenceRule: tool.persistenceRule,
    webviewRule: tool.webviewRule,
    wasmRule: tool.wasmRule,
  }

  return {
    id: request.jobId,
    caseId: request.caseId,
    kind: tool.jobKind,
    status: tool.defaultStatus,
    progress: 0,
    vendor: tool.vendor,
    modelId: request.modelId,
    checkpointSha256: request.checkpointSha256,
    paramsJson: JSON.stringify(jobParams),
  }
}

export class PersistibleClinicalToolUseCase {
  constructor(private readonly clinicalJobs: ClinicalJobRepository) {}

  async execute(request: RunPersistibleClinicalToolRequest): Promise<ClinicalJobRecord> {
    return this.clinicalJobs.record(buildPersistibleClinicalToolJobRequest(request))
  }
}

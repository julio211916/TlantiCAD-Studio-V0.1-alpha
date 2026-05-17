import type { ModuleSurfaceId } from './module-surface-registry'

export type WorkflowStepId = 'import' | 'clean' | 'segment' | 'design' | 'validate' | 'export'

export type WorkflowStepOwner = 'react-ui' | 'tauri-rust' | 'rust-core' | 'fastapi-python' | 'local-service'

export type WorkflowStepComputeMode = 'metadata' | 'streaming-io' | 'gpu-preview' | 'cpu-job' | 'ai-job'

export interface WorkflowStepDefinition {
  readonly id: WorkflowStepId
  readonly order: number
  readonly label: string
  readonly owner: WorkflowStepOwner
  readonly computeMode: WorkflowStepComputeMode
  readonly moduleSurfaceIds: readonly ModuleSurfaceId[]
  readonly inputArtifacts: readonly string[]
  readonly outputArtifacts: readonly string[]
  readonly tools: readonly string[]
  readonly localServices: readonly string[]
  readonly performanceContract: string
}

export type WorkflowStepRegistry = Readonly<Record<WorkflowStepId, WorkflowStepDefinition>>

export const WORKFLOW_STEP_IDS = [
  'import',
  'clean',
  'segment',
  'design',
  'validate',
  'export',
] as const satisfies readonly WorkflowStepId[]

export const WORKFLOW_STEP_REGISTRY = {
  import: {
    id: 'import',
    order: 10,
    label: 'Import',
    owner: 'tauri-rust',
    computeMode: 'streaming-io',
    moduleSurfaceIds: ['workspace-hub', 'cad', 'dicom'],
    inputArtifacts: ['source-path', 'dicom-series', 'stl-file', 'obj-file', 'ply-file', 'photo-file'],
    outputArtifacts: ['asset-manifest', 'mesh-vault-handle', 'dicom-study-index', 'source-hash'],
    tools: ['DICOM Import', 'STL Import', 'OBJ Import', 'PLY Import', 'Hash Verify'],
    localServices: ['asset-vault-service', 'mesh-vault-service', 'dicom-service'],
    performanceContract: 'Import must stream from disk and persist source hashes without loading large binaries into React memory.',
  },
  clean: {
    id: 'clean',
    order: 20,
    label: 'Clean',
    owner: 'rust-core',
    computeMode: 'cpu-job',
    moduleSurfaceIds: ['cad', 'dicom'],
    inputArtifacts: ['mesh-vault-handle', 'dicom-study-index'],
    outputArtifacts: ['clean-mesh-handle', 'normalized-dicom-index', 'repair-report'],
    tools: ['Repair Mesh', 'Decimate', 'Normalize Orientation', 'DICOM Sanitize'],
    localServices: ['cad-compute-service', 'dicom-service', 'mesh-vault-service'],
    performanceContract: 'Cleaning must run as cancellable background jobs with chunked progress and derived artifacts.',
  },
  segment: {
    id: 'segment',
    order: 30,
    label: 'Segment',
    owner: 'fastapi-python',
    computeMode: 'ai-job',
    moduleSurfaceIds: ['cad', 'dicom'],
    inputArtifacts: ['clean-mesh-handle', 'normalized-dicom-index', 'volume-slice-cache'],
    outputArtifacts: ['segmentation-mask', 'tooth-labels', 'bone-mesh-handle', 'nerve-path'],
    tools: ['Local Segment', 'Tooth Labels', 'Nerve Trace', 'Extract Mesh'],
    localServices: ['ai-local-service', 'dicom-service', 'mesh-vault-service'],
    performanceContract: 'Segmentation must use local ONNX/Torch runtimes and return handles or masks, not giant arrays in UI state.',
  },
  design: {
    id: 'design',
    order: 40,
    label: 'Design',
    owner: 'rust-core',
    computeMode: 'gpu-preview',
    moduleSurfaceIds: ['cad'],
    inputArtifacts: ['clean-mesh-handle', 'segmentation-mask', 'tooth-labels', 'case-work-definition'],
    outputArtifacts: ['construction-graph', 'designed-mesh-handle', 'tool-history'],
    tools: ['Margin', 'Axis', 'Boolean', 'Offset', 'Sculpt', 'Contacts'],
    localServices: ['cad-compute-service', 'mesh-vault-service'],
    performanceContract: 'Design previews belong in Three.js GPU buffers; accepted operations persist as command deltas and Rust artifacts.',
  },
  validate: {
    id: 'validate',
    order: 50,
    label: 'Validate',
    owner: 'rust-core',
    computeMode: 'cpu-job',
    moduleSurfaceIds: ['workspace-hub', 'cad', 'dicom', 'manufacturing', 'system'],
    inputArtifacts: ['designed-mesh-handle', 'construction-graph', 'material-profile', 'runtime-profile'],
    outputArtifacts: ['validation-report', 'thickness-map', 'collision-report', 'runtime-health-report'],
    tools: ['Mesh Validate', 'Thickness', 'Collision Check', 'Runtime Check', 'Storage Verify'],
    localServices: ['cad-compute-service', 'export-service', 'storage-health-service'],
    performanceContract: 'Validation must be incremental and job-backed so thickness/collision checks do not block the interaction thread.',
  },
  export: {
    id: 'export',
    order: 60,
    label: 'Export',
    owner: 'tauri-rust',
    computeMode: 'streaming-io',
    moduleSurfaceIds: ['workspace-hub', 'cad', 'dicom', 'manufacturing'],
    inputArtifacts: ['validated-mesh-handle', 'validation-report', 'case-manifest'],
    outputArtifacts: ['stl-export', 'obj-export', 'three-mf-export', 'manufacturing-manifest', 'audit-entry'],
    tools: ['Export STL', 'Export OBJ', 'Export 3MF', 'Manifest Export', 'Reveal Folder'],
    localServices: ['export-service', 'asset-vault-service', 'audit-service'],
    performanceContract: 'Export must write from persisted handles with provenance and avoid duplicate browser Blob copies for heavy meshes.',
  },
} as const satisfies WorkflowStepRegistry

export const WORKFLOW_STEP_DEFINITIONS = WORKFLOW_STEP_IDS.map(
  (stepId) => WORKFLOW_STEP_REGISTRY[stepId],
) as readonly WorkflowStepDefinition[]

export function listWorkflowSteps(): readonly WorkflowStepDefinition[] {
  return WORKFLOW_STEP_DEFINITIONS
}

export function isWorkflowStepId(value: string): value is WorkflowStepId {
  return Object.prototype.hasOwnProperty.call(WORKFLOW_STEP_REGISTRY, value)
}

export function resolveWorkflowStep(id?: string | null): WorkflowStepDefinition | null {
  if (!id || !isWorkflowStepId(id)) {
    return null
  }

  return WORKFLOW_STEP_REGISTRY[id]
}

export function listWorkflowStepsForSurface(surfaceId: ModuleSurfaceId): readonly WorkflowStepDefinition[] {
  return WORKFLOW_STEP_DEFINITIONS.filter((step) => step.moduleSurfaceIds.includes(surfaceId))
}

export function resolveNextWorkflowStep(stepId: WorkflowStepId): WorkflowStepDefinition | null {
  const index = WORKFLOW_STEP_IDS.indexOf(stepId)

  if (index < 0 || index + 1 >= WORKFLOW_STEP_IDS.length) {
    return null
  }

  return WORKFLOW_STEP_REGISTRY[WORKFLOW_STEP_IDS[index + 1]]
}

export function resolvePreviousWorkflowStep(stepId: WorkflowStepId): WorkflowStepDefinition | null {
  const index = WORKFLOW_STEP_IDS.indexOf(stepId)

  if (index <= 0) {
    return null
  }

  return WORKFLOW_STEP_REGISTRY[WORKFLOW_STEP_IDS[index - 1]]
}

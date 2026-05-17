import type { WorkflowStepId } from './workflow-step-registry'

export type ModuleSurfaceId = 'workspace-hub' | 'cad' | 'dicom' | 'manufacturing' | 'system'

export type ModuleSurfaceGroup =
  | 'workspace-core'
  | 'cad-core'
  | 'dicom-core'
  | 'fabrication'
  | 'system'

export type ModuleSurfaceOwner =
  | 'react-ui'
  | 'three-render'
  | 'tauri-rust'
  | 'rust-core'
  | 'fastapi-python'
  | 'local-service'

export interface ModuleSurfaceDefinition {
  readonly id: ModuleSurfaceId
  readonly label: string
  readonly group: ModuleSurfaceGroup
  readonly owner: ModuleSurfaceOwner
  readonly routeModuleIds: readonly string[]
  readonly screens: readonly string[]
  readonly tools: readonly string[]
  readonly workflowStepIds: readonly WorkflowStepId[]
  readonly localServices: readonly string[]
  readonly offlineReady: boolean
  readonly performanceContract: string
}

export type ModuleSurfaceRegistry = Readonly<Record<ModuleSurfaceId, ModuleSurfaceDefinition>>

export const MODULE_SURFACE_IDS = [
  'workspace-hub',
  'cad',
  'dicom',
  'manufacturing',
  'system',
] as const satisfies readonly ModuleSurfaceId[]

export const MODULE_SURFACE_REGISTRY = {
  'workspace-hub': {
    id: 'workspace-hub',
    label: 'Workspace Hub',
    group: 'workspace-core',
    owner: 'react-ui',
    routeModuleIds: ['hub', 'patients', 'jobs'],
    screens: ['Recent cases', 'Patients', 'Case assets', 'Jobs', 'Audit timeline'],
    tools: ['New Workspace', 'Open Workspace', 'Reveal Folder', 'Import Asset', 'Runtime Settings'],
    workflowStepIds: ['import', 'validate', 'export'],
    localServices: ['workspace-service', 'asset-vault-service', 'audit-service'],
    offlineReady: true,
    performanceContract: 'Keep hub state metadata-only; never store mesh, DICOM or preview buffers in React state.',
  },
  cad: {
    id: 'cad',
    label: 'CAD',
    group: 'cad-core',
    owner: 'rust-core',
    routeModuleIds: ['cad', 'crown', 'bridge', 'implant', 'splint', 'orthocad', 'model-creator', 'partials'],
    screens: ['CAD Workspace', 'Viewport', 'Layers', 'Wizard', 'Properties', 'Measurements'],
    tools: ['Select', 'Transform', 'Measure', 'Repair', 'Boolean', 'Offset', 'Sculpt', 'Thickness'],
    workflowStepIds: ['import', 'clean', 'segment', 'design', 'validate', 'export'],
    localServices: ['cad-compute-service', 'mesh-vault-service', 'asset-vault-service'],
    offlineReady: true,
    performanceContract: 'Run destructive mesh operations as cancellable Rust jobs and keep Three.js buffers out of global UI state.',
  },
  dicom: {
    id: 'dicom',
    label: 'DICOM',
    group: 'dicom-core',
    owner: 'fastapi-python',
    routeModuleIds: ['dicom', 'implant', 'ceph'],
    screens: ['Study browser', 'Series browser', 'MPR', 'Volume preview', 'Segmentation', 'Mesh extraction'],
    tools: ['DICOM Import', 'Window Level', 'MPR', 'Segment', 'Extract Mesh', 'Send to CAD'],
    workflowStepIds: ['import', 'clean', 'segment', 'validate', 'export'],
    localServices: ['dicom-service', 'ai-local-service', 'mesh-vault-service'],
    offlineReady: true,
    performanceContract: 'Stream DICOM metadata and slices through the local sidecar; avoid full-series browser decoding.',
  },
  manufacturing: {
    id: 'manufacturing',
    label: 'Manufacturing',
    group: 'fabrication',
    owner: 'tauri-rust',
    routeModuleIds: ['fab', 'export', 'cam'],
    screens: ['Export package', 'Mesh validation', 'Material profile', 'Manufacturing report', 'Manifest'],
    tools: ['Mesh Validate', 'Repair', 'Export STL', 'Export OBJ', 'Export 3MF', 'Manifest Export'],
    workflowStepIds: ['validate', 'export'],
    localServices: ['export-service', 'asset-vault-service', 'cad-compute-service'],
    offlineReady: true,
    performanceContract: 'Validate and export from persisted mesh handles, not duplicated browser blobs.',
  },
  system: {
    id: 'system',
    label: 'System',
    group: 'system',
    owner: 'local-service',
    routeModuleIds: ['runtime', 'settings', 'logs'],
    screens: ['Python runtime', 'GPU/CPU profile', 'Storage health', 'Permissions', 'Logs'],
    tools: ['Runtime Check', 'Storage Verify', 'Compute Benchmark', 'Open Logs', 'Repair Index'],
    workflowStepIds: ['validate'],
    localServices: ['workspace-service', 'ai-local-service', 'cad-compute-service', 'storage-health-service'],
    offlineReady: true,
    performanceContract: 'Expose runtime health from local probes and keep benchmarks asynchronous.',
  },
} as const satisfies ModuleSurfaceRegistry

export const MODULE_SURFACE_DEFINITIONS = MODULE_SURFACE_IDS.map(
  (surfaceId) => MODULE_SURFACE_REGISTRY[surfaceId],
) as readonly ModuleSurfaceDefinition[]

export function listModuleSurfaces(): readonly ModuleSurfaceDefinition[] {
  return MODULE_SURFACE_DEFINITIONS
}

export function isModuleSurfaceId(value: string): value is ModuleSurfaceId {
  return Object.prototype.hasOwnProperty.call(MODULE_SURFACE_REGISTRY, value)
}

export function resolveModuleSurface(id?: string | null): ModuleSurfaceDefinition | null {
  if (!id || !isModuleSurfaceId(id)) {
    return null
  }

  return MODULE_SURFACE_REGISTRY[id]
}

export function listModuleSurfacesForService(localService: string): readonly ModuleSurfaceDefinition[] {
  return MODULE_SURFACE_DEFINITIONS.filter((surface) => surface.localServices.includes(localService))
}

export function listModuleSurfacesForWorkflowStep(stepId: WorkflowStepId): readonly ModuleSurfaceDefinition[] {
  return MODULE_SURFACE_DEFINITIONS.filter((surface) => surface.workflowStepIds.includes(stepId))
}

export function listModuleSurfacesForRoute(routeModuleId: string): readonly ModuleSurfaceDefinition[] {
  return MODULE_SURFACE_DEFINITIONS.filter((surface) => surface.routeModuleIds.includes(routeModuleId))
}

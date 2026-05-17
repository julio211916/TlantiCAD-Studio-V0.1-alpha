import type { TlantiCadProductModuleId } from './cad-product-module-registry'
import type { TlantiModuleId, ToothNumber } from './entities'

export type CadCoreModuleId = TlantiModuleId | TlantiCadProductModuleId
export type CadCoreLayer = 'react-ui' | 'three-render' | 'tauri-command' | 'rust-core' | 'python-sidecar' | 'asset-vault'
export type CadCoreAssetKind = 'dicom-series' | 'stl-mesh' | 'obj-mesh' | 'ply-mesh' | 'texture' | 'photo' | 'mask' | 'scene-snapshot' | 'report' | 'manufacturing-export'
export type CadCoreToolCategory = 'scene' | 'scan' | 'mesh' | 'dental-design' | 'implant' | 'dicom' | 'articulator' | 'ortho' | 'ceph' | 'manufacturing'
export type WizardGuardKind = 'permission' | 'asset' | 'dependency' | 'job-complete' | 'manual-review'

export interface PatientRecord {
  id: string
  externalId?: string | null
  displayAlias: string
  firstName?: string | null
  lastName?: string | null
  dateOfBirth?: string | null
  metadata?: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface CaseRecord {
  id: string
  caseNumber: string
  patientId: string
  title: string
  activeModuleId: CadCoreModuleId
  status: 'draft' | 'intake' | 'scanning' | 'designing' | 'review' | 'manufacturing' | 'exported' | 'archived'
  createdAt: string
  updatedAt: string
}

export interface CaseFolderLayout {
  caseId: string
  root: string
  manifestPath: string
  workDefinitionPath: string
  directories: readonly string[]
}

export interface CaseManifest {
  schemaVersion: 1
  caseId: string
  caseNumber: string
  patientId: string
  activeModuleId: CadCoreModuleId
  createdAt: string
  updatedAt: string
  layout: CaseFolderLayout
  assets: readonly AssetManifest[]
  workDefinitionId: string
}

export interface AssetManifest {
  id: string
  kind: CadCoreAssetKind
  role: string
  storagePath: string
  checksumSha256?: string | null
  source: 'scanner' | 'manual-import' | 'rust-job' | 'python-job' | 'derived' | 'report'
  moduleId?: CadCoreModuleId | null
  toothNumbers?: readonly ToothNumber[]
  createdAt: string
  metadata?: Record<string, unknown>
}

export interface ToothIndication {
  toothNumber: ToothNumber
  workType:
    | 'crown'
    | 'bridge-abutment'
    | 'pontic'
    | 'implant'
    | 'abutment'
    | 'post-core'
    | 'splint-support'
    | 'aligner-stage'
    | 'denture-tooth'
    | 'partial-anchor'
  materialId?: string | null
  preparationType?: string | null
  notes?: string | null
}

export interface ConstructionNode {
  id: string
  kind: 'single-tooth' | 'bridge-span' | 'implant-stack' | 'guide' | 'splint' | 'aligner-arch' | 'ceph-analysis' | 'model' | 'denture' | 'partial' | 'bar'
  toothNumbers: readonly ToothNumber[]
  inputAssetIds: readonly string[]
  outputAssetKinds: readonly CadCoreAssetKind[]
  requiredTools: readonly string[]
  requiredJobs: readonly string[]
}

export interface ConstructionGraph {
  id: string
  nodes: readonly ConstructionNode[]
  edges: readonly { from: string; to: string; reason: string }[]
}

export interface WorkDefinition {
  id: string
  caseId: string
  moduleId: CadCoreModuleId
  indications: readonly ToothIndication[]
  constructionGraph: ConstructionGraph
  workflowId: string
  createdAt: string
  updatedAt: string
}

export interface CadObject {
  id: string
  assetId?: string | null
  kind: 'mesh' | 'dicom-volume' | 'mask' | 'curve' | 'landmark' | 'implant' | 'sleeve' | 'attachment' | 'report-anchor'
  label: string
  layerId: string
  visible: boolean
  locked: boolean
  transform: {
    position: readonly [number, number, number]
    rotation: readonly [number, number, number]
    scale: readonly [number, number, number]
  }
  render: {
    mode: 'solid' | 'wireframe' | 'xray' | 'points' | 'volume'
    triangleBudget?: number
    meshKey?: string
    bufferHandle?: string
  }
}

export interface Layer {
  id: string
  label: string
  visible: boolean
  locked: boolean
  owner: CadCoreLayer
}

export interface SelectionSet {
  objectIds: readonly string[]
  toothNumbers: readonly ToothNumber[]
  activeObjectId?: string | null
}

export interface SceneGraph {
  id: string
  caseId: string
  revision: number
  layers: readonly Layer[]
  objects: readonly CadObject[]
  selection: SelectionSet
  updatedAt: string
}

export interface CadToolDefinition {
  id: string
  label: string
  category: CadCoreToolCategory
  owner: CadCoreLayer
  permissions: readonly string[]
  requiredAssets: readonly CadCoreAssetKind[]
  jobKinds: readonly string[]
  performanceRule: string
}

export interface CadToolContext {
  caseId: string
  moduleId: CadCoreModuleId
  sceneId: string
  selection: SelectionSet
  assetIds: readonly string[]
  params?: Record<string, unknown>
}

export interface WizardGuard {
  kind: WizardGuardKind
  ref: string
  message: string
}

export interface WizardStep {
  id: string
  label: string
  owner: CadCoreLayer
  tools: readonly string[]
  jobs: readonly string[]
  requiredAssets: readonly CadCoreAssetKind[]
  outputAssets: readonly CadCoreAssetKind[]
  guards: readonly WizardGuard[]
}

export interface WizardDefinition {
  id: string
  label: string
  moduleId: CadCoreModuleId
  steps: readonly WizardStep[]
}

export interface ModuleManifest {
  id: CadCoreModuleId
  label: string
  owner: CadCoreLayer
  purpose: string
  workflows: readonly WizardDefinition[]
  tools: readonly string[]
  permissions: readonly string[]
  dependencies: readonly string[]
  outputAssets: readonly CadCoreAssetKind[]
}

export interface ToolRegistry {
  tools: readonly CadToolDefinition[]
  modules: readonly ModuleManifest[]
}

export interface CadJobArtifact {
  id: string
  kind: CadCoreAssetKind | 'mesh-key' | 'metric' | 'operation-progress'
  storagePath?: string | null
  checksumSha256?: string | null
  metadata?: Record<string, unknown>
}

export interface CadJobRequest {
  caseId: string
  moduleId: CadCoreModuleId
  kind: string
  runtime: Extract<CadCoreLayer, 'rust-core' | 'python-sidecar' | 'tauri-command'>
  inputAssetIds: readonly string[]
  params: Record<string, unknown>
}

export interface CadJobStatus {
  id: string
  caseId: string
  kind: string
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'manual-review'
  progress: number
  runtime: CadJobRequest['runtime']
  artifacts: readonly CadJobArtifact[]
  error?: string | null
}

export interface FeaturePermissionContext {
  features: ReadonlySet<string> | readonly string[]
  role?: string | null
  moduleId?: CadCoreModuleId | null
  installedDependencies?: readonly string[]
}

export type FeaturePermissionExpression = string

export interface CadCoreDependency {
  id: string
  label: string
  owner: CadCoreLayer
  requiredFor: readonly CadCoreModuleId[]
  offline: boolean
  notes: string
}

export interface CadCoreServiceContract {
  id: string
  sourcePattern: 'scanner-wizard' | 'mesh-vault' | 'ortho-fipos' | 'bitesplint-bottom' | 'restorative-ai' | 'dicom-ai'
  owner: CadCoreLayer
  operations: readonly string[]
  inputRefs: readonly string[]
  outputRefs: readonly string[]
  streaming: 'none' | 'client' | 'server' | 'bidirectional'
  performanceRule: string
}

export const CAD_CORE_CASE_DIRECTORIES = [
  'input/scans',
  'input/dicom',
  'input/photos',
  'input/jaw-motion',
  'working/scene',
  'working/meshes',
  'working/masks',
  'working/registrations',
  'working/articulator',
  'working/aligners',
  'jobs/rust',
  'jobs/python',
  'jobs/logs',
  'libraries/implants',
  'libraries/materials',
  'libraries/teeth',
  'output/stl',
  'output/obj',
  'output/3mf',
  'output/reports',
  'output/surgical-guide',
  'output/cam',
] as const

export const CAD_CORE_TOOLS = [
  tool('select', 'Select', 'scene', 'react-ui', [], [], [], 'Selector writes object ids only; no mesh copies.'),
  tool('transform', 'Move / rotate / scale', 'scene', 'three-render', [], [], [], 'Transform handles update scene graph deltas, not React mesh snapshots.'),
  tool('layers', 'Layers / groups', 'scene', 'react-ui', [], [], [], 'Layer toggles are batched and invalidate the viewport once.'),
  tool('scan-wizard', 'Scan wizard', 'scan', 'tauri-command', ['case:create'], [], ['scan-session'], 'Scanner events stream into jobs/assets; UI never owns scanner state.'),
  tool('mesh-vault', 'Mesh push / pull / find', 'mesh', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['mesh-import', 'mesh-export'], 'Meshes are addressed by hash keys and streamed in chunks.'),
  tool('measure', 'Measure', 'mesh', 'three-render', [], ['stl-mesh'], [], 'GPU picking uses cached buffers; persistent measurements are command events.'),
  tool('thickness', 'Minimum thickness', 'mesh', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['thickness-map'], 'Thickness maps are derived artifacts and never live in React state.'),
  tool('cut-view', 'Cut view', 'mesh', 'three-render', [], ['stl-mesh'], [], 'Section planes are render state until accepted as a command.'),
  tool('repair', 'Repair mesh', 'mesh', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['mesh-repair'], 'Repair runs async in Rust with artifact lineage.'),
  tool('offset', 'Offset', 'mesh', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['mesh-offset'], 'Offset output is a derived asset with source checksum.'),
  tool('boolean', 'Boolean', 'mesh', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['mesh-boolean'], 'Boolean is never executed on the React thread.'),
  tool('margin', 'Margin', 'dental-design', 'rust-core', ['cad:mesh_edit'], ['stl-mesh'], ['margin-detection'], 'Manual edits are curves; automatic detection is a cancellable job.'),
  tool('insertion-axis', 'Insertion axis', 'dental-design', 'three-render', ['cad:mesh_edit'], ['stl-mesh'], ['blockout-preview'], 'Axis preview stays lightweight; blockout map is derived.'),
  tool('crown-bottom', 'Crown bottom', 'dental-design', 'rust-core', ['module:cad'], ['stl-mesh'], ['crown-bottom'], 'Cement gap and minimum thickness are params, not UI constants.'),
  tool('connectors', 'Connectors', 'dental-design', 'rust-core', ['module:cad'], ['stl-mesh'], ['connector-validation'], 'Connector thresholds come from material library.'),
  tool('contacts', 'Contacts / occlusion', 'articulator', 'python-sidecar', ['module:splint'], ['stl-mesh'], ['contact-analysis', 'occlusion-map'], 'Distance maps are artifacts rendered by Three.'),
  tool('articulator', 'Virtual articulator', 'articulator', 'rust-core', ['module:splint'], ['stl-mesh'], ['jaw-registration', 'jaw-motion'], 'Jaw transforms are sampled and cached, not recomputed per frame.'),
  tool('dicom-mpr', 'DICOM MPR', 'dicom', 'python-sidecar', ['asset:import_dicom'], ['dicom-series'], ['dicom-metadata', 'dicom-preview'], 'Volume IO is tiled/streamed through Python/Rust.'),
  tool('dicom-segmentation', 'DICOM segmentation', 'dicom', 'python-sidecar', ['module:dicom'], ['dicom-series'], ['dicom-segmentation'], 'ONNX/TorchScript runs offline in the sidecar.'),
  tool('implant-library', 'Implant library', 'implant', 'tauri-command', ['module:implant'], [], ['implant-plan'], 'Libraries are local app-data manifests.'),
  tool('abutment-cross-section', 'Abutment cross-section profile', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-cross-section'], 'Profile buffers are generated once per preset and reused across preview/export.'),
  tool('abutment-margin-loop', 'Abutment margin loop', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-margin-loop'], 'Margin loops are curve assets; surface projection uses cached BVH queries.'),
  tool('abutment-collar', 'Abutment collar body', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-collar-body'], 'Interactive edits batch into accepted jobs instead of per-slider mesh rewrites.'),
  tool('abutment-shrinkwrap', 'Abutment surface adaptation', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-shrinkwrap'], 'Projection caches source mesh acceleration structures by mesh hash.'),
  tool('abutment-screw-channel', 'Abutment screw channel', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-boolean-cut'], 'Screw-channel booleans run as cancellable Rust jobs; Three only previews tool placement.'),
  tool('abutment-cleanup', 'Abutment mesh cleanup', 'implant', 'rust-core', ['module:implant', 'cad:mesh_edit'], ['stl-mesh'], ['abutment-mesh-cleanup'], 'Remesh/smooth/weld/decimate/manifold cleanup is batched to reduce asset IO.'),
  tool('abutment-report', 'Abutment export package', 'manufacturing', 'python-sidecar', ['export:manufacturing'], ['stl-mesh'], ['abutment-export-package'], 'STL/report output streams through the asset vault with hashes and implant metadata.'),
  tool('sleeve-guide', 'Sleeve / surgical guide', 'implant', 'rust-core', ['module:surgical_guide'], ['stl-mesh'], ['guide-preview', 'guide-export'], 'Guide booleans and drill protocol run as jobs.'),
  tool('ceph-landmarks', 'Ceph landmarks', 'ceph', 'python-sidecar', ['module:ceph'], ['dicom-series', 'photo'], ['ceph-landmarks'], 'Landmark confidence is stored with manual overrides.'),
  tool('ortho-setup', 'Ortho setup', 'ortho', 'python-sidecar', ['module:aligner'], ['stl-mesh'], ['tooth-segmentation', 'ortho-fipos'], 'Staging result stores per-tooth transforms and constraints.'),
  tool('aligner-attachments', 'Attachments / IPR', 'ortho', 'rust-core', ['module:aligner'], ['stl-mesh'], ['attachment-validation', 'ipr-map'], 'Attachment instances use instancing in render and artifacts in storage.'),
  tool('export-manufacturing', 'Manufacturing export', 'manufacturing', 'rust-core', ['export:manufacturing'], ['stl-mesh'], ['stl-export', '3mf-export'], 'Exports are deterministic and include provenance.'),
] as const satisfies readonly CadToolDefinition[]

export const CAD_CORE_SERVICE_CONTRACTS = [
  serviceContract('scanner-wizard-service', 'scanner-wizard', 'tauri-command', ['connect', 'preview', 'scan', 'pause', 'resume', 'cancel', 'triangulate', 'finish', 'write-viewer-object'], ['Treatment', 'ScanDefinition', 'ScannerSettings'], ['ScanWizardResult', 'scan mesh asset', 'scanner events'], 'server', 'Scanner state is event-driven; mesh payloads become asset refs before React sees them.'),
  serviceContract('mesh-vault-service', 'mesh-vault', 'rust-core', ['importFromPath', 'jobStatus', 'cancel', 'findMesh', 'pushMesh', 'pullMesh', 'pushTexture', 'pullTexture'], ['local file path', 'mesh hash key', 'mesh kind', 'format', 'ttl', 'metadata chunks'], ['mesh key', 'job progress', 'blob chunks', 'texture count', 'GPU upload hints'], 'bidirectional', 'Large meshes are chunked and keyed by hash; UI passes keys and never clones byte buffers.'),
  serviceContract('ortho-fipos-service', 'ortho-fipos', 'python-sidecar', ['pushInitialBite', 'pushBite', 'pullSegmentedBite', 'pullFipos', 'pushFeedback', 'listAssets', 'resetPatientData'], ['bite id', 'creation control', 'IPR settings', 'restorative objects'], ['tooth transforms', 'stage count', 'interdental distances', 'boundary constraints'], 'server', 'Aligner setup is a job artifact; Three renders staged transforms without recalculating treatment planning.'),
  serviceContract('bitesplint-bottom-service', 'bitesplint-bottom', 'rust-core', ['createBottom'], ['jaw scan mesh key', 'insertion axis', 'max undercut', 'offset', 'milling head radius', 'closure radius'], ['bottom mesh key', 'vanilla mesh key', 'operation progress'], 'server', 'Splint bottom generation streams progress and emits mesh keys; no blocking WebGL or React state writes.'),
  serviceContract('dicom-ai-service', 'dicom-ai', 'python-sidecar', ['sanitize', 'metadata', 'mprPreview', 'segment', 'extractMesh', 'landmarks'], ['dicom series path', 'model id', 'mask params'], ['sanitized series', 'preview tiles', 'mask', 'mesh key', 'landmarks'], 'server', 'DICOM volumes are streamed/tiled; segmentation is offline and cacheable by series hash.'),
] as const satisfies readonly CadCoreServiceContract[]

export const CAD_CORE_DEPENDENCIES = [
  dep('sqlite-local-db', 'SQLite local DB', 'asset-vault', ['cad', 'dicom', 'implant', 'guide', 'splint', 'ceph', 'aligners', 'orthocad'], true, 'Metadata, jobs, permissions and audit index.'),
  dep('case-folder-vault', 'Case folder vault', 'asset-vault', ['cad', 'dicom', 'implant', 'guide', 'splint', 'ceph', 'aligners', 'orthocad'], true, 'Large files, meshes, DICOM, masks, reports and exports.'),
  dep('rust-mesh-core', 'Rust mesh core', 'rust-core', ['cad', 'guide', 'splint', 'model-creator', 'partials'], true, 'Mesh repair, boolean, offset, export and registration.'),
  dep('python-ai-dicom', 'Embedded Python AI/DICOM', 'python-sidecar', ['dicom', 'implant', 'ceph', 'aligners', 'orthocad'], true, 'PyDICOM, ONNX/TorchScript/MONAI style jobs without external APIs.'),
  dep('three-viewport', 'Three viewport', 'three-render', ['cad', 'dicom', 'implant', 'guide', 'splint', 'ceph', 'aligners', 'orthocad'], true, 'Rendering only; no workflow or compute ownership.'),
  dep('implant-library', 'Implant and sleeve libraries', 'asset-vault', ['implant', 'guide', 'tlanticad-implant', 'tlanticad-abutment'], true, 'Local manifests for implants, scan bodies, sleeves and drill protocols.'),
  dep('material-library', 'Material library', 'asset-vault', ['cad', 'fab', 'tlanticad-crown', 'tlanticad-bridge', 'tlanticad-bite-splint'], true, 'Material rules, connector thresholds, minimum thickness and export profiles.'),
] as const satisfies readonly CadCoreDependency[]

export const CAD_CORE_WIZARDS = [
  wizard('cad-core-import-design-export', 'Core CAD: Import -> Clean -> Segment -> Design -> Validate -> Export', 'cad', [
    step('cad-import', 'Import scans/assets', 'tauri-command', ['scan-wizard', 'mesh-vault'], ['scan-session', 'mesh-import'], [], ['stl-mesh', 'obj-mesh'], [guard('permission', 'case:create', 'Case creation permission is required.')]),
    step('cad-clean', 'Clean and repair', 'rust-core', ['repair', 'cut-view'], ['mesh-repair'], ['stl-mesh'], ['stl-mesh'], [guard('permission', 'cad:mesh_edit', 'Mesh edit permission is required.')]),
    step('cad-segment', 'Segment / identify anatomy', 'python-sidecar', ['dicom-segmentation', 'margin'], ['tooth-segmentation', 'margin-detection'], ['stl-mesh'], ['mask'], [guard('dependency', 'python-ai-dicom', 'Python AI/DICOM runtime must be available.')]),
    step('cad-design', 'Design restoration', 'rust-core', ['margin', 'insertion-axis', 'crown-bottom', 'connectors', 'contacts'], ['crown-bottom', 'connector-validation', 'contact-analysis'], ['stl-mesh'], ['stl-mesh'], [guard('asset', 'prep-scan', 'A preparation scan is required.')]),
    step('cad-validate-export', 'Validate and export', 'rust-core', ['measure', 'contacts', 'export-manufacturing'], ['thickness-map', 'stl-export'], ['stl-mesh'], ['manufacturing-export', 'report'], [guard('permission', 'export:manufacturing', 'Manufacturing export permission is required.')]),
  ]),
  wizard('dicom-ai-pipeline', 'DICOM: Load -> Preprocess -> Infer -> Postprocess -> Edit', 'dicom', [
    step('dicom-load', 'Load DICOM series', 'tauri-command', ['dicom-mpr'], ['dicom-metadata'], ['dicom-series'], ['scene-snapshot'], [guard('permission', 'asset:import_dicom', 'DICOM import permission is required.')]),
    step('dicom-infer', 'Offline segmentation', 'python-sidecar', ['dicom-segmentation'], ['dicom-segmentation'], ['dicom-series'], ['mask'], [guard('dependency', 'python-ai-dicom', 'Python AI/DICOM runtime must be available.')]),
    step('dicom-mesh', 'Mesh extraction and CAD handoff', 'rust-core', ['mesh-vault'], ['mask-to-mesh'], ['mask'], ['stl-mesh'], [guard('job-complete', 'dicom-segmentation', 'Segmentation must finish before mesh extraction.')]),
  ]),
  wizard('implant-surgical-guide', 'Implant: DICOM/STL -> Plan -> Guide -> Report', 'implant', [
    step('implant-ingest', 'CBCT and STL ingest', 'python-sidecar', ['dicom-mpr', 'mesh-vault'], ['dicom-metadata', 'mesh-import'], ['dicom-series', 'stl-mesh'], ['scene-snapshot'], [guard('permission', 'module:implant', 'Implant module permission is required.')]),
    step('implant-plan', 'Implant and abutment plan', 'rust-core', ['implant-library', 'insertion-axis', 'measure'], ['surface-registration', 'implant-plan'], ['stl-mesh'], ['report'], [guard('dependency', 'implant-library', 'Implant library must be installed locally.')]),
    step('implant-guide', 'Surgical guide and drill protocol', 'rust-core', ['sleeve-guide', 'boolean', 'export-manufacturing'], ['guide-preview', 'guide-export'], ['stl-mesh'], ['manufacturing-export', 'report'], [guard('permission', 'module:surgical_guide', 'Surgical guide permission is required.')]),
  ]),
  wizard('custom-abutment-industrial-v1', 'Custom Abutment: Platform -> Profile -> Adapt -> Channel -> Export', 'implant', [
    step('abutment-platform-context', 'Implant platform and tissue context', 'tauri-command', ['implant-library'], ['implant-platform-resolve'], ['stl-mesh'], ['scene-snapshot'], [guard('permission', 'module:implant', 'Implant module permission is required.'), guard('dependency', 'implant-library', 'Implant library must be installed locally.')]),
    step('abutment-profile-margin', 'Cross-section profile and margin loop', 'rust-core', ['abutment-cross-section', 'abutment-margin-loop'], ['abutment-cross-section', 'abutment-margin-loop'], ['stl-mesh'], ['stl-mesh'], [guard('permission', 'cad:mesh_edit', 'Mesh edit permission is required.')]),
    step('abutment-collar-adapt', 'Collar, emergence and surface adaptation', 'rust-core', ['abutment-collar', 'abutment-shrinkwrap'], ['abutment-collar-body', 'abutment-shrinkwrap'], ['stl-mesh'], ['stl-mesh'], [guard('asset', 'gingiva-scan', 'A gingiva/prep surface scan is required.')]),
    step('abutment-channel-cleanup', 'Screw channel and mesh cleanup', 'rust-core', ['abutment-screw-channel', 'abutment-cleanup', 'thickness'], ['abutment-boolean-cut', 'abutment-mesh-cleanup', 'thickness-map'], ['stl-mesh'], ['stl-mesh', 'report'], [guard('manual-review', 'screw-channel-angle', 'Angulated screw channels require manual review.')]),
    step('abutment-export-package', 'STL, construction info and report', 'python-sidecar', ['abutment-report', 'export-manufacturing'], ['abutment-export-package', 'stl-export'], ['stl-mesh'], ['manufacturing-export', 'report'], [guard('permission', 'export:manufacturing', 'Manufacturing export permission is required.')]),
  ]),
  wizard('splint-articulator', 'Splint: Bite -> Articulator -> Bottom -> Top -> Export', 'splint', [
    step('splint-bite', 'Scan pair and bite relation', 'tauri-command', ['scan-wizard', 'articulator'], ['jaw-registration'], ['stl-mesh'], ['scene-snapshot'], [guard('asset', 'maxilla-scan', 'Maxilla scan is required.'), guard('asset', 'mandible-scan', 'Mandible scan is required.')]),
    step('splint-bottom', 'Create bite splint bottom', 'rust-core', ['insertion-axis', 'offset'], ['bitesplint-create-bottom'], ['stl-mesh'], ['stl-mesh'], [guard('permission', 'module:splint', 'Splint module permission is required.')]),
    step('splint-occlusion', 'Contacts and occlusion', 'python-sidecar', ['contacts', 'articulator'], ['jaw-motion', 'occlusion-map'], ['stl-mesh'], ['mask', 'report'], [guard('dependency', 'python-ai-dicom', 'Python runtime is required for jaw motion analysis.')]),
    step('splint-export', 'Validate and export splint', 'rust-core', ['measure', 'export-manufacturing'], ['thickness-map', 'stl-export'], ['stl-mesh'], ['manufacturing-export'], [guard('permission', 'export:manufacturing', 'Manufacturing export permission is required.')]),
  ]),
  wizard('ceph-analysis', 'Ceph: Image/CBCT -> Landmarks -> Trace -> Report', 'ceph', [
    step('ceph-load', 'Load image or CBCT', 'tauri-command', ['dicom-mpr'], ['dicom-metadata'], ['dicom-series', 'photo'], ['scene-snapshot'], [guard('permission', 'module:ceph', 'Ceph module permission is required.')]),
    step('ceph-landmarks', 'Landmarks and planes', 'python-sidecar', ['ceph-landmarks'], ['ceph-landmarks'], ['dicom-series', 'photo'], ['report'], [guard('dependency', 'python-ai-dicom', 'Python AI runtime is required.')]),
    step('ceph-report', 'Measurements and report', 'react-ui', ['measure'], ['ceph-report'], ['report'], ['report'], [guard('manual-review', 'landmark-confidence', 'Low confidence landmarks require manual review.')]),
  ]),
  wizard('ortho-aligner-fipos', 'Ortho/Aligners: Segment -> Setup -> Stage -> Attachments -> Export', 'aligners', [
    step('aligner-segment', 'Segment teeth and roots', 'python-sidecar', ['ortho-setup'], ['tooth-segmentation'], ['stl-mesh', 'dicom-series'], ['mask'], [guard('permission', 'module:aligner', 'Aligner module permission is required.')]),
    step('aligner-setup', 'FIPOS setup and constraints', 'python-sidecar', ['ortho-setup'], ['ortho-fipos'], ['stl-mesh'], ['scene-snapshot', 'report'], [guard('job-complete', 'tooth-segmentation', 'Tooth segmentation must be completed.')]),
    step('aligner-attachments', 'Attachments, IPR and collision', 'rust-core', ['aligner-attachments', 'measure'], ['attachment-validation', 'ipr-map'], ['stl-mesh'], ['stl-mesh', 'report'], [guard('manual-review', 'collision-check', 'Collision and IPR changes require review.')]),
    step('aligner-export', 'Tray export', 'rust-core', ['export-manufacturing'], ['3mf-export', 'stl-export'], ['stl-mesh'], ['manufacturing-export'], [guard('permission', 'export:manufacturing', 'Manufacturing export permission is required.')]),
  ]),
] as const satisfies readonly WizardDefinition[]

export const CAD_CORE_MODULE_MANIFESTS = [
  manifest('cad', 'CAD Core', 'rust-core', 'Mesh-first restorative CAD core.', ['cad-core-import-design-export'], ['select', 'transform', 'layers', 'mesh-vault', 'measure', 'repair', 'offset', 'boolean', 'margin', 'insertion-axis', 'crown-bottom', 'connectors', 'contacts', 'export-manufacturing'], ['module:cad', 'cad:mesh_edit'], ['sqlite-local-db', 'case-folder-vault', 'rust-mesh-core', 'three-viewport'], ['stl-mesh', 'report', 'manufacturing-export']),
  manifest('dicom', 'DICOM Core', 'python-sidecar', 'CBCT/DICOM processing and CAD handoff.', ['dicom-ai-pipeline'], ['dicom-mpr', 'dicom-segmentation', 'mesh-vault'], ['module:dicom', 'asset:import_dicom'], ['python-ai-dicom', 'case-folder-vault'], ['mask', 'stl-mesh', 'report']),
  manifest('implant', 'Implant Core', 'rust-core', 'Implant planning, abutment and prosthetic handoff.', ['implant-surgical-guide', 'custom-abutment-industrial-v1'], ['implant-library', 'abutment-cross-section', 'abutment-margin-loop', 'abutment-collar', 'abutment-shrinkwrap', 'abutment-screw-channel', 'abutment-cleanup', 'abutment-report', 'sleeve-guide', 'measure', 'insertion-axis', 'dicom-mpr'], ['module:implant'], ['implant-library', 'rust-mesh-core', 'python-ai-dicom'], ['report', 'manufacturing-export']),
  manifest('guide', 'Surgical Guide Core', 'rust-core', 'Sleeve, guide body and drill protocol generation.', ['implant-surgical-guide'], ['sleeve-guide', 'boolean', 'offset', 'export-manufacturing'], ['module:surgical_guide'], ['implant-library', 'material-library', 'rust-mesh-core'], ['manufacturing-export', 'report']),
  manifest('splint', 'Splint and Articulator Core', 'rust-core', 'Bite splints, articulator and occlusion maps.', ['splint-articulator'], ['articulator', 'contacts', 'offset', 'measure', 'export-manufacturing'], ['module:splint'], ['rust-mesh-core', 'python-ai-dicom', 'material-library'], ['manufacturing-export', 'report']),
  manifest('ceph', 'Cephalometrics Core', 'python-sidecar', 'Landmarks, tracing and cephalometric reports.', ['ceph-analysis'], ['ceph-landmarks', 'measure', 'dicom-mpr'], ['module:ceph'], ['python-ai-dicom'], ['report']),
  manifest('aligners', 'Aligner Core', 'python-sidecar', 'Tooth segmentation, setup, staging, IPR and trays.', ['ortho-aligner-fipos'], ['ortho-setup', 'aligner-attachments', 'measure', 'export-manufacturing'], ['module:aligner'], ['python-ai-dicom', 'rust-mesh-core'], ['manufacturing-export', 'report']),
  manifest('orthocad', 'Smile and Ortho Core', 'python-sidecar', 'Smile, orthodontic setup and waxup handoff.', ['ortho-aligner-fipos'], ['ortho-setup', 'ceph-landmarks', 'measure'], ['module:orthocad'], ['python-ai-dicom'], ['scene-snapshot', 'report']),
  manifest('model-creator', 'Model Creator Core', 'rust-core', 'Printable models, bases, labels and dies.', ['cad-core-import-design-export'], ['mesh-vault', 'repair', 'offset', 'measure', 'export-manufacturing'], ['module:model_creator'], ['rust-mesh-core', 'material-library'], ['manufacturing-export']),
  manifest('partials', 'Partial and Bar Core', 'rust-core', 'Survey, framework, bars, clasps and telescopes.', ['cad-core-import-design-export'], ['measure', 'offset', 'boolean', 'connectors', 'export-manufacturing'], ['module:partials'], ['rust-mesh-core', 'material-library'], ['manufacturing-export', 'report']),
  manifest('fab', 'Manufacturing Core', 'rust-core', 'Validation, material profiles, STL/3MF and CAM handoff.', ['cad-core-import-design-export'], ['measure', 'repair', 'export-manufacturing'], ['module:fab', 'export:manufacturing'], ['rust-mesh-core', 'material-library'], ['manufacturing-export', 'report']),
] as const satisfies readonly ModuleManifest[]

export function buildCaseFolderLayout(caseId: string, root = 'TlantiCADData/cases'): CaseFolderLayout {
  const safeCaseId = safeSegment(caseId || 'case')
  const caseRoot = `${root}/${safeCaseId}`
  return {
    caseId: safeCaseId,
    root: caseRoot,
    manifestPath: `${caseRoot}/manifest.json`,
    workDefinitionPath: `${caseRoot}/work-definition.json`,
    directories: CAD_CORE_CASE_DIRECTORIES.map((directory) => `${caseRoot}/${directory}`),
  }
}

export function createCaseManifest(input: {
  caseId: string
  caseNumber: string
  patientId: string
  activeModuleId: CadCoreModuleId
  workDefinitionId?: string
  now?: Date
  root?: string
  assets?: readonly AssetManifest[]
}): CaseManifest {
  const timestamp = (input.now ?? new Date()).toISOString()
  return {
    schemaVersion: 1,
    caseId: input.caseId,
    caseNumber: input.caseNumber,
    patientId: input.patientId,
    activeModuleId: input.activeModuleId,
    createdAt: timestamp,
    updatedAt: timestamp,
    layout: buildCaseFolderLayout(input.caseId, input.root),
    assets: input.assets ?? [],
    workDefinitionId: input.workDefinitionId ?? `work-${safeSegment(input.caseId)}`,
  }
}

export function createWorkDefinition(input: {
  caseId: string
  moduleId: CadCoreModuleId
  indications?: readonly ToothIndication[]
  nodes?: readonly ConstructionNode[]
  edges?: ConstructionGraph['edges']
  workflowId?: string
  now?: Date
}): WorkDefinition {
  const timestamp = (input.now ?? new Date()).toISOString()
  return {
    id: `work-${safeSegment(input.caseId)}`,
    caseId: input.caseId,
    moduleId: input.moduleId,
    indications: input.indications ?? [],
    constructionGraph: {
      id: `graph-${safeSegment(input.caseId)}`,
      nodes: input.nodes ?? [],
      edges: input.edges ?? [],
    },
    workflowId: input.workflowId ?? resolveCadCoreModuleManifest(input.moduleId).workflows[0]?.id ?? 'cad-core-import-design-export',
    createdAt: timestamp,
    updatedAt: timestamp,
  }
}

export function evaluateFeaturePermissionExpression(expression: FeaturePermissionExpression, context: FeaturePermissionContext): boolean {
  const features = context.features instanceof Set ? new Set(context.features) : new Set(context.features)
  if (context.role) {
    features.add(`role:${context.role}`)
  }
  if (context.moduleId) {
    features.add(`module:${context.moduleId}`)
  }
  for (const dependency of context.installedDependencies ?? []) {
    features.add(`dependency:${dependency}`)
  }

  const normalized = expression.trim()
  if (!normalized) {
    return true
  }

  return normalized
    .split(';')
    .map((clause) => clause.trim())
    .filter(Boolean)
    .some((clause) =>
      clause
        .split(',')
        .map((term) => term.trim())
        .filter(Boolean)
        .every((term) => {
          const negated = term.startsWith('!')
          const feature = negated ? term.slice(1).trim() : term
          const allowed = features.has(feature)
          return negated ? !allowed : allowed
        }),
    )
}

export function resolveCadCoreModuleManifest(moduleId?: CadCoreModuleId | string | null): ModuleManifest {
  return CAD_CORE_MODULE_MANIFESTS.find((module) => module.id === moduleId) ?? CAD_CORE_MODULE_MANIFESTS[0]
}

export function listCadCoreToolsForModule(moduleId: CadCoreModuleId): readonly CadToolDefinition[] {
  const manifestDef = resolveCadCoreModuleManifest(moduleId)
  const ids = new Set(manifestDef.tools)
  return CAD_CORE_TOOLS.filter((toolDef) => ids.has(toolDef.id))
}

export function validateCadCorePlatform(): string[] {
  const issues: string[] = []
  const toolIds = new Set(CAD_CORE_TOOLS.map((toolDef) => toolDef.id))
  const dependencyIds = new Set(CAD_CORE_DEPENDENCIES.map((dependency) => dependency.id))
  const wizardIds = new Set(CAD_CORE_WIZARDS.map((wizardDef) => wizardDef.id))

  for (const manifestDef of CAD_CORE_MODULE_MANIFESTS) {
    for (const toolId of manifestDef.tools) {
      if (!toolIds.has(toolId)) {
        issues.push(`${manifestDef.id} references missing tool ${toolId}`)
      }
    }
    for (const dependencyId of manifestDef.dependencies) {
      if (!dependencyIds.has(dependencyId)) {
        issues.push(`${manifestDef.id} references missing dependency ${dependencyId}`)
      }
    }
    for (const workflow of manifestDef.workflows) {
      if (!wizardIds.has(workflow.id)) {
        issues.push(`${manifestDef.id} references missing workflow ${workflow.id}`)
      }
    }
  }

  for (const wizardDef of CAD_CORE_WIZARDS) {
    for (const stepDef of wizardDef.steps) {
      for (const toolId of stepDef.tools) {
        if (!toolIds.has(toolId)) {
          issues.push(`${wizardDef.id}/${stepDef.id} references missing tool ${toolId}`)
        }
      }
    }
  }

  return issues
}

function tool(
  id: string,
  label: string,
  category: CadCoreToolCategory,
  owner: CadCoreLayer,
  permissions: readonly string[],
  requiredAssets: readonly CadCoreAssetKind[],
  jobKinds: readonly string[],
  performanceRule: string,
): CadToolDefinition {
  return { id, label, category, owner, permissions, requiredAssets, jobKinds, performanceRule }
}

function dep(
  id: string,
  label: string,
  owner: CadCoreLayer,
  requiredFor: readonly CadCoreModuleId[],
  offline: boolean,
  notes: string,
): CadCoreDependency {
  return { id, label, owner, requiredFor, offline, notes }
}

function wizard(id: string, label: string, moduleId: CadCoreModuleId, steps: readonly WizardStep[]): WizardDefinition {
  return { id, label, moduleId, steps }
}

function step(
  id: string,
  label: string,
  owner: CadCoreLayer,
  tools: readonly string[],
  jobs: readonly string[],
  requiredAssets: readonly CadCoreAssetKind[],
  outputAssets: readonly CadCoreAssetKind[],
  guards: readonly WizardGuard[],
): WizardStep {
  return { id, label, owner, tools, jobs, requiredAssets, outputAssets, guards }
}

function guard(kind: WizardGuardKind, ref: string, message: string): WizardGuard {
  return { kind, ref, message }
}

function serviceContract(
  id: string,
  sourcePattern: CadCoreServiceContract['sourcePattern'],
  owner: CadCoreLayer,
  operations: readonly string[],
  inputRefs: readonly string[],
  outputRefs: readonly string[],
  streaming: CadCoreServiceContract['streaming'],
  performanceRule: string,
): CadCoreServiceContract {
  return { id, sourcePattern, owner, operations, inputRefs, outputRefs, streaming, performanceRule }
}

function manifest(
  id: CadCoreModuleId,
  label: string,
  owner: CadCoreLayer,
  purpose: string,
  workflowIds: readonly string[],
  tools: readonly string[],
  permissions: readonly string[],
  dependencies: readonly string[],
  outputAssets: readonly CadCoreAssetKind[],
): ModuleManifest {
  return {
    id,
    label,
    owner,
    purpose,
    workflows: workflowIds.map((workflowId) => CAD_CORE_WIZARDS.find((wizardDef) => wizardDef.id === workflowId)).filter(Boolean) as WizardDefinition[],
    tools,
    permissions,
    dependencies,
    outputAssets,
  }
}

function safeSegment(value: string): string {
  const sanitized = value.replace(/[^a-zA-Z0-9._-]/g, '')
  return sanitized || 'case'
}

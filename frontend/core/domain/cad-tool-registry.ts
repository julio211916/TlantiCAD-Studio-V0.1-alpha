import {
  CAD_PRODUCT_MODULE_DEFINITIONS,
  TLANTI_CAD_PRODUCT_MODULE_IDS,
  type TlantiCadProductModuleId,
} from './cad-product-module-registry'
import { CAD_MODULE_ROADMAP_DEFINITIONS } from './cad-module-roadmap'
import type { TlantiModuleId } from './entities'
import { TLANTI_MODULE_DEFINITIONS } from './module-registry'

export type CadToolOwner = 'react-ui' | 'three-render' | 'tauri-command' | 'rust-core' | 'python-sidecar'
export type CadToolRuntime = 'react' | 'three' | 'tauri' | 'rust' | 'python'
export type CadToolStatus = 'ready' | 'planned' | 'disabled'
export type CadToolCategory =
  | 'scene'
  | 'view'
  | 'mesh'
  | 'dental'
  | 'dicom'
  | 'implant'
  | 'guide'
  | 'splint'
  | 'ortho'
  | 'ceph'
  | 'manufacturing'
  | 'ai'

export type CadToolRuntimePlacement = 'top-dock' | 'rail' | 'wizard' | 'context-menu' | 'command-palette' | 'panel'

export type CadToolCapability =
  | 'selection'
  | 'transform'
  | 'measurement'
  | 'visual-preview'
  | 'mesh-read'
  | 'mesh-write'
  | 'job'
  | 'asset-import'
  | 'asset-export'
  | 'workflow'
  | 'ai-local'
  | 'case-state'

export type CadToolId = string

export interface CadToolDefinition {
  id: CadToolId
  label: string
  category: CadToolCategory
  owner: CadToolOwner
  runtime: CadToolRuntime
  status: CadToolStatus
  placements: readonly CadToolRuntimePlacement[]
  capabilities: readonly CadToolCapability[]
  commandId: string
  runtimeCommand?: string
  requiresActiveMesh?: boolean
  requiresActiveDicom?: boolean
  notes: string
}

type ToolSeed = Omit<CadToolDefinition, 'id' | 'commandId' | 'runtime' | 'status' | 'placements' | 'capabilities'> & {
  id: CadToolId
  commandId?: string
  runtime?: CadToolRuntime
  status?: CadToolStatus
  runtimeCommand?: string
  placements?: readonly CadToolRuntimePlacement[]
  capabilities?: readonly CadToolCapability[]
}

const RUNTIME_BY_OWNER: Record<CadToolOwner, CadToolRuntime> = {
  'react-ui': 'react',
  'three-render': 'three',
  'tauri-command': 'tauri',
  'rust-core': 'rust',
  'python-sidecar': 'python',
}

function defaultStatus(seed: ToolSeed): CadToolStatus {
  if (seed.status) return seed.status
  if (seed.runtimeCommand || seed.owner === 'react-ui' || seed.owner === 'three-render') return 'ready'
  return 'planned'
}

function tool(seed: ToolSeed): CadToolDefinition {
  return {
    commandId: `cad.tool.${seed.id}`,
    runtime: seed.runtime ?? RUNTIME_BY_OWNER[seed.owner],
    status: defaultStatus(seed),
    placements: ['command-palette'],
    capabilities: [],
    ...seed,
  }
}

export const CAD_TOOL_DEFINITIONS = [
  tool({ id: 'select', label: 'Select', category: 'scene', owner: 'react-ui', placements: ['top-dock', 'rail', 'context-menu', 'command-palette'], capabilities: ['selection'], notes: 'Selection stores ids only.' }),
  tool({ id: 'move', label: 'Move', category: 'scene', owner: 'three-render', placements: ['top-dock', 'rail', 'command-palette'], capabilities: ['transform', 'visual-preview'], notes: 'Preview matrix in Three; commit as command delta.' }),
  tool({ id: 'rotate', label: 'Rotate', category: 'scene', owner: 'three-render', placements: ['top-dock', 'rail', 'command-palette'], capabilities: ['transform', 'visual-preview'], notes: 'No geometry mutation during preview.' }),
  tool({ id: 'scale', label: 'Scale', category: 'scene', owner: 'three-render', placements: ['top-dock', 'rail', 'command-palette'], capabilities: ['transform', 'visual-preview'], notes: 'Scale writes transform delta, not mesh buffer.' }),
  tool({ id: 'clip', label: 'Cut / Clip', category: 'view', owner: 'three-render', placements: ['top-dock', 'context-menu', 'command-palette'], capabilities: ['visual-preview'], requiresActiveMesh: true, notes: 'Render clipping plane until accepted.' }),
  tool({ id: 'crop', label: 'Crop Scene', category: 'view', owner: 'react-ui', placements: ['top-dock', 'command-palette'], capabilities: ['visual-preview'], notes: 'Viewport crop selection only.' }),
  tool({ id: 'boolean', label: 'Boolean Cut', category: 'mesh', owner: 'rust-core', placements: ['top-dock', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job'], commandId: 'mesh.boolean.cut', runtimeCommand: 'mesh_op', requiresActiveMesh: true, notes: 'Boolean runs as Rust job.' }),
  tool({ id: 'sculpt', label: 'Sculpt', category: 'mesh', owner: 'three-render', placements: ['top-dock', 'rail', 'command-palette'], capabilities: ['mesh-write', 'visual-preview'], requiresActiveMesh: true, notes: 'Brush strokes store params; backend owns destructive commit.' }),
  tool({ id: 'segment', label: 'Segment', category: 'ai', owner: 'python-sidecar', placements: ['top-dock', 'rail', 'command-palette'], capabilities: ['ai-local', 'job'], status: 'planned', notes: 'Local AI segmentation only.' }),
  tool({ id: 'measure', label: 'Measure', category: 'scene', owner: 'three-render', placements: ['top-dock', 'rail', 'context-menu', 'command-palette'], capabilities: ['measurement'], notes: 'Picking reads cached scene refs.' }),
  tool({ id: 'margin', label: 'Margin', category: 'dental', owner: 'rust-core', placements: ['wizard', 'rail', 'context-menu', 'command-palette'], capabilities: ['mesh-read', 'job', 'workflow'], commandId: 'dental.margin.detect', runtimeCommand: 'cad_margin_detect_real', requiresActiveMesh: true, notes: 'Automatic detection is cancellable job; manual edits are curves.' }),
  tool({ id: 'axis', label: 'Insertion Axis', category: 'dental', owner: 'three-render', placements: ['wizard', 'rail', 'context-menu', 'command-palette'], capabilities: ['visual-preview', 'workflow'], requiresActiveMesh: true, notes: 'Axis preview is render state until accepted.' }),
  tool({ id: 'crown-bottom', label: 'Crown Bottoms', category: 'dental', owner: 'rust-core', placements: ['wizard', 'command-palette'], capabilities: ['mesh-write', 'job'], commandId: 'dental.crown-bottom.generate', runtimeCommand: 'cad_crown_bottom_generate', requiresActiveMesh: true, notes: 'Cement gap and minimum thickness are params.' }),
  tool({ id: 'copy-mirror', label: 'Copy / Mirror', category: 'dental', owner: 'rust-core', placements: ['wizard', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Mirroring produces derived asset.' }),
  tool({ id: 'freeform', label: 'Free-form', category: 'dental', owner: 'three-render', placements: ['wizard', 'command-palette'], capabilities: ['mesh-write', 'visual-preview'], requiresActiveMesh: true, notes: 'Clinical sculpt mode for restorative anatomy.' }),
  tool({ id: 'connectors', label: 'Connectors', category: 'dental', owner: 'rust-core', placements: ['wizard', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Connector thresholds come from material profile.' }),
  tool({ id: 'contacts', label: 'Contacts / Occlusion', category: 'dental', owner: 'python-sidecar', placements: ['wizard', 'command-palette'], capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Distance maps are artifacts rendered by Three.' }),
  tool({ id: 'thickness', label: 'Minimum Thickness', category: 'mesh', owner: 'rust-core', placements: ['wizard', 'command-palette'], capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Thickness validation runs as mesh analysis job.' }),
  tool({ id: 'articulator', label: 'Articulator', category: 'splint', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['workflow', 'job'], requiresActiveMesh: true, notes: 'Jaw transforms cached by artifact.' }),
  tool({ id: 'align', label: 'Model Alignment', category: 'scene', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['workflow', 'job'], requiresActiveMesh: true, notes: 'Registration stores immutable transforms.' }),
  tool({ id: 'manufacturing-export', label: 'Export Production', category: 'manufacturing', owner: 'rust-core', placements: ['context-menu', 'command-palette'], capabilities: ['asset-export', 'job'], commandId: 'manufacturing.export', requiresActiveMesh: true, notes: 'Deterministic export with provenance.' }),
  tool({ id: 'repair', label: 'Repair Mesh', category: 'mesh', owner: 'rust-core', placements: ['context-menu', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Mesh repair job creates derived asset.' }),
  tool({ id: 'offset', label: 'Offset', category: 'mesh', owner: 'rust-core', placements: ['command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Offset/shell job with artifact lineage.' }),
  tool({ id: 'trim', label: 'Trim', category: 'mesh', owner: 'rust-core', placements: ['context-menu', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Plane/curve trim is job-backed.' }),
  tool({ id: 'base', label: 'Model Base', category: 'mesh', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Model base generation belongs to backend.' }),
  tool({ id: 'label', label: 'Label', category: 'mesh', owner: 'react-ui', placements: ['panel', 'command-palette'], capabilities: ['workflow'], requiresActiveMesh: true, notes: 'Labels start as annotations and export as derived geometry.' }),
  tool({ id: 'implant-library', label: 'Implant Library', category: 'implant', owner: 'tauri-command', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['workflow', 'case-state'], notes: 'Local implant libraries only.' }),
  tool({ id: 'material-config', label: 'Material Config', category: 'manufacturing', owner: 'react-ui', placements: ['panel', 'command-palette'], capabilities: ['case-state'], notes: 'Material profile drives validation thresholds.' }),
  tool({ id: 'scan-body', label: 'Scan Body Match', category: 'implant', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['job', 'workflow'], requiresActiveMesh: true, notes: 'Scan body matching is registration job.' }),
  tool({ id: 'abutment-design', label: 'Abutment Design', category: 'implant', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Emergence, screw channel and cement gap workflow.' }),
  tool({ id: 'abutment-cross-section', label: 'Abutment Cross Section', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job', 'workflow'], commandId: 'implant.abutment.cross-section.create', requiresActiveMesh: true, notes: 'Ports Blender Cross_Section/CURVE profile logic as parametric Rust mesh generation.' }),
  tool({ id: 'abutment-margin-loop', label: 'Abutment Margin Loop', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-read', 'mesh-write', 'job', 'workflow'], commandId: 'implant.abutment.margin-loop.create', requiresActiveMesh: true, notes: 'Creates explicit margin/emergence curves instead of relying on Blender object names.' }),
  tool({ id: 'abutment-collar', label: 'Abutment Collar', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job', 'workflow'], commandId: 'implant.abutment.collar.generate', requiresActiveMesh: true, notes: 'Generates collar/free-formed emergence body from profile and margin loop.' }),
  tool({ id: 'abutment-shrinkwrap', label: 'Abutment Surface Adapt', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job', 'workflow'], commandId: 'implant.abutment.surface-adapt', requiresActiveMesh: true, notes: 'Ports Shrinkwrap/vertex-weight adaptation as BVH-backed surface projection job.' }),
  tool({ id: 'abutment-screw-channel', label: 'Abutment Screw Channel', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job', 'measurement'], commandId: 'implant.abutment.screw-channel.cut', requiresActiveMesh: true, notes: 'Straight or angulated screw channel boolean with material/tool clearance validation.' }),
  tool({ id: 'abutment-cleanup', label: 'Abutment Mesh Cleanup', category: 'implant', owner: 'rust-core', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['mesh-write', 'job'], commandId: 'implant.abutment.mesh-cleanup', requiresActiveMesh: true, notes: 'Runs remesh/smooth/weld/decimate/manifold cleanup as one cancellable mesh job.' }),
  tool({ id: 'abutment-report', label: 'Abutment Export Package', category: 'manufacturing', owner: 'python-sidecar', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['asset-export', 'job'], commandId: 'implant.abutment.export-package', requiresActiveMesh: true, notes: 'Exports STL, construction info and planning report with source hashes.' }),
  tool({ id: 'bar-design', label: 'Bar Design', category: 'dental', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Bar path creates derived framework.' }),
  tool({ id: 'telescope-fit', label: 'Telescope Fit', category: 'dental', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Spacing/friction validation.' }),
  tool({ id: 'dicom-import', label: 'DICOM Import', category: 'dicom', owner: 'python-sidecar', placements: ['panel', 'command-palette'], capabilities: ['asset-import', 'job'], commandId: 'dicom.import', notes: 'DICOM import streams through local runtime.' }),
  tool({ id: 'dicom-mpr', label: 'DICOM MPR', category: 'dicom', owner: 'python-sidecar', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['visual-preview', 'job'], requiresActiveDicom: true, notes: 'MPR state stays outside React buffers.' }),
  tool({ id: 'dicom-metadata', label: 'DICOM Metadata', category: 'dicom', owner: 'react-ui', placements: ['panel', 'context-menu', 'command-palette'], capabilities: ['case-state'], requiresActiveDicom: true, notes: 'Metadata sanitized before persistence.' }),
  tool({ id: 'dicom-sanitize', label: 'DICOM Sanitize', category: 'dicom', owner: 'python-sidecar', placements: ['command-palette'], capabilities: ['job'], requiresActiveDicom: true, notes: 'PHI-safe local sanitization.' }),
  tool({ id: 'dicom-preview', label: 'DICOM Preview', category: 'dicom', owner: 'three-render', placements: ['panel'], capabilities: ['visual-preview'], requiresActiveDicom: true, notes: 'Preview chunks only.' }),
  tool({ id: 'surface-registration', label: 'Surface Registration', category: 'implant', owner: 'rust-core', capabilities: ['job'], requiresActiveMesh: true, notes: 'CBCT/STL registration transform artifact.' }),
  tool({ id: 'nerve-mark', label: 'Nerve Mark', category: 'implant', owner: 'python-sidecar', capabilities: ['ai-local', 'workflow'], requiresActiveDicom: true, notes: 'Manual/AI mandibular nerve tracing.' }),
  tool({ id: 'sinus-mark', label: 'Sinus Mark', category: 'implant', owner: 'python-sidecar', capabilities: ['ai-local', 'workflow'], requiresActiveDicom: true, notes: 'Sinus safety zone tracing.' }),
  tool({ id: 'implant-planning', label: 'Implant Planning', category: 'implant', owner: 'tauri-command', placements: ['panel', 'command-palette'], capabilities: ['workflow'], notes: 'Implant plan orchestration.' }),
  tool({ id: 'implant-measure', label: 'Implant Measure', category: 'implant', owner: 'three-render', placements: ['panel', 'command-palette'], capabilities: ['measurement'], notes: 'Implant-specific measurement mode.' }),
  tool({ id: 'implant-axis', label: 'Implant Axis', category: 'implant', owner: 'three-render', capabilities: ['visual-preview'], notes: 'Prosthetic axis preview.' }),
  tool({ id: 'sleeve-controls', label: 'Sleeve Controls', category: 'guide', owner: 'rust-core', capabilities: ['workflow', 'job'], notes: 'Sleeve parameters for guide jobs.' }),
  tool({ id: 'guide-wizard', label: 'Guide Wizard', category: 'guide', owner: 'tauri-command', placements: ['panel', 'command-palette'], capabilities: ['workflow'], notes: 'Surgical guide step orchestration.' }),
  tool({ id: 'guide-export', label: 'Guide Export', category: 'guide', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['asset-export', 'job'], notes: 'Guide export with drill protocol.' }),
  tool({ id: 'smile-workflow', label: 'Smile Workflow', category: 'ortho', owner: 'react-ui', placements: ['panel', 'command-palette'], capabilities: ['workflow'], notes: 'Smile design workflow shell.' }),
  tool({ id: 'smile-photos', label: 'Smile Photos', category: 'ortho', owner: 'react-ui', capabilities: ['asset-import'], notes: 'Photo alignment source.' }),
  tool({ id: 'odontogram', label: 'Odontogram', category: 'dental', owner: 'react-ui', placements: ['panel', 'command-palette'], capabilities: ['case-state'], notes: 'Tooth work definition UI.' }),
  tool({ id: 'splint-workflow', label: 'Splint Workflow', category: 'splint', owner: 'tauri-command', placements: ['panel', 'command-palette'], capabilities: ['workflow'], notes: 'Splint wizard orchestration.' }),
  tool({ id: 'splint-export', label: 'Splint Export', category: 'splint', owner: 'rust-core', placements: ['panel', 'command-palette'], capabilities: ['asset-export', 'job'], requiresActiveMesh: true, notes: 'Splint manufacturing export.' }),
  tool({ id: 'layers-panel', label: 'Layers Panel', category: 'scene', owner: 'react-ui', placements: ['rail', 'command-palette'], capabilities: ['case-state'], notes: 'Layer visibility and organization.' }),
  tool({ id: 'groups-panel', label: 'Groups Panel', category: 'scene', owner: 'react-ui', placements: ['rail', 'command-palette'], capabilities: ['case-state'], notes: 'Scene groups and hierarchy.' }),
  tool({ id: 'voice-copilot', label: 'Voice Copilot', category: 'ai', owner: 'react-ui', placements: ['rail', 'command-palette'], capabilities: ['ai-local'], notes: 'Local command suggestions only.' }),
  tool({ id: 'command-palette', label: 'Command Palette', category: 'scene', owner: 'react-ui', placements: ['top-dock', 'rail', 'context-menu', 'command-palette'], capabilities: ['workflow'], notes: 'Searchable local commands.' }),
  tool({ id: 'import-scan', label: 'Import Scan', category: 'mesh', owner: 'tauri-command', capabilities: ['asset-import'], notes: 'Path-based import into Mesh Vault.' }),
  tool({ id: 'tooth-detect', label: 'Tooth Detect', category: 'ai', owner: 'python-sidecar', capabilities: ['ai-local', 'job'], requiresActiveMesh: true, notes: 'Local tooth detection.' }),
  tool({ id: 'prep-select', label: 'Prep Select', category: 'dental', owner: 'react-ui', capabilities: ['selection'], notes: 'Prepared tooth selection.' }),
  tool({ id: 'blockout-preview', label: 'Blockout Preview', category: 'dental', owner: 'three-render', capabilities: ['visual-preview'], requiresActiveMesh: true, notes: 'Lightweight blockout preview.' }),
  tool({ id: 'pontic-design', label: 'Pontic Design', category: 'dental', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Bridge pontic generation.' }),
  tool({ id: 'connector-size', label: 'Connector Size', category: 'dental', owner: 'rust-core', capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Connector sizing validation.' }),
  tool({ id: 'tooth-library', label: 'Tooth Library', category: 'dental', owner: 'tauri-command', capabilities: ['workflow'], notes: 'Local tooth/anatomy library.' }),
  tool({ id: 'anatomy-morph', label: 'Anatomy Morph', category: 'dental', owner: 'three-render', capabilities: ['visual-preview', 'mesh-write'], requiresActiveMesh: true, notes: 'Anatomy morph preview.' }),
  tool({ id: 'smile-guides', label: 'Smile Guides', category: 'ortho', owner: 'react-ui', capabilities: ['visual-preview'], notes: '2D/3D smile guides.' }),
  tool({ id: 'mockup-export', label: 'Mockup Export', category: 'manufacturing', owner: 'rust-core', capabilities: ['asset-export'], notes: 'Mockup export artifact.' }),
  tool({ id: 'case-share-preview', label: 'Case Share Preview', category: 'manufacturing', owner: 'react-ui', capabilities: ['visual-preview'], notes: 'Offline preview package.' }),
  tool({ id: 'import-mesh', label: 'Import Mesh', category: 'mesh', owner: 'tauri-command', capabilities: ['asset-import'], notes: 'Mesh Vault import.' }),
  tool({ id: 'validate-mesh', label: 'Validate Mesh', category: 'mesh', owner: 'rust-core', capabilities: ['job'], requiresActiveMesh: true, notes: 'Mesh validity report.' }),
  tool({ id: 'handoff-module', label: 'Handoff Module', category: 'scene', owner: 'tauri-command', capabilities: ['workflow'], notes: 'Send derived asset to another module.' }),
  tool({ id: 'gingiva-context', label: 'Gingiva Context', category: 'implant', owner: 'three-render', capabilities: ['visual-preview'], requiresActiveMesh: true, notes: 'Gingiva context overlay.' }),
  tool({ id: 'emergence-profile', label: 'Emergence Profile', category: 'implant', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Emergence profile generation.' }),
  tool({ id: 'post-bottom', label: 'Post Bottom', category: 'implant', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Post/core bottom generation.' }),
  tool({ id: 'screw-channel', label: 'Screw Channel', category: 'implant', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Screw channel validation/generation.' }),
  tool({ id: 'core-spacing', label: 'Core Spacing', category: 'implant', owner: 'rust-core', capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Core spacing validation.' }),
  tool({ id: 'antagonist-prompt', label: 'Antagonist Prompt', category: 'dental', owner: 'react-ui', capabilities: ['workflow'], notes: 'Prompts required antagonist assets.' }),
  tool({ id: 'hollow', label: 'Hollow', category: 'mesh', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Model hollowing job.' }),
  tool({ id: 'drainage-holes', label: 'Drainage Holes', category: 'mesh', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Drainage hole booleans.' }),
  tool({ id: 'decimate', label: 'Decimate', category: 'mesh', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Mesh decimation job.' }),
  tool({ id: 'implant-position-ref', label: 'Implant Position Reference', category: 'implant', owner: 'three-render', capabilities: ['visual-preview'], notes: 'Implant ref overlay.' }),
  tool({ id: 'undercut-map', label: 'Undercut Map', category: 'dental', owner: 'rust-core', capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Undercut analysis artifact.' }),
  tool({ id: 'attachment-library', label: 'Attachment Library', category: 'dental', owner: 'tauri-command', capabilities: ['workflow'], notes: 'Local attachment library.' }),
  tool({ id: 'relief', label: 'Relief', category: 'dental', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Relief generation.' }),
  tool({ id: 'mesh-offset', label: 'Mesh Offset', category: 'mesh', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Alias-level explicit mesh offset.' }),
  tool({ id: 'primary-crown', label: 'Primary Crown', category: 'dental', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Telescope primary crown.' }),
  tool({ id: 'spacing-map', label: 'Spacing Map', category: 'dental', owner: 'rust-core', capabilities: ['measurement', 'job'], requiresActiveMesh: true, notes: 'Spacing map artifact.' }),
  tool({ id: 'fit-report', label: 'Fit Report', category: 'manufacturing', owner: 'rust-core', capabilities: ['asset-export'], notes: 'Fit report export.' }),
  tool({ id: 'bite-align', label: 'Bite Align', category: 'splint', owner: 'rust-core', capabilities: ['job'], requiresActiveMesh: true, notes: 'Bite registration alignment.' }),
  tool({ id: 'occlusal-plane', label: 'Occlusal Plane', category: 'splint', owner: 'three-render', capabilities: ['visual-preview'], requiresActiveMesh: true, notes: 'Occlusal plane preview.' }),
  tool({ id: 'ceph-landmarks', label: 'Ceph Landmarks', category: 'ceph', owner: 'python-sidecar', capabilities: ['ai-local', 'job'], notes: 'Landmark detection with manual overrides.' }),
  tool({ id: 'ortho-setup', label: 'Ortho Setup', category: 'ortho', owner: 'python-sidecar', capabilities: ['ai-local', 'job'], requiresActiveMesh: true, notes: 'Tooth segmentation and setup staging.' }),
  tool({ id: 'aligner-attachments', label: 'Attachments / IPR', category: 'ortho', owner: 'rust-core', capabilities: ['mesh-write', 'job'], requiresActiveMesh: true, notes: 'Attachment and IPR validation.' }),
] as const satisfies readonly CadToolDefinition[]

export const CAD_VIEWPORT_TOOL_MODE_IDS = [
  'SELECT',
  'MOVE',
  'ROTATE',
  'SCALE',
  'CLIP',
  'CROP',
  'BOOLEAN_CUT',
  'SCULPT',
  'SEGMENT',
  'MEASURE',
] as const

export const CAD_EXPERT_TOOL_MODE_IDS = [
  'INSERTION',
  'CROWN',
  'COPY',
  'FREEFORM',
  'CONNECTORS',
  'THICKNESS',
  'ARTICULATOR',
  'ALIGN',
  'EXPORT_PROD',
] as const

export const CAD_TOOL_LEGACY_ALIASES: Record<string, CadToolId> = {
  SELECT: 'select',
  MOVE: 'move',
  ROTATE: 'rotate',
  SCALE: 'scale',
  CLIP: 'clip',
  CROP: 'crop',
  BOOLEAN_CUT: 'boolean',
  SCULPT: 'sculpt',
  SEGMENT: 'segment',
  MEASURE: 'measure',
  INSERTION: 'axis',
  CROWN: 'crown-bottom',
  COPY: 'copy-mirror',
  FREEFORM: 'freeform',
  CONNECTORS: 'connectors',
  THICKNESS: 'thickness',
  ARTICULATOR: 'articulator',
  ALIGN: 'align',
  EXPORT_PROD: 'manufacturing-export',
  'import-dicom': 'dicom-import',
  'scan-import': 'import-scan',
  mpr: 'dicom-mpr',
  metadata: 'dicom-metadata',
  'plane-cut': 'trim',
  export: 'manufacturing-export',
  'stl-3mf': 'manufacturing-export',
  'sleeve-preview': 'sleeve-controls',
  trajectory: 'implant-axis',
  'nerve-sinus': 'nerve-mark',
  'occlusion-map': 'contacts',
  'landmark-editor': 'ceph-landmarks',
  'ceph-overlay': 'ceph-landmarks',
  'measurement-table': 'measure',
  report: 'manufacturing-export',
  naming: 'manufacturing-export',
  'material-profile': 'material-config',
  'job-package': 'manufacturing-export',
  staging: 'ortho-setup',
  attachments: 'aligner-attachments',
  'ipr-map': 'aligner-attachments',
  'collision-map': 'aligner-attachments',
  'photo-align': 'smile-photos',
  'smile-lines': 'smile-guides',
  mockup: 'smile-workflow',
  'series-browser': 'dicom-mpr',
  'volume-presets': 'dicom-mpr',
  layers: 'layers-panel',
  groups: 'groups-panel',
  blockout: 'blockout-preview',
  'span-select': 'prep-select',
  'material-route': 'material-config',
  smooth: 'sculpt',
  cut: 'trim',
  Cross_Section: 'abutment-cross-section',
  CURVE: 'abutment-cross-section',
  Collar: 'abutment-collar',
  Free_Formed: 'abutment-collar',
  ActiveSurface: 'abutment-shrinkwrap',
  CuttingTool: 'abutment-screw-channel',
  'screw-channel': 'abutment-screw-channel',
  'abutment-export': 'abutment-report',
  'face-align': 'smile-photos',
  'scan-align': 'align',
  'jaw-motion': 'articulator',
  clasp: 'bar-design',
  'major-connector': 'connectors',
  'minor-connector': 'connectors',
  'mesh-checks': 'validate-mesh',
  sleeve: 'sleeve-controls',
  'fixation-pins': 'guide-wizard',
  undercut: 'undercut-map',
}

const TOOL_BY_ID = new Map(CAD_TOOL_DEFINITIONS.map((definition) => [definition.id, definition]))

export function normalizeCadToolId(id: string): CadToolId {
  return CAD_TOOL_LEGACY_ALIASES[id] ?? id
}

export function resolveCadToolDefinition(id: string): CadToolDefinition | null {
  return TOOL_BY_ID.get(normalizeCadToolId(id)) ?? null
}

export function listCadToolsForProductModule(moduleId: TlantiCadProductModuleId): readonly CadToolDefinition[] {
  const productModule = CAD_PRODUCT_MODULE_DEFINITIONS[moduleId]
  const ids = new Set<CadToolId>()

  for (const toolId of productModule.primaryTools) {
    ids.add(normalizeCadToolId(toolId))
  }

  for (const phase of CAD_MODULE_ROADMAP_DEFINITIONS[moduleId].workflow) {
    for (const toolId of phase.tools) {
      ids.add(normalizeCadToolId(toolId))
    }
  }

  return [...ids].map((id) => TOOL_BY_ID.get(id)).filter(Boolean) as CadToolDefinition[]
}

export function listCadToolsForClinicalModule(moduleId: TlantiModuleId): readonly CadToolDefinition[] {
  const moduleDefinition = TLANTI_MODULE_DEFINITIONS[moduleId]
  return moduleDefinition.tools.map((id) => resolveCadToolDefinition(id)).filter(Boolean) as CadToolDefinition[]
}

export function listCadToolsForModule(moduleId: TlantiCadProductModuleId | TlantiModuleId): readonly CadToolDefinition[] {
  if ((TLANTI_CAD_PRODUCT_MODULE_IDS as readonly string[]).includes(moduleId)) {
    return listCadToolsForProductModule(moduleId as TlantiCadProductModuleId)
  }

  return listCadToolsForClinicalModule(moduleId as TlantiModuleId)
}

export function validateCadToolPlatform(): string[] {
  const issues: string[] = []
  const seen = new Set<string>()

  for (const definition of CAD_TOOL_DEFINITIONS) {
    if (seen.has(definition.id)) issues.push(`Duplicate tool id ${definition.id}`)
    seen.add(definition.id)
    if (!definition.commandId.trim()) issues.push(`${definition.id} missing commandId`)
    if (!definition.label.trim()) issues.push(`${definition.id} missing label`)
    if (!definition.owner.trim()) issues.push(`${definition.id} missing owner`)
    if (!definition.runtime.trim()) issues.push(`${definition.id} missing runtime`)
    if (!definition.status.trim()) issues.push(`${definition.id} missing status`)
    if (!definition.placements.length) issues.push(`${definition.id} missing placements`)
    if (definition.status === 'ready' && !definition.capabilities.length) issues.push(`${definition.id} ready tool missing capabilities`)
    if (definition.status === 'ready' && ['rust', 'tauri', 'python'].includes(definition.runtime) && !definition.runtimeCommand?.trim()) {
      issues.push(`${definition.id} ready ${definition.runtime} tool missing runtimeCommand`)
    }
  }

  for (const [alias, target] of Object.entries(CAD_TOOL_LEGACY_ALIASES)) {
    if (!TOOL_BY_ID.has(target)) issues.push(`Alias ${alias} points to missing tool ${target}`)
  }

  for (const [moduleId, definition] of Object.entries(TLANTI_MODULE_DEFINITIONS)) {
    for (const toolId of definition.tools) {
      if (!resolveCadToolDefinition(toolId)) issues.push(`${moduleId} references missing clinical tool ${toolId}`)
    }
  }

  for (const moduleId of TLANTI_CAD_PRODUCT_MODULE_IDS) {
    const productModule = CAD_PRODUCT_MODULE_DEFINITIONS[moduleId]
    for (const toolId of productModule.primaryTools) {
      if (!resolveCadToolDefinition(toolId)) issues.push(`${moduleId} references missing primary tool ${toolId}`)
    }

    for (const phase of CAD_MODULE_ROADMAP_DEFINITIONS[moduleId].workflow) {
      for (const toolId of phase.tools) {
        if (!resolveCadToolDefinition(toolId)) issues.push(`${moduleId}/${phase.id} references missing roadmap tool ${toolId}`)
      }
    }
  }

  return issues
}

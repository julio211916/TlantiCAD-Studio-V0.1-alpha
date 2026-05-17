import type { TlantiModuleId } from './entities'

export type AssetLibraryRootId =
  | 'icons'
  | 'library'
  | 'library-controls'
  | 'predefined-elements'
  | 'operation-bitmaps'

export type AssetRuntimeOwner = 'react-ui' | 'tauri-resource' | 'rust-core' | 'python-backend'

export type AssetPlacementId =
  | 'shell-icons'
  | 'dental-icons'
  | 'worktype-icons'
  | 'controls-gizmos'
  | 'articulator-library'
  | 'implant-library'
  | 'model-creator-library'
  | 'production-library'
  | 'feature-replicas-library'
  | 'smile-design-library'
  | 'visualizer-library'
  | 'ceph-predefined-elements'
  | 'ceph-operation-bitmaps'

export interface AssetLibraryRootDefinition {
  id: AssetLibraryRootId
  sourcePath: string
  stableTargetPath: string
  legacyMirrorPath?: string
  owner: AssetRuntimeOwner
  purpose: string
  risk: string
}

export interface AssetPlacementDefinition {
  id: AssetPlacementId
  rootId: AssetLibraryRootId
  label: string
  subpath?: string
  modules: readonly TlantiModuleId[]
  uiSurface:
    | 'shell'
    | 'module-launcher'
    | 'workflow-help'
    | 'cad-gizmo'
    | 'resource-catalog'
    | 'ceph-landmarks'
    | 'visualizer'
  workflowUsage: readonly string[]
  runtimeRule: string
}

export interface ModuleAssetUsageDefinition {
  moduleId: TlantiModuleId
  roots: readonly AssetLibraryRootId[]
  workflowUsage: readonly string[]
  runtimeRule: string
}

export const ASSET_LIBRARY_ROOTS = {
  icons: {
    id: 'icons',
    sourcePath: 'public/icons',
    stableTargetPath: 'public/icons',
    legacyMirrorPath: 'dist/icons',
    owner: 'react-ui',
    purpose: 'Canonical vector/raster icon pack for shell actions, status chips, module launchers and workflow affordances.',
    risk: 'Large legacy mirror still exists under dist; keep explicit icon maps and avoid wildcard imports into React.',
  },
  library: {
    id: 'library',
    sourcePath: 'public/library',
    stableTargetPath: 'public/library',
    legacyMirrorPath: 'dist/library',
    owner: 'tauri-resource',
    purpose: 'Clinical CAD libraries: tooth sets, implant parts, visualizers, bridge splitters and Smile Design assets.',
    risk: 'Large mixed library; index and load lazily per module, never import recursively into the initial React bundle.',
  },
  'library-controls': {
    id: 'library-controls',
    sourcePath: 'public/library/controls',
    stableTargetPath: 'public/library/controls',
    legacyMirrorPath: 'dist/library/controls',
    owner: 'rust-core',
    purpose: '3D interaction controls for rotate, scale, implant scale and cut view handles.',
    risk: 'STL/OFF controls must be loaded through the CAD engine or asset protocol to avoid UI thread parsing.',
  },
  'predefined-elements': {
    id: 'predefined-elements',
    sourcePath: 'public/PredefinedElements',
    stableTargetPath: 'public/PredefinedElements',
    legacyMirrorPath: 'dist/PredefinedElements',
    owner: 'python-backend',
    purpose: 'Ceph predefined landmark/trace/media elements and guided cefalometry resources.',
    risk: 'High file count; Ceph must lazy-index this folder and avoid scanning during shell startup.',
  },
  'operation-bitmaps': {
    id: 'operation-bitmaps',
    sourcePath: 'public/Bitmaps/Operations',
    stableTargetPath: 'public/Bitmaps/Operations',
    legacyMirrorPath: 'dist/Bitmaps/Operations',
    owner: 'react-ui',
    purpose: 'Step illustrations for surgical/orthodontic operation workflows and operator guidance.',
    risk: 'JPG assets are useful for step help, but must be decoded on demand to avoid RAM spikes.',
  },
} as const satisfies Record<AssetLibraryRootId, AssetLibraryRootDefinition>

export const ASSET_PLACEMENTS: readonly AssetPlacementDefinition[] = [
  {
    id: 'shell-icons',
    rootId: 'icons',
    label: 'Shell and workspace icons',
    modules: ['cad', 'dicom', 'model-creator', 'partials', 'implant', 'guide', 'splint', 'ceph', 'fab', 'aligners', 'orthocad'],
    uiSurface: 'shell',
    workflowUsage: ['workspace switcher', 'file menu', 'status chips', 'jobs panel'],
    runtimeRule: 'Resolve from public/icons via explicit icon names or manifest DTOs; never glob-import the full folder into React.',
  },
  {
    id: 'dental-icons',
    rootId: 'icons',
    label: 'Dental glyphs and tooth state icons',
    subpath: 'dental',
    modules: ['cad', 'model-creator', 'partials', 'implant', 'guide', 'splint', 'aligners', 'orthocad'],
    uiSurface: 'module-launcher',
    workflowUsage: ['tooth selection', 'restoration type badges', 'case navigation'],
    runtimeRule: 'Treat as 2D UI glyphs only; keep the filename mapping in core DTOs instead of component-local string literals.',
  },
  {
    id: 'worktype-icons',
    rootId: 'icons',
    label: 'Worktype and indication icons',
    subpath: 'worktype',
    modules: ['cad', 'model-creator', 'partials', 'implant', 'guide', 'splint', 'fab', 'aligners', 'orthocad'],
    uiSurface: 'module-launcher',
    workflowUsage: ['case intake', 'module cards', 'manufacturing route selection'],
    runtimeRule: 'Map worktype ids to filenames in the shell/core and hydrate the final icon URL through the asset registry.',
  },
  {
    id: 'controls-gizmos',
    rootId: 'library-controls',
    label: '3D gizmos and splitter controls',
    modules: ['cad', 'model-creator', 'implant', 'guide', 'splint', 'aligners'],
    uiSurface: 'cad-gizmo',
    workflowUsage: ['move/rotate/scale', 'cut view', 'before/after splitter'],
    runtimeRule: 'Load STL/OFF controls through Rust/Three adapters after module activation; never parse them during the initial React render path.',
  },
  {
    id: 'articulator-library',
    rootId: 'library',
    label: 'Articulator presets and jaw review assets',
    subpath: 'articulator',
    modules: ['cad', 'splint', 'guide'],
    uiSurface: 'resource-catalog',
    workflowUsage: ['jaw motion review', 'occlusion', 'articulator presets'],
    runtimeRule: 'Catalog and stream lazily; pair previews with JawMotion DTOs instead of embedding articulator geometry in React state.',
  },
  {
    id: 'implant-library',
    rootId: 'library',
    label: 'Implant, sleeve and planning catalogs',
    subpath: 'implant',
    modules: ['implant', 'guide', 'fab'],
    uiSurface: 'resource-catalog',
    workflowUsage: ['implant planning', 'sleeve selection', 'drill kit handoff'],
    runtimeRule: 'Index by manufacturer/platform in Rust/Tauri and expose filtered DTOs; React must not crawl the folder tree directly.',
  },
  {
    id: 'model-creator-library',
    rootId: 'library',
    label: 'Model Creator templates and manufacturing helpers',
    subpath: 'modelcreator',
    modules: ['model-creator', 'cad', 'fab'],
    uiSurface: 'resource-catalog',
    workflowUsage: ['base templates', 'trim helpers', 'label presets'],
    runtimeRule: 'Treat as manufacturing resources: send only preview metadata first and load heavy geometry on demand.',
  },
  {
    id: 'production-library',
    rootId: 'library',
    label: 'Production, nesting and fabrication assets',
    subpath: 'production',
    modules: ['fab', 'guide', 'implant'],
    uiSurface: 'resource-catalog',
    workflowUsage: ['CAM packaging', 'material routing', 'printer/mill output prep'],
    runtimeRule: 'Keep machine/vendor specifics outside the initial bundle and expose them through Tauri resource manifests.',
  },
  {
    id: 'feature-replicas-library',
    rootId: 'library',
    label: 'Feature replica CAD meshes',
    subpath: 'feature-replicas',
    modules: ['cad', 'model-creator', 'partials', 'implant', 'guide', 'splint', 'fab', 'aligners', 'orthocad'],
    uiSurface: 'resource-catalog',
    workflowUsage: ['waxup', 'denture', 'alignment', 'tooth library', 'model helpers', 'training parity'],
    runtimeRule: 'Use as local CAD reference assets only: index manifest metadata first, preview STL/OBJ/PLY lazily, never execute imported scripts or native binaries.',
  },
  {
    id: 'smile-design-library',
    rootId: 'library',
    label: 'Smile design overlays and dental preview assets',
    subpath: 'smiledesign',
    modules: ['orthocad', 'cad', 'model-creator'],
    uiSurface: 'visualizer',
    workflowUsage: ['photo mockup', 'waxup preview', 'tooth library handoff'],
    runtimeRule: 'Keep photo and waxup overlays in the UI, but route any AI generation or heavy transforms through Python/Rust jobs.',
  },
  {
    id: 'visualizer-library',
    rootId: 'library',
    label: 'Visualizers and presentation assets',
    subpath: 'visualizers',
    modules: ['cad', 'dicom', 'implant', 'orthocad'],
    uiSurface: 'visualizer',
    workflowUsage: ['3D preview', 'material preview', 'presentation'],
    runtimeRule: 'Mount only after the viewer route is active and cache preview materials separately from case assets.',
  },
  {
    id: 'ceph-predefined-elements',
    rootId: 'predefined-elements',
    label: 'Ceph landmarks, cards and audio prompts',
    modules: ['ceph'],
    uiSurface: 'ceph-landmarks',
    workflowUsage: ['landmark picking', 'trace cards', 'measurement guidance'],
    runtimeRule: 'Lazy-index XML, PNG and MP3 by name; do not scan the full tree during shell startup or module boot.',
  },
  {
    id: 'ceph-operation-bitmaps',
    rootId: 'operation-bitmaps',
    label: 'Ceph and ortho operation step images',
    modules: ['ceph', 'orthocad'],
    uiSurface: 'workflow-help',
    workflowUsage: ['operation explainer', 'before/after guidance', 'report context'],
    runtimeRule: 'Decode JPGs on demand near the active workflow step and release image memory when the user leaves the module.',
  },
] as const

export const MODULE_ASSET_USAGE = {
  cad: {
    moduleId: 'cad',
    roots: ['icons', 'library', 'library-controls'],
    workflowUsage: ['Import', 'Clean', 'Segment', 'Design', 'Validate', 'Export'],
    runtimeRule: 'CAD Design resolves library metadata first, then streams meshes through Rust/Three loaders on demand.',
  },
  dicom: {
    moduleId: 'dicom',
    roots: ['icons'],
    workflowUsage: ['Load', 'Metadata', 'MPR', 'Align scan', 'Segment', 'Handoff'],
    runtimeRule: 'DICOM Viewer uses icons in React, but DICOM files flow through Tauri/Python PyDICOM services.',
  },
  'model-creator': {
    moduleId: 'model-creator',
    roots: ['icons', 'library-controls', 'operation-bitmaps'],
    workflowUsage: ['Import scan', 'Orient', 'Base', 'Hollow/Drain', 'Label', 'Validate'],
    runtimeRule: 'Model Creator loads controls and operation imagery only after the module route is active.',
  },
  partials: {
    moduleId: 'partials',
    roots: ['icons', 'library', 'library-controls'],
    workflowUsage: ['Tooth context', 'Survey', 'Path insertion', 'Framework', 'Clasp', 'Validate'],
    runtimeRule: 'PartialCAD AI should use local library meshes and AI suggestions behind CAD Design toolsets.',
  },
  implant: {
    moduleId: 'implant',
    roots: ['icons', 'library', 'library-controls'],
    workflowUsage: ['CBCT', 'Surface alignment', 'Prosthetic plan', 'Implant', 'Safety', 'Report'],
    runtimeRule: 'Implant libraries are Tauri resources indexed by Rust; React receives filtered catalog DTOs only.',
  },
  guide: {
    moduleId: 'guide',
    roots: ['icons', 'library', 'library-controls'],
    workflowUsage: ['Implant plan', 'Contact surface', 'Sleeve', 'Support', 'Drill protocol', 'Fab'],
    runtimeRule: 'Surgical Guide reuses implant/sleeve assets and writes guide results through AssetStorage.',
  },
  splint: {
    moduleId: 'splint',
    roots: ['icons', 'library-controls'],
    workflowUsage: ['Scan pair', 'Occlusal alignment', 'Offset', 'Contacts', 'Relief', 'Fab'],
    runtimeRule: 'Splint should share CAD controls and keep offset/contact computation in Rust workers.',
  },
  ceph: {
    moduleId: 'ceph',
    roots: ['icons', 'predefined-elements', 'operation-bitmaps'],
    workflowUsage: ['DICOM/lateral', 'Calibrate', 'Landmarks', 'Trace', 'Measurements', 'Report'],
    runtimeRule: 'Ceph owns PredefinedElements and sends images/landmarks to Python PyTorch/PyDICOM pipelines lazily.',
  },
  fab: {
    moduleId: 'fab',
    roots: ['icons', 'library', 'operation-bitmaps'],
    workflowUsage: ['Validate mesh', 'Orient', 'Nest/package', 'CAM profile', 'Report', 'Archive'],
    runtimeRule: 'Fab reads material/manufacturing profiles from resources and writes job packages via Tauri filesystem.',
  },
  aligners: {
    moduleId: 'aligners',
    roots: ['icons', 'library-controls', 'operation-bitmaps'],
    workflowUsage: ['Model setup', 'Segment teeth', 'Movement stages', 'Attachments', 'IPR', 'Export'],
    runtimeRule: 'Aligners loads staging assets after setup and delegates segmentation/inference to local AI backends.',
  },
  orthocad: {
    moduleId: 'orthocad',
    roots: ['icons', 'library', 'operation-bitmaps'],
    workflowUsage: ['Photos', '2D/3D align', 'Tooth library', 'Waxup', 'Preview', 'Handoff'],
    runtimeRule: 'Smile & Ortho uses Smile Design assets lazily and hands generated waxups back to CAD Design.',
  },
} as const satisfies Record<TlantiModuleId, ModuleAssetUsageDefinition>

export function listAssetLibraryRoots(): readonly AssetLibraryRootDefinition[] {
  return Object.values(ASSET_LIBRARY_ROOTS)
}

export function listAssetUsageForModule(moduleId: TlantiModuleId): ModuleAssetUsageDefinition {
  return MODULE_ASSET_USAGE[moduleId]
}

export function listAssetPlacements(): readonly AssetPlacementDefinition[] {
  return ASSET_PLACEMENTS
}

export function listAssetPlacementsForModule(moduleId: TlantiModuleId): readonly AssetPlacementDefinition[] {
  return ASSET_PLACEMENTS.filter((placement) => placement.modules.includes(moduleId))
}

import type { TlantiModuleId } from './entities'

export type TlantiCadProductModuleId =
  | 'tlanticad-crown'
  | 'tlanticad-implant'
  | 'tlanticad-bridge'
  | 'tlanticad-waxup'
  | 'tlanticad-freeform'
  | 'tlanticad-abutment'
  | 'tlanticad-model'
  | 'tlanticad-bar'
  | 'tlanticad-telescope'
  | 'tlanticad-bite-splint'

export interface TlantiCadProductModuleDefinition {
  id: TlantiCadProductModuleId
  libraryWeight: number
  label: string
  shortLabel: string
  purpose: string
  routeModuleId: TlantiModuleId
  requiredAssets: readonly string[]
  workflowSteps: readonly string[]
  primaryTools: readonly string[]
  jobTypes: readonly string[]
  outputAssets: readonly string[]
}

export const CAD_PRODUCT_MODULE_DEFINITIONS = {
  'tlanticad-crown': {
    id: 'tlanticad-crown',
    libraryWeight: 1160,
    label: 'Crown',
    shortLabel: 'Crown',
    purpose: 'Parametric crown design',
    routeModuleId: 'cad',
    requiredAssets: ['prep-scan', 'antagonist-scan', 'bite-scan'],
    workflowSteps: ['Import scan', 'Detect prep', 'Margin', 'Insertion axis', 'Anatomy proposal', 'Contacts/occlusion', 'Thickness validation', 'Export'],
    primaryTools: ['select', 'margin', 'axis', 'sculpt', 'contacts', 'thickness', 'manufacturing-export'],
    jobTypes: ['mesh-repair', 'margin-detection', 'contact-analysis', 'thickness-map'],
    outputAssets: ['crown-stl', 'case-xml', 'manufacturing-report'],
  },
  'tlanticad-implant': {
    id: 'tlanticad-implant',
    libraryWeight: 996,
    label: 'Implant',
    shortLabel: 'Implant',
    purpose: 'Implant planning + Post and Core restorative workflow',
    routeModuleId: 'implant',
    requiredAssets: ['cbct-series', 'surface-scan', 'implant-library'],
    workflowSteps: ['Load CBCT/STL', 'Registration', 'Nerve/sinus planning', 'Implant library / work type', 'Prosthetic axis', 'Post and Core / sleeve/guide', 'Report/export'],
    primaryTools: ['implant-planning', 'implant-library', 'material-config', 'scan-body', 'implant-measure', 'axis', 'guide-wizard', 'manufacturing-export'],
    jobTypes: ['dicom-sanitize', 'dicom-segmentation', 'surface-registration', 'guide-preview', 'post-core-brief'],
    outputAssets: ['implant-plan', 'post-core-plan', 'guide-stl', 'planning-report'],
  },
  'tlanticad-bridge': {
    id: 'tlanticad-bridge',
    libraryWeight: 667,
    label: 'Bridge',
    shortLabel: 'Bridge',
    purpose: 'Bridge design',
    routeModuleId: 'cad',
    requiredAssets: ['prep-scan', 'antagonist-scan', 'tooth-context'],
    workflowSteps: ['Import preps', 'Define span', 'Pontic design', 'Connector sizing', 'Insertion path', 'Contacts', 'Validation/export'],
    primaryTools: ['select', 'axis', 'sculpt', 'contacts', 'thickness', 'boolean', 'manufacturing-export'],
    jobTypes: ['connector-validation', 'contact-analysis', 'thickness-map', 'mesh-boolean'],
    outputAssets: ['bridge-stl', 'case-xml', 'manufacturing-report'],
  },
  'tlanticad-freeform': {
    id: 'tlanticad-freeform',
    libraryWeight: 409,
    label: 'Freeform',
    shortLabel: 'Free',
    purpose: 'Freeform sculpting',
    routeModuleId: 'cad',
    requiredAssets: ['mesh'],
    workflowSteps: ['Import mesh', 'Mask', 'Sculpt/smooth/cut', 'Repair', 'Validate mesh', 'Derived asset'],
    primaryTools: ['select', 'move', 'rotate', 'sculpt', 'repair', 'boolean', 'layers-panel'],
    jobTypes: ['mesh-repair', 'mesh-boolean', 'mesh-smooth', 'asset-derive'],
    outputAssets: ['derived-mesh', 'mesh-manifest'],
  },
  'tlanticad-waxup': {
    id: 'tlanticad-waxup',
    libraryWeight: 371,
    label: 'Waxup',
    shortLabel: 'Waxup',
    purpose: 'Digital waxup + anatomy',
    routeModuleId: 'orthocad',
    requiredAssets: ['scan', 'photos', 'tooth-library'],
    workflowSteps: ['Load scan/photos', 'Tooth library', 'Anatomy edit', 'Smile/occlusion check', 'Mockup', 'Export'],
    primaryTools: ['smile-workflow', 'smile-photos', 'odontogram', 'sculpt', 'contacts', 'manufacturing-export'],
    jobTypes: ['smile-preview', 'tooth-library-placement', 'contact-analysis'],
    outputAssets: ['waxup-stl', 'mockup-preview', 'case-xml'],
  },
  'tlanticad-abutment': {
    id: 'tlanticad-abutment',
    libraryWeight: 352,
    label: 'Abutment',
    shortLabel: 'Abut',
    purpose: 'Abutment generation + post/core restorative controls',
    routeModuleId: 'implant',
    requiredAssets: ['implant-plan', 'implant-platform', 'gingiva-scan'],
    workflowSteps: ['Implant platform', 'Cross-section profile', 'Margin loop', 'Collar/emergence body', 'Surface adaptation', 'Screw channel', 'Cleanup/export'],
    primaryTools: ['implant-library', 'abutment-cross-section', 'abutment-margin-loop', 'abutment-collar', 'abutment-shrinkwrap', 'abutment-screw-channel', 'abutment-cleanup', 'abutment-report'],
    jobTypes: ['implant-platform-resolve', 'abutment-cross-section', 'abutment-margin-loop', 'abutment-collar-body', 'abutment-shrinkwrap', 'abutment-boolean-cut', 'abutment-mesh-cleanup', 'abutment-export-package', 'mesh-offset'],
    outputAssets: ['abutment-stl', 'construction-info-json', 'planning-pdf'],
  },
  'tlanticad-model': {
    id: 'tlanticad-model',
    libraryWeight: 344,
    label: 'Model',
    shortLabel: 'Model',
    purpose: 'Model creation',
    routeModuleId: 'model-creator',
    requiredAssets: ['scan'],
    workflowSteps: ['Import scan', 'Align', 'Base', 'Hollow', 'Label', 'Antagonist prompt', 'Printable export'],
    primaryTools: ['select', 'axis', 'base', 'trim', 'label', 'repair', 'manufacturing-export'],
    jobTypes: ['mesh-repair', 'model-base', 'model-hollow', 'asset-derive'],
    outputAssets: ['model-stl', 'printable-model-report'],
  },
  'tlanticad-bar': {
    id: 'tlanticad-bar',
    libraryWeight: 202,
    label: 'Bar',
    shortLabel: 'Bar',
    purpose: 'Bar / partial denture design',
    routeModuleId: 'partials',
    requiredAssets: ['edentulous-scan', 'implant-positions', 'tooth-context'],
    workflowSteps: ['Implants/edentulous scan', 'Bar path', 'Attachments', 'Relief', 'Manufacturing export'],
    primaryTools: ['segment', 'bar-design', 'axis', 'measure', 'repair', 'manufacturing-export'],
    jobTypes: ['bar-path-preview', 'attachment-validation', 'mesh-offset'],
    outputAssets: ['bar-stl', 'partial-framework-report'],
  },
  'tlanticad-telescope': {
    id: 'tlanticad-telescope',
    libraryWeight: 122,
    label: 'Telescope',
    shortLabel: 'Tele',
    purpose: 'Telescope crowns',
    routeModuleId: 'partials',
    requiredAssets: ['prep-scan', 'insertion-path', 'material-profile'],
    workflowSteps: ['Primary crown', 'Insertion axis', 'Secondary crown', 'Friction/spacing', 'Validation/export'],
    primaryTools: ['select', 'axis', 'telescope-fit', 'measure', 'thickness', 'manufacturing-export'],
    jobTypes: ['spacing-validation', 'friction-fit-preview', 'thickness-map'],
    outputAssets: ['primary-crown-stl', 'secondary-crown-stl', 'fit-report'],
  },
  'tlanticad-bite-splint': {
    id: 'tlanticad-bite-splint',
    libraryWeight: 122,
    label: 'Bite Splint',
    shortLabel: 'Splint',
    purpose: 'Bite splint design',
    routeModuleId: 'splint',
    requiredAssets: ['maxilla-scan', 'mandible-scan', 'bite-scan'],
    workflowSteps: ['Max/mandible/bite', 'Occlusal plane', 'Offset', 'Trim', 'Contacts', 'Export'],
    primaryTools: ['splint-workflow', 'axis', 'offset', 'trim', 'contacts', 'splint-export'],
    jobTypes: ['jaw-motion', 'occlusion-map', 'mesh-offset', 'contact-analysis'],
    outputAssets: ['splint-stl', 'occlusion-report'],
  },
} as const satisfies Record<TlantiCadProductModuleId, TlantiCadProductModuleDefinition>

export const TLANTI_CAD_PRODUCT_MODULE_IDS = Object.keys(CAD_PRODUCT_MODULE_DEFINITIONS) as TlantiCadProductModuleId[]

export function listCadProductModules(): readonly TlantiCadProductModuleDefinition[] {
  return TLANTI_CAD_PRODUCT_MODULE_IDS.map((id) => CAD_PRODUCT_MODULE_DEFINITIONS[id])
}

export function isTlantiCadProductModuleId(value: string): value is TlantiCadProductModuleId {
  return Object.prototype.hasOwnProperty.call(CAD_PRODUCT_MODULE_DEFINITIONS, value)
}

export function resolveCadProductModuleDefinition(moduleId?: string | null): TlantiCadProductModuleDefinition {
  if (moduleId && isTlantiCadProductModuleId(moduleId)) {
    return CAD_PRODUCT_MODULE_DEFINITIONS[moduleId]
  }

  return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-crown']
}

export function resolveCadProductModuleForRoute(routeModuleId?: string | null): TlantiCadProductModuleDefinition {
  switch (routeModuleId) {
    case 'implant':
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-implant']
    case 'model-creator':
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-model']
    case 'partials':
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-bar']
    case 'splint':
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-bite-splint']
    case 'orthocad':
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-waxup']
    default:
      return CAD_PRODUCT_MODULE_DEFINITIONS['tlanticad-crown']
  }
}

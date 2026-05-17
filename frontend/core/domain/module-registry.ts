import type { FileActionDefinition, FileActionId, TlantiModuleDefinition, TlantiModuleId } from './entities'

export const TLANTI_MODULE_DEFINITIONS = {
  cad: {
    id: 'cad',
    label: 'CAD Design',
    shortLabel: 'CAD',
    description: 'Core CAD mesh-first para restauraciones, IA local, edición de mallas y validación.',
    stage: 'design',
    workflow: ['Import', 'Clean', 'Segment', 'Design', 'Validate', 'Export'],
    tools: ['select', 'move', 'rotate', 'scale', 'sculpt', 'measure', 'layers', 'groups', 'command-palette'],
    aiCapabilities: ['segmentacion dental', 'margenes sugeridos', 'deteccion de huecos', 'QA de espesor'],
    isCore: true,
  },
  dicom: {
    id: 'dicom',
    label: 'DICOM Viewer',
    shortLabel: 'DICOM',
    description: 'Viewer CBCT/DICOM con metadata, MPR, mediciones y handoff a CAD/Implant/Ceph.',
    stage: 'imaging',
    workflow: ['Load', 'Metadata', 'MPR', 'Align scan', 'Segment', 'Handoff'],
    tools: ['import-dicom', 'series-browser', 'mpr', 'metadata', 'measure', 'volume-presets'],
    aiCapabilities: ['artefactos', 'landmarks', 'canal mandibular', 'sinus segmentation'],
  },
  'model-creator': {
    id: 'model-creator',
    label: 'Model Creator',
    shortLabel: 'Model',
    description: 'Preparacion de modelos, bases, hollow/drain, labels y validacion de manufactura.',
    stage: 'design',
    workflow: ['Import scan', 'Orient', 'Base', 'Hollow/Drain', 'Label', 'Validate'],
    tools: ['base', 'trim', 'hollow', 'label', 'plane-cut', 'repair'],
    aiCapabilities: ['orientacion auto', 'deteccion de agujeros', 'propuesta de base'],
  },
  partials: {
    id: 'partials',
    label: 'PartialCAD AI',
    shortLabel: 'Partial',
    description: 'Parciales asistidos por IA local: survey, blockout, connectors, clasps y override manual.',
    stage: 'design',
    workflow: ['Tooth context', 'Survey', 'Path insertion', 'Framework', 'Clasp', 'Validate'],
    tools: ['blockout', 'clasp', 'major-connector', 'minor-connector', 'relief', 'mesh-checks'],
    aiCapabilities: ['clasp sugerido', 'connector sugerido', 'zonas retentivas', 'blockout inicial'],
  },
  implant: {
    id: 'implant',
    label: 'Implant',
    shortLabel: 'Implant',
    description: 'Planeacion implantologica, abutments y Post and Core con biblioteca local, scan body y zonas de seguridad.',
    stage: 'design',
    workflow: ['CBCT / Scan', 'Surface alignment', 'Prosthetic plan', 'Implant / Post and Core', 'Safety', 'Report'],
    tools: ['implant-library', 'material-config', 'scan-body', 'nerve-sinus', 'measure', 'trajectory', 'sleeve-preview'],
    aiCapabilities: ['canal mandibular', 'densidad/riesgo', 'posicion protesica sugerida', 'sugerencia de workflow post and core'],
  },
  guide: {
    id: 'guide',
    label: 'Surgical Guide',
    shortLabel: 'Guide',
    description: 'Guia quirurgica con superficie de contacto, sleeves, pins, booleans y reporte.',
    stage: 'manufacture',
    workflow: ['Implant plan', 'Contact surface', 'Sleeve', 'Support', 'Drill protocol', 'Fab'],
    tools: ['guide-wizard', 'sleeve', 'fixation-pins', 'undercut', 'export'],
    aiCapabilities: ['colisiones', 'espesor minimo', 'riesgo de manufactura'],
  },
  splint: {
    id: 'splint',
    label: 'Splint',
    shortLabel: 'Splint',
    description: 'Ferulas con alineacion oclusal, offset, contactos, alivios, trim y export.',
    stage: 'design',
    workflow: ['Scan pair', 'Occlusal alignment', 'Offset', 'Contacts', 'Relief', 'Fab'],
    tools: ['splint-workflow', 'occlusion-map', 'offset', 'trim', 'export'],
    aiCapabilities: ['mapa de contactos', 'interferencias', 'alivios sugeridos'],
  },
  ceph: {
    id: 'ceph',
    label: 'Ceph',
    shortLabel: 'Ceph',
    description: 'Cefalometria local con calibracion, landmarks, trazados, medidas y reporte.',
    stage: 'review',
    workflow: ['DICOM/lateral', 'Calibrate', 'Landmarks', 'Trace', 'Measurements', 'Report'],
    tools: ['landmark-editor', 'ceph-overlay', 'measurement-table', 'report'],
    aiCapabilities: ['landmarks cefalometricos', 'score de confianza'],
  },
  fab: {
    id: 'fab',
    label: 'Fab',
    shortLabel: 'Fab',
    description: 'Validacion y empaquetado CAM local: STL/3MF, perfiles de material y reporte.',
    stage: 'manufacture',
    workflow: ['Validate mesh', 'Orient', 'Nest/package', 'CAM profile', 'Report', 'Archive'],
    tools: ['stl-3mf', 'repair', 'naming', 'material-profile', 'job-package'],
    aiCapabilities: ['QA manufactura', 'print readiness', 'mill readiness'],
  },
  aligners: {
    id: 'aligners',
    label: 'Aligners',
    shortLabel: 'Align',
    description: 'Setup de alineadores con segmentacion dental, stages, attachments, IPR y export.',
    stage: 'design',
    workflow: ['Model setup', 'Segment teeth', 'Movement stages', 'Attachments', 'IPR', 'Export'],
    tools: ['staging', 'attachments', 'ipr-map', 'collision-map', 'export'],
    aiCapabilities: ['segmentacion dental', 'staging inicial', 'riesgo de colision'],
  },
  orthocad: {
    id: 'orthocad',
    label: 'Smile & Ortho',
    shortLabel: 'Smile',
    description: 'Fotos, alineacion 2D/3D, libreria dental, waxup, preview y handoff a CAD.',
    stage: 'review',
    workflow: ['Photos', '2D/3D align', 'Tooth library', 'Waxup', 'Preview', 'Handoff'],
    tools: ['photo-align', 'smile-lines', 'tooth-library', 'mockup'],
    aiCapabilities: ['smile preview local', 'proporcion dental sugerida'],
  },
} as const satisfies Record<TlantiModuleId, TlantiModuleDefinition>

export const TLANTI_MODULE_IDS = Object.keys(TLANTI_MODULE_DEFINITIONS) as TlantiModuleId[]

export const FILE_ACTION_DEFINITIONS = {
  export: {
    id: 'export',
    label: 'Export',
    description: 'Exportar STL/OBJ/PLY/3MF, paquete de caso o reporte desde menu de archivos.',
  },
  'xml-interop': {
    id: 'xml-interop',
    label: 'XML interop',
    description: 'Generar interoperabilidad XML local sin convertirlo en modulo clinico.',
  },
  'folder-reveal': {
    id: 'folder-reveal',
    label: 'Reveal folder',
    description: 'Abrir carpeta local del caso desde el menu de archivos.',
  },
  snapshot: {
    id: 'snapshot',
    label: 'Snapshot',
    description: 'Capturar estado visual/clinico del caso.',
  },
  print: {
    id: 'print',
    label: 'Print',
    description: 'Enviar reporte o snapshot a impresion local.',
  },
  copy: {
    id: 'copy',
    label: 'Copy',
    description: 'Copiar assets o datos del caso como accion de archivo.',
  },
} as const satisfies Record<FileActionId, FileActionDefinition>

export function isTlantiModuleId(value: string): value is TlantiModuleId {
  return Object.prototype.hasOwnProperty.call(TLANTI_MODULE_DEFINITIONS, value)
}

export function isFileActionId(value: string): value is FileActionId {
  return Object.prototype.hasOwnProperty.call(FILE_ACTION_DEFINITIONS, value)
}

export function resolveTlantiModuleDefinition(moduleId?: string | null): TlantiModuleDefinition {
  if (!moduleId || !isTlantiModuleId(moduleId)) {
    return TLANTI_MODULE_DEFINITIONS.cad
  }

  return TLANTI_MODULE_DEFINITIONS[moduleId]
}

export function listTlantiModules(): readonly TlantiModuleDefinition[] {
  return TLANTI_MODULE_IDS.map((id) => TLANTI_MODULE_DEFINITIONS[id])
}

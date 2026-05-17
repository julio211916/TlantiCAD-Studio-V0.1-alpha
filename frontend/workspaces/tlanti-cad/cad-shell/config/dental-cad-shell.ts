import { createElement, type ComponentType } from 'react';
import type { TlantiCadProductModuleId, TlantiModuleId } from '@/core';
import { AppIcon, type AppIconName } from '@/features/app-icons';
import {
  ActivitySquare,
  Braces,
  BrainCircuit,
  ClipboardList,
  Factory,
  FolderTree,
  Layers3,
  Maximize2,
  Move,
  MousePointer2,
  PencilRuler,
  RotateCw,
  ScanSearch,
  ScanFace,
  Smile,
  Sparkles,
  Syringe,
  Target,
  Wand2,
} from 'lucide-react';

export type DentalCadShellModuleId = TlantiModuleId | 'layers';
export type DentalCadShellIcon = ComponentType<{ className?: string; size?: number }>;

function tlantiIcon(name: AppIconName): DentalCadShellIcon {
  const TlantiCadShellIcon = ({ className, size = 20 }: { className?: string; size?: number }) =>
    createElement(AppIcon, {
      name,
      className,
      size,
      'aria-hidden': true,
    });

  TlantiCadShellIcon.displayName = `TlantiCadShellIcon(${name})`;
  return TlantiCadShellIcon;
}

export interface DentalCadShellModuleItem {
  id: DentalCadShellModuleId;
  label: string;
  shortLabel: string;
  icon: DentalCadShellIcon;
  description: string;
}

export type DentalCadShellActionId =
  | 'select'
  | 'move'
  | 'rotate'
  | 'scale'
  | 'measure'
  | 'sculpt'
  | 'segment'
  | 'margin'
  | 'axis'
  | 'contacts'
  | 'thickness'
  | 'repair'
  | 'offset'
  | 'trim'
  | 'base'
  | 'label'
  | 'boolean'
  | 'implant-library'
  | 'abutment-design'
  | 'abutment-cross-section'
  | 'abutment-margin-loop'
  | 'abutment-collar'
  | 'abutment-shrinkwrap'
  | 'abutment-screw-channel'
  | 'abutment-cleanup'
  | 'abutment-report'
  | 'bar-design'
  | 'telescope-fit'
  | 'manufacturing-export'
  | 'dicom-import'
  | 'dicom-mpr'
  | 'dicom-metadata'
  | 'smile-workflow'
  | 'smile-photos'
  | 'odontogram'
  | 'implant-planning'
  | 'implant-measure'
  | 'guide-wizard'
  | 'guide-export'
  | 'splint-workflow'
  | 'splint-export'
  | 'layers-panel'
  | 'groups-panel'
  | 'voice-copilot'
  | 'command-palette';

export interface DentalCadShellActionItem {
  id: DentalCadShellActionId;
  label: string;
  shortLabel: string;
  icon: DentalCadShellIcon;
  description: string;
}

export const DENTAL_CAD_SHELL_MODULES: DentalCadShellModuleItem[] = [
  {
    id: 'cad',
    label: 'CAD Design',
    shortLabel: 'CAD',
    icon: tlantiIcon('module.crown-bridge'),
    description: 'Core CAD: diseño restaurativo, IA local, edición de mallas y validación.',
  },
  {
    id: 'dicom',
    label: 'DICOM Viewer',
    shortLabel: 'DICOM',
    icon: tlantiIcon('module.dicom-viewer'),
    description: 'Viewer CBCT/DICOM con MPR, metadata clínica y navegación axial.',
  },
  {
    id: 'orthocad',
    label: 'Smile & Ortho',
    shortLabel: 'Smile',
    icon: tlantiIcon('module.smile-design'),
    description: 'Smile design, ortodoncia, waxup y validación estética.',
  },
  {
    id: 'implant',
    label: 'Implant',
    shortLabel: 'Implant',
    icon: tlantiIcon('module.implant-planning'),
    description: 'Planificación implantológica y relación protésica.',
  },
  {
    id: 'model-creator',
    label: 'Model Creator',
    shortLabel: 'Model',
    icon: tlantiIcon('module.model-creator'),
    description: 'Orientación, bases, hollow, labels y validación de modelos.',
  },
  {
    id: 'guide',
    label: 'Guide',
    shortLabel: 'Guide',
    icon: tlantiIcon('module.surgical-guide'),
    description: 'Guías quirúrgicas y manufactura clínica.',
  },
  {
    id: 'splint',
    label: 'Splint',
    shortLabel: 'Splint',
    icon: tlantiIcon('module.splint'),
    description: 'Férulas, oclusión y ajuste clínico.',
  },
  {
    id: 'partials',
    label: 'PartialCAD AI',
    shortLabel: 'Partial',
    icon: Layers3,
    description: 'Estructuras parciales asistidas por IA local dentro del core CAD.',
  },
  {
    id: 'ceph',
    label: 'Ceph',
    shortLabel: 'Ceph',
    icon: ScanFace,
    description: 'Cefalometría, landmarks y trazados ortodónticos.',
  },
  {
    id: 'fab',
    label: 'Fab',
    shortLabel: 'Fab',
    icon: tlantiIcon('module.cam-fabrication'),
    description: 'Preparación CAM y validación de manufactura local.',
  },
  {
    id: 'aligners',
    label: 'Aligners',
    shortLabel: 'Align',
    icon: tlantiIcon('module.ortho-aligners'),
    description: 'Setup de alineadores, stages, attachments e IPR.',
  },
  {
    id: 'layers',
    label: 'Layers',
    shortLabel: 'Layers',
    icon: Layers3,
    description: 'Capas, grupos y organización de escena.',
  },
];

export const DENTAL_CAD_SHELL_ACTIONS: Record<DentalCadShellActionId, DentalCadShellActionItem> = {
  select: {
    id: 'select',
    label: 'Select',
    shortLabel: 'Select',
    icon: MousePointer2,
    description: 'Selección y foco de objetos clínicos.',
  },
  move: {
    id: 'move',
    label: 'Move',
    shortLabel: 'Move',
    icon: Move,
    description: 'Reposicionamiento del activo actual.',
  },
  rotate: {
    id: 'rotate',
    label: 'Rotate',
    shortLabel: 'Rotate',
    icon: RotateCw,
    description: 'Ajuste angular del activo actual.',
  },
  scale: {
    id: 'scale',
    label: 'Scale',
    shortLabel: 'Scale',
    icon: Maximize2,
    description: 'Escalado clínico del activo actual.',
  },
  measure: {
    id: 'measure',
    label: 'Measure',
    shortLabel: 'Measure',
    icon: tlantiIcon('tool.measurement'),
    description: 'Mediciones lineales y validación espacial.',
  },
  sculpt: {
    id: 'sculpt',
    label: 'Sculpt',
    shortLabel: 'Sculpt',
    icon: tlantiIcon('tool.freeforming'),
    description: 'Esculpido y refinado restaurativo.',
  },
  segment: {
    id: 'segment',
    label: 'Segment',
    shortLabel: 'Segment',
    icon: BrainCircuit,
    description: 'Segmentación y asistencia AI local.',
  },
  margin: {
    id: 'margin',
    label: 'Margin',
    shortLabel: 'Margin',
    icon: tlantiIcon('workflow.margin-detect'),
    description: 'Marcado y revisión de margen restaurativo.',
  },
  axis: {
    id: 'axis',
    label: 'Insertion axis',
    shortLabel: 'Axis',
    icon: tlantiIcon('tool.insertion-direction'),
    description: 'Eje de inserción, orientación protésica y plano clínico.',
  },
  contacts: {
    id: 'contacts',
    label: 'Contacts',
    shortLabel: 'Contact',
    icon: ActivitySquare,
    description: 'Contactos proximales, oclusión y clearance.',
  },
  thickness: {
    id: 'thickness',
    label: 'Thickness',
    shortLabel: 'Thick',
    icon: tlantiIcon('tool.minimum-thickness'),
    description: 'Mapa de espesor mínimo y manufacturabilidad.',
  },
  repair: {
    id: 'repair',
    label: 'Repair',
    shortLabel: 'Repair',
    icon: Wand2,
    description: 'Reparación de malla, holes y manifold checks.',
  },
  offset: {
    id: 'offset',
    label: 'Offset',
    shortLabel: 'Offset',
    icon: Maximize2,
    description: 'Offset, shell y alivios controlados.',
  },
  trim: {
    id: 'trim',
    label: 'Trim',
    shortLabel: 'Trim',
    icon: PencilRuler,
    description: 'Recorte clínico por curva/plano.',
  },
  base: {
    id: 'base',
    label: 'Base',
    shortLabel: 'Base',
    icon: FolderTree,
    description: 'Base de modelo, hollow, drain y soporte.',
  },
  label: {
    id: 'label',
    label: 'Label',
    shortLabel: 'Label',
    icon: ClipboardList,
    description: 'Etiquetado de modelo/caso y marcas de manufactura.',
  },
  boolean: {
    id: 'boolean',
    label: 'Boolean',
    shortLabel: 'Bool',
    icon: Layers3,
    description: 'Boolean cut/union/intersection mediante job backend.',
  },
  'implant-library': {
    id: 'implant-library',
    label: 'Implant library',
    shortLabel: 'Library',
    icon: tlantiIcon('tool.select-implant-type'),
    description: 'Biblioteca local de implantes, sleeves y plataformas.',
  },
  'abutment-design': {
    id: 'abutment-design',
    label: 'Abutment',
    shortLabel: 'Abut',
    icon: tlantiIcon('tool.abutment-design'),
    description: 'Emergence profile, screw channel y cement gap.',
  },
  'abutment-cross-section': {
    id: 'abutment-cross-section',
    label: 'Cross section',
    shortLabel: 'Profile',
    icon: tlantiIcon('tool.crown-bottom'),
    description: 'Perfil Cross_Section/CURVE paramétrico para el abutment.',
  },
  'abutment-margin-loop': {
    id: 'abutment-margin-loop',
    label: 'Margin loop',
    shortLabel: 'Margin',
    icon: tlantiIcon('tool.emergence-profile'),
    description: 'Loop de margen/emergencia persistente como curva clínica.',
  },
  'abutment-collar': {
    id: 'abutment-collar',
    label: 'Collar body',
    shortLabel: 'Collar',
    icon: tlantiIcon('tool.abutment-design'),
    description: 'Cuerpo collar/free-form desde perfil, margen y emergencia.',
  },
  'abutment-shrinkwrap': {
    id: 'abutment-shrinkwrap',
    label: 'Surface adapt',
    shortLabel: 'Adapt',
    icon: tlantiIcon('tool.freeforming'),
    description: 'Adaptación tipo Shrinkwrap contra gingiva/prep con control points.',
  },
  'abutment-screw-channel': {
    id: 'abutment-screw-channel',
    label: 'Screw channel',
    shortLabel: 'Screw',
    icon: tlantiIcon('tool.select-implant-type'),
    description: 'Canal recto/angulado con boolean y validación de clearances.',
  },
  'abutment-cleanup': {
    id: 'abutment-cleanup',
    label: 'Mesh cleanup',
    shortLabel: 'Clean',
    icon: tlantiIcon('tool.freeforming'),
    description: 'Remesh, weld, smooth, manifold y QA de manufactura.',
  },
  'abutment-report': {
    id: 'abutment-report',
    label: 'Abutment report',
    shortLabel: 'Report',
    icon: tlantiIcon('tool.export-pdf'),
    description: 'STL, construction info y reporte con hashes del caso.',
  },
  'bar-design': {
    id: 'bar-design',
    label: 'Bar path',
    shortLabel: 'Bar',
    icon: Layers3,
    description: 'Bar path, attachments y relief para parcial.',
  },
  'telescope-fit': {
    id: 'telescope-fit',
    label: 'Telescope fit',
    shortLabel: 'Fit',
    icon: Target,
    description: 'Fricción, spacing y ajuste de telescópicas.',
  },
  'manufacturing-export': {
    id: 'manufacturing-export',
    label: 'Manufacturing export',
    shortLabel: 'Export',
    icon: tlantiIcon('tool.export-stl'),
    description: 'Salida STL/XML/report para manufactura offline.',
  },
  'dicom-import': {
    id: 'dicom-import',
    label: 'DICOM intake',
    shortLabel: 'Import',
    icon: tlantiIcon('action.import-asset'),
    description: 'Importación de estudio/serie DICOM.',
  },
  'dicom-mpr': {
    id: 'dicom-mpr',
    label: 'MPR',
    shortLabel: 'MPR',
    icon: tlantiIcon('tool.sectional-view'),
    description: 'Entrada a revisión multiplanar y navegación.',
  },
  'dicom-metadata': {
    id: 'dicom-metadata',
    label: 'Metadata',
    shortLabel: 'Meta',
    icon: tlantiIcon('tool.dicom-control'),
    description: 'Resumen clínico y metadata del estudio.',
  },
  'smile-workflow': {
    id: 'smile-workflow',
    label: 'Workflow',
    shortLabel: 'Workflow',
    icon: tlantiIcon('module.smile-design'),
    description: 'Workflow de smile design y mockup.',
  },
  'smile-photos': {
    id: 'smile-photos',
    label: 'Photos',
    shortLabel: 'Photos',
    icon: tlantiIcon('tool.screenshot'),
    description: 'Referencias foto/retracted/smile.',
  },
  odontogram: {
    id: 'odontogram',
    label: 'Odontogram',
    shortLabel: 'Teeth',
    icon: tlantiIcon('tool.tooth-axes'),
    description: 'Selección y contexto odontológico.',
  },
  'implant-planning': {
    id: 'implant-planning',
    label: 'Planning',
    shortLabel: 'Planning',
    icon: tlantiIcon('module.implant-planning'),
    description: 'Entrada al planeamiento implantológico.',
  },
  'implant-measure': {
    id: 'implant-measure',
    label: 'Bone measure',
    shortLabel: 'Measure',
    icon: tlantiIcon('tool.measurement'),
    description: 'Medición ósea y de trayectoria implantaría.',
  },
  'guide-wizard': {
    id: 'guide-wizard',
    label: 'Guide wizard',
    shortLabel: 'Wizard',
    icon: tlantiIcon('module.surgical-guide'),
    description: 'Stackable/foundation/surgical guide workflow.',
  },
  'guide-export': {
    id: 'guide-export',
    label: 'Guide export',
    shortLabel: 'Export',
    icon: tlantiIcon('tool.export-3mf'),
    description: 'Salida clínica/manufactura del guide.',
  },
  'splint-workflow': {
    id: 'splint-workflow',
    label: 'Splint workflow',
    shortLabel: 'Workflow',
    icon: tlantiIcon('module.splint'),
    description: 'Preparación clínica de férulas y exportable base.',
  },
  'splint-export': {
    id: 'splint-export',
    label: 'Splint export',
    shortLabel: 'Export',
    icon: tlantiIcon('tool.export-stl'),
    description: 'Salida del brief/export de férula.',
  },
  'layers-panel': {
    id: 'layers-panel',
    label: 'Layers',
    shortLabel: 'Layers',
    icon: Layers3,
    description: 'Capas, visibilidad y organización de escena.',
  },
  'groups-panel': {
    id: 'groups-panel',
    label: 'Groups',
    shortLabel: 'Groups',
    icon: FolderTree,
    description: 'Selección de grupos, presets y sets clínicos.',
  },
  'voice-copilot': {
    id: 'voice-copilot',
    label: 'Voice copilot',
    shortLabel: 'Copilot',
    icon: BrainCircuit,
    description: 'Panel de voz y copiloto local.',
  },
  'command-palette': {
    id: 'command-palette',
    label: 'Search',
    shortLabel: 'Search',
    icon: ScanSearch,
    description: 'Palette de acciones clínicas/globales.',
  },
};

export const DENTAL_CAD_MODULE_TOOLSETS: Record<DentalCadShellModuleId, DentalCadShellActionId[]> = {
  cad: ['select', 'move', 'rotate', 'scale', 'measure', 'sculpt', 'layers-panel', 'groups-panel'],
  dicom: ['dicom-import', 'dicom-mpr', 'dicom-metadata', 'measure', 'layers-panel'],
  orthocad: ['smile-workflow', 'smile-photos', 'odontogram', 'sculpt', 'measure', 'voice-copilot'],
  implant: ['implant-planning', 'implant-measure', 'measure', 'layers-panel'],
  'model-creator': ['select', 'move', 'rotate', 'scale', 'measure', 'layers-panel'],
  guide: ['guide-wizard', 'guide-export', 'measure', 'layers-panel'],
  splint: ['splint-workflow', 'splint-export', 'measure', 'layers-panel'],
  partials: ['segment', 'sculpt', 'measure', 'layers-panel', 'voice-copilot'],
  ceph: ['dicom-mpr', 'dicom-metadata', 'measure', 'voice-copilot'],
  fab: ['guide-export', 'splint-export', 'layers-panel', 'command-palette'],
  aligners: ['odontogram', 'sculpt', 'measure', 'layers-panel', 'voice-copilot'],
  layers: ['layers-panel', 'groups-panel', 'command-palette'],
};

export const DENTAL_CAD_PRODUCT_MODULE_TOOLSETS: Record<TlantiCadProductModuleId, DentalCadShellActionId[]> = {
  'tlanticad-crown': ['select', 'margin', 'axis', 'sculpt', 'contacts', 'thickness', 'manufacturing-export'],
  'tlanticad-implant': ['implant-planning', 'implant-library', 'implant-measure', 'axis', 'guide-wizard', 'manufacturing-export'],
  'tlanticad-bridge': ['select', 'axis', 'sculpt', 'contacts', 'thickness', 'boolean', 'manufacturing-export'],
  'tlanticad-waxup': ['smile-workflow', 'smile-photos', 'odontogram', 'sculpt', 'contacts', 'manufacturing-export'],
  'tlanticad-freeform': ['select', 'move', 'rotate', 'sculpt', 'repair', 'boolean', 'layers-panel'],
  'tlanticad-abutment': ['implant-library', 'abutment-cross-section', 'abutment-margin-loop', 'abutment-collar', 'abutment-shrinkwrap', 'abutment-screw-channel', 'abutment-cleanup', 'abutment-report'],
  'tlanticad-model': ['select', 'axis', 'base', 'trim', 'label', 'repair', 'manufacturing-export'],
  'tlanticad-bar': ['segment', 'bar-design', 'axis', 'measure', 'repair', 'manufacturing-export'],
  'tlanticad-telescope': ['select', 'axis', 'telescope-fit', 'measure', 'thickness', 'manufacturing-export'],
  'tlanticad-bite-splint': ['splint-workflow', 'axis', 'offset', 'trim', 'contacts', 'splint-export'],
};

import type { LucideIcon } from 'lucide-react';
import {
  Boxes,
  Crosshair,
  ImagePlus,
  Layers3,
  MousePointer2,
  RotateCw,
  ScanSearch,
  Smile,
  Sparkles,
  Syringe,
  ZoomIn,
} from 'lucide-react';

export type TlantiInterfaceAssetKey =
  | 'new-stl-case'
  | 'new-dicom-case'
  | 'import-images'
  | 'standard-docs'
  | 'implant'
  | 'smile-design'
  | 'fab'
  | 'fab-3d'
  | 'add-material'
  | 'remove-material'
  | 'smooth-material'
  | 'extrusion'
  | 'sculpt-dialog'
  | 'lasso-select'
  | 'zoom-in'
  | 'rotate';

export interface TlantiInterfaceAsset {
  key: TlantiInterfaceAssetKey;
  label: string;
  description: string;
  src: string;
  category: 'project' | 'module' | 'cad-tool' | 'selection';
  fallbackIcon: LucideIcon;
}

export const TLANTI_INTERFACE_ASSETS: Record<TlantiInterfaceAssetKey, TlantiInterfaceAsset> = {
  'new-stl-case': {
    key: 'new-stl-case',
    label: 'New STL case',
    description: 'Create a case from local mesh scan data.',
    src: '/Graphics/newcast3dstl.svg',
    category: 'project',
    fallbackIcon: Layers3,
  },
  'new-dicom-case': {
    key: 'new-dicom-case',
    label: 'New DICOM case',
    description: 'Create a case from CBCT or DICOM source data.',
    src: '/Graphics/newcast3dfromdicom.svg',
    category: 'project',
    fallbackIcon: ScanSearch,
  },
  'import-images': {
    key: 'import-images',
    label: 'Import images',
    description: 'Attach local clinical photos and shade references.',
    src: '/Graphics/importimages.svg',
    category: 'project',
    fallbackIcon: ImagePlus,
  },
  'standard-docs': {
    key: 'standard-docs',
    label: 'Standard docs',
    description: 'Attach prescriptions, PDFs and office documents.',
    src: '/Graphics/standarddocs.svg',
    category: 'project',
    fallbackIcon: Boxes,
  },
  implant: {
    key: 'implant',
    label: 'Implant',
    description: 'Open implant planning context.',
    src: '/Graphics/implant.svg',
    category: 'module',
    fallbackIcon: Syringe,
  },
  'smile-design': {
    key: 'smile-design',
    label: 'Smile design',
    description: 'Open smile design and esthetic workflow.',
    src: '/Graphics/smiledesign3d.svg',
    category: 'module',
    fallbackIcon: Smile,
  },
  fab: {
    key: 'fab',
    label: 'FAB',
    description: 'Open fabrication context.',
    src: '/Graphics/nemofab.svg',
    category: 'module',
    fallbackIcon: Sparkles,
  },
  'fab-3d': {
    key: 'fab-3d',
    label: 'FAB 3D',
    description: 'Open 3D fabrication context.',
    src: '/Graphics/nemofab3d.svg',
    category: 'module',
    fallbackIcon: Sparkles,
  },
  'add-material': {
    key: 'add-material',
    label: 'Add material',
    description: 'Virtual wax add material brush.',
    src: '/Graphics/Añadir-Material.svg',
    category: 'cad-tool',
    fallbackIcon: Sparkles,
  },
  'remove-material': {
    key: 'remove-material',
    label: 'Remove material',
    description: 'Virtual wax remove material brush.',
    src: '/Graphics/Quitar-Material.svg',
    category: 'cad-tool',
    fallbackIcon: Sparkles,
  },
  'smooth-material': {
    key: 'smooth-material',
    label: 'Smooth material',
    description: 'Smooth sculpted dental surfaces.',
    src: '/Graphics/Suavizar-Material.svg',
    category: 'cad-tool',
    fallbackIcon: Sparkles,
  },
  extrusion: {
    key: 'extrusion',
    label: 'Extrusion',
    description: 'Extrusion setup handle for mesh-first CAD tools.',
    src: '/Graphics/Extrusion.svg',
    category: 'cad-tool',
    fallbackIcon: Crosshair,
  },
  'sculpt-dialog': {
    key: 'sculpt-dialog',
    label: 'Sculpt dialog',
    description: 'Open sculpting controls.',
    src: '/Graphics/Esculpir-Dialogo.svg',
    category: 'cad-tool',
    fallbackIcon: Sparkles,
  },
  'lasso-select': {
    key: 'lasso-select',
    label: 'Lasso selection',
    description: 'Projected dental selection tool.',
    src: '/Graphics/Seleccionar Lasso.svg',
    category: 'selection',
    fallbackIcon: MousePointer2,
  },
  'zoom-in': {
    key: 'zoom-in',
    label: 'Zoom',
    description: 'Zoom into clinical view.',
    src: '/Graphics/zoom_more.svg',
    category: 'selection',
    fallbackIcon: ZoomIn,
  },
  rotate: {
    key: 'rotate',
    label: 'Rotate',
    description: 'Rotate model view.',
    src: '/Graphics/Rotation1.svg',
    category: 'selection',
    fallbackIcon: RotateCw,
  },
};

export function getTlantiInterfaceAsset(key: TlantiInterfaceAssetKey) {
  return TLANTI_INTERFACE_ASSETS[key];
}

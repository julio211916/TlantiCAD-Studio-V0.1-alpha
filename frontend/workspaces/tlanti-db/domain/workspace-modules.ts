import {
  Activity,
  Factory,
  Layers,
  MoonStar,
  Scan,
  Syringe,
  Target,
  type LucideIcon,
} from 'lucide-react';

import type { TlantiInterfaceAssetKey } from '@/lib/tlanticad-interface-assets';

export type TlantiDbWorkspaceModuleId =
  | 'cad'
  | 'model-creator'
  | 'dicom'
  | 'guide'
  | 'implant'
  | 'splint'
  | 'fab';

export interface TlantiDbWorkspaceModule {
  id: TlantiDbWorkspaceModuleId;
  label: string;
  module: string;
  icon: LucideIcon;
  assetKey?: TlantiInterfaceAssetKey;
}

export const TLANTIDB_WORKSPACE_MODULES: TlantiDbWorkspaceModule[] = [
  { id: 'cad', label: 'Design', module: 'cad', icon: Layers },
  { id: 'model-creator', label: 'Model Creator', module: 'model-creator', icon: Activity },
  { id: 'dicom', label: 'DICOM Viewer', module: 'dicom', icon: Scan, assetKey: 'new-dicom-case' },
  { id: 'guide', label: 'Surgical Guide', module: 'guide', icon: Target },
  { id: 'implant', label: 'Implant', module: 'implant', icon: Syringe, assetKey: 'implant' },
  { id: 'splint', label: 'Splint', module: 'splint', icon: MoonStar },
  { id: 'fab', label: 'FAB', module: 'fab', icon: Factory, assetKey: 'fab' },
];

export function getTlantiDbWorkspaceModule(moduleId: string | null | undefined) {
  return TLANTIDB_WORKSPACE_MODULES.find((item) => item.module === moduleId || item.id === moduleId);
}

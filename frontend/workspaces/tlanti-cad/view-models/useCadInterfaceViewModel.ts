import { useMemo } from 'react';

import type { FileData } from '@/types';
import type { DentalCadShellActionId } from '@/features/cad-shell/config/dental-cad-shell';
import type { TlantiCadProductModuleId } from '@/core';

const MODEL_REQUIRED_ACTIONS = new Set<DentalCadShellActionId>([
  'margin',
  'axis',
  'contacts',
  'thickness',
  'repair',
  'offset',
  'trim',
  'boolean',
  'abutment-design',
  'bar-design',
  'telescope-fit',
  'manufacturing-export',
  'guide-export',
]);

const MODEL_REQUIRED_MODULES = new Set<TlantiCadProductModuleId>([
  'tlanticad-crown',
  'tlanticad-implant',
  'tlanticad-bridge',
  'tlanticad-abutment',
  'tlanticad-bar',
  'tlanticad-telescope',
  'tlanticad-freeform',
]);

export interface UseCadInterfaceViewModelInput {
  files: FileData[];
  selectedFile: FileData | undefined;
}

export function useCadInterfaceViewModel({ files, selectedFile }: UseCadInterfaceViewModelInput) {
  return useMemo(() => {
    const hasSceneContent = files.length > 0;
    const hasModel = files.some((file) => file.type === 'MODEL' || file.type === 'DICOM');
    const activeSelectionLabel = selectedFile ? `${selectedFile.type.toLowerCase()} · ${selectedFile.name}` : 'focus none';
    const blockedModelReason = 'Importa un STL/OBJ/DICOM o selecciona un modelo del caso antes de usar herramientas CAD.';

    return {
      hasSceneContent,
      hasModel,
      activeSelectionLabel,
      blockedModelReason,
      isActionBlocked(actionId: DentalCadShellActionId) {
        return MODEL_REQUIRED_ACTIONS.has(actionId) && !hasModel;
      },
      isProductModuleBlocked(moduleId: TlantiCadProductModuleId) {
        return MODEL_REQUIRED_MODULES.has(moduleId) && !hasModel;
      },
    };
  }, [files, selectedFile]);
}

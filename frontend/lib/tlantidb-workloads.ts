import type { TlantiCase, TlantiCaseAssetRole } from '@/stores/tlantidb-case-store';
import { isTlantiModuleId } from '@/core';
import type { TlantiModuleId } from '@/core';

export type TlantiWorkloadId =
  | 'crown-bridge'
  | 'implant-planning'
  | 'dicom-cbct'
  | 'smile-design'
  | 'splint-guide'
  | 'orthodontics';

export interface TlantiWorkloadPreset {
  id: TlantiWorkloadId;
  label: string;
  shortLabel: string;
  description: string;
  stage: 'scan' | 'design' | 'model' | 'manufacture' | 'export';
  moduleTarget: string;
  accent: string;
  requiredAssetRoles: TlantiCaseAssetRole[];
  optionalAssetRoles: TlantiCaseAssetRole[];
}

export type TlantiWorkloadStatus = 'missing-assets' | 'ready-to-design' | 'in-design' | 'ready-to-export' | 'complete';

export interface TlantiWorkloadAction {
  id: string;
  label: string;
  moduleTarget: TlantiModuleId;
}

export interface ApplyWorkloadToCaseInput {
  workloadId: TlantiWorkloadId;
  toothNumbers?: string[];
  activeJaw?: TlantiCase['activeJaw'];
  materialShade?: string;
  occlusionScanType?: string;
}

const WORKLOAD_MODULE_TARGETS: Record<string, TlantiModuleId> = {
  cad: 'cad',
  'crown-bridge': 'cad',
  'implant-planning': 'implant',
  implant: 'implant',
  'dicom-cbct': 'dicom',
  'dicom-viewer': 'dicom',
  dicom: 'dicom',
  'smile-design': 'orthocad',
  smile: 'orthocad',
  orthocad: 'orthocad',
  'splint-guide': 'splint',
  splint: 'splint',
  guide: 'guide',
  orthodontics: 'aligners',
  aligners: 'aligners',
  'model-creator': 'model-creator',
  fab: 'fab',
  partials: 'partials',
  ceph: 'ceph',
};

export function normalizeWorkloadModuleTarget(target?: string | null): TlantiModuleId {
  if (!target) return 'cad';
  const normalized = target.trim().toLowerCase();
  if (isTlantiModuleId(normalized)) return normalized;
  return WORKLOAD_MODULE_TARGETS[normalized] ?? 'cad';
}

export const TLANTIDB_WORKLOAD_PRESETS: TlantiWorkloadPreset[] = [
  {
    id: 'crown-bridge',
    label: 'Crown and bridge',
    shortLabel: 'Crown',
    description: 'Restorative CAD workflow with prep, antagonist, bite, margin and export readiness.',
    stage: 'design',
    moduleTarget: 'cad',
    accent: '#d71921',
    requiredAssetRoles: ['prep-scan'],
    optionalAssetRoles: ['antagonist-scan', 'bite-registration', 'shade-reference', 'lab-prescription'],
  },
  {
    id: 'implant-planning',
    label: 'Implant planning',
    shortLabel: 'Implant',
    description: 'CBCT, restoration and surgical planning assets for implant-driven design.',
    stage: 'design',
    moduleTarget: 'implant-planning',
    accent: '#f59e0b',
    requiredAssetRoles: ['dicom-study', 'restoration-model'],
    optionalAssetRoles: ['gingiva-scan', 'clinical-photo', 'lab-prescription'],
  },
  {
    id: 'dicom-cbct',
    label: 'DICOM / CBCT review',
    shortLabel: 'DICOM',
    description: 'Volumetric review, segmentation preparation and imaging handoff to CAD.',
    stage: 'scan',
    moduleTarget: 'dicom-viewer',
    accent: '#38bdf8',
    requiredAssetRoles: ['dicom-study'],
    optionalAssetRoles: ['clinical-note', 'restoration-model'],
  },
  {
    id: 'smile-design',
    label: 'Smile design',
    shortLabel: 'Smile',
    description: 'Photo-driven esthetic planning with shade, pre-op and mockup references.',
    stage: 'design',
    moduleTarget: 'smile-design',
    accent: '#a855f7',
    requiredAssetRoles: ['smile-photo'],
    optionalAssetRoles: ['pre-op-photo', 'shade-reference', 'clinical-photo'],
  },
  {
    id: 'splint-guide',
    label: 'Splint / surgical guide',
    shortLabel: 'Guide',
    description: 'Guide or splint workflow from scans, DICOM evidence and manufacturing output.',
    stage: 'model',
    moduleTarget: 'splint-guide',
    accent: '#22c55e',
    requiredAssetRoles: ['prep-scan', 'antagonist-scan'],
    optionalAssetRoles: ['dicom-study', 'bite-registration', 'manufacturing-report'],
  },
  {
    id: 'orthodontics',
    label: 'Orthodontics',
    shortLabel: 'Ortho',
    description: 'Upper/lower model workflow for aligners, treatment stages and case review.',
    stage: 'model',
    moduleTarget: 'orthodontics',
    accent: '#14b8a6',
    requiredAssetRoles: ['prep-scan', 'antagonist-scan'],
    optionalAssetRoles: ['bite-registration', 'clinical-photo', 'clinical-note'],
  },
];

export function getWorkloadPreset(workloadId: TlantiWorkloadId): TlantiWorkloadPreset {
  return TLANTIDB_WORKLOAD_PRESETS.find((preset) => preset.id === workloadId) ?? TLANTIDB_WORKLOAD_PRESETS[0];
}

export function inferWorkloadFromCase(caseItem: TlantiCase): TlantiWorkloadPreset {
  if (caseItem.workloadId) {
    return getWorkloadPreset(caseItem.workloadId as TlantiWorkloadId);
  }

  const moduleId = typeof caseItem.moduleTarget === 'string'
    ? caseItem.moduleTarget
    : typeof caseItem.moduleId === 'string'
      ? caseItem.moduleId
      : '';
  const assetRoles = new Set((caseItem.assets ?? []).map((asset) => asset.role));
  const name = `${caseItem.name} ${caseItem.notes} ${moduleId}`.toLowerCase();

  if (moduleId.includes('dicom') || assetRoles.has('dicom-study') || name.includes('cbct')) {
    return getWorkloadPreset(moduleId.includes('implant') ? 'implant-planning' : 'dicom-cbct');
  }
  if (moduleId.includes('smile') || assetRoles.has('smile-photo') || name.includes('smile')) {
    return getWorkloadPreset('smile-design');
  }
  if (moduleId.includes('orth') || name.includes('aligner') || name.includes('ortho')) {
    return getWorkloadPreset('orthodontics');
  }
  if (moduleId.includes('guide') || moduleId.includes('splint') || name.includes('guide') || name.includes('splint')) {
    return getWorkloadPreset('splint-guide');
  }

  return getWorkloadPreset('crown-bridge');
}

export function getMissingRequiredAssets(caseItem: TlantiCase): TlantiCaseAssetRole[] {
  const workload = inferWorkloadFromCase(caseItem);
  const requiredRoles = caseItem.requiredAssetRoles?.length ? caseItem.requiredAssetRoles : workload.requiredAssetRoles;
  const attachedRoles = new Set((caseItem.assets ?? []).map((asset) => asset.role));
  return requiredRoles.filter((role) => !attachedRoles.has(role));
}

export function getWorkloadStatus(caseItem: TlantiCase): TlantiWorkloadStatus {
  const missing = getMissingRequiredAssets(caseItem);
  if (missing.length) return 'missing-assets';
  if (caseItem.pipeline?.export) return 'complete';
  if (caseItem.pipeline?.manufacture) return 'ready-to-export';
  if (caseItem.pipeline?.design || caseItem.pipeline?.model) return 'in-design';
  return 'ready-to-design';
}

export function getNextWorkloadAction(caseItem: TlantiCase): TlantiWorkloadAction {
  const workload = inferWorkloadFromCase(caseItem);
  const missing = getMissingRequiredAssets(caseItem);
  const moduleTarget = normalizeWorkloadModuleTarget(caseItem.moduleTarget ?? workload.moduleTarget);

  if (missing.length) {
    return {
      id: 'import-required-assets',
      label: `Import ${missing.length} required asset${missing.length === 1 ? '' : 's'}`,
      moduleTarget,
    };
  }

  if (!caseItem.pipeline?.design) {
    return { id: 'open-design-module', label: `Open ${workload.shortLabel} module`, moduleTarget };
  }

  if (!caseItem.pipeline?.export) {
    return { id: 'validate-export', label: 'Validate export package', moduleTarget };
  }

  return { id: 'review-complete', label: 'Review completed case', moduleTarget };
}

export function applyWorkloadToCase(caseItem: TlantiCase, input: ApplyWorkloadToCaseInput): TlantiCase {
  const workload = getWorkloadPreset(input.workloadId);
  const moduleTarget = normalizeWorkloadModuleTarget(workload.moduleTarget);
  const selectedTeeth = new Set(input.toothNumbers ?? []);
  const nextToothMap = Object.fromEntries(
    Object.entries(caseItem.toothMap).map(([key, state]) => {
      const toothNumber = key.replace('tooth-', '');
      if (!selectedTeeth.has(toothNumber)) return [key, state];
      return [
        key,
        {
          ...state,
          selected: true,
          shade: input.materialShade ?? state.shade ?? 'A1',
          workTypeId: workload.id,
        },
      ];
    }),
  ) as TlantiCase['toothMap'];

  return {
    ...caseItem,
    activeJaw: input.activeJaw ?? caseItem.activeJaw,
    toothMap: nextToothMap,
    workloadId: workload.id,
    workloadLabel: workload.label,
    moduleTarget,
    moduleId: moduleTarget,
    requiredAssetRoles: workload.requiredAssetRoles,
    optionalAssetRoles: workload.optionalAssetRoles,
    workloadStatus: getWorkloadStatus({ ...caseItem, requiredAssetRoles: workload.requiredAssetRoles }),
    occlusionScanType: input.occlusionScanType ?? caseItem.occlusionScanType,
  };
}

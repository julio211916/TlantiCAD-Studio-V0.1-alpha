import { DENTAL_WORK_TYPES, resolveWorkType } from '@/lib/dental-work-catalog';
import { getMissingRequiredAssets, inferWorkloadFromCase, normalizeWorkloadModuleTarget } from '@/lib/tlantidb-workloads';
import type { TlantiCase, TlantiToothState } from '@/stores/tlantidb-case-store';

export interface TlantiDbClinicalOrderNode {
  id: string;
  label: string;
  detail: string;
  status: 'ready' | 'partial' | 'blocked';
}

export interface TlantiDbToothWorkOrder {
  id: string;
  toothCode: string;
  workTypeId: string;
  workTypeLabel: string;
  materialType: string;
  shade: string;
  productionMethod: string;
  implantMode: string;
  isAntagonist: boolean;
  parameters: Array<{
    key: string;
    kind: 'text' | 'number' | 'boolean' | 'json';
    value: string | number | boolean | Record<string, unknown>;
  }>;
}

export interface TlantiDbClinicalOrderGraph {
  practice: TlantiDbClinicalOrderNode;
  patient: TlantiDbClinicalOrderNode;
  technician: TlantiDbClinicalOrderNode;
  caseOrder: TlantiDbClinicalOrderNode;
  toothWorks: TlantiDbToothWorkOrder[];
  missingAssetRoles: string[];
  moduleTarget: string;
  workloadLabel: string;
  sqlTables: string[];
}

function nodeStatus(hasValue: boolean, partial = false): TlantiDbClinicalOrderNode['status'] {
  if (hasValue) return partial ? 'partial' : 'ready';
  return 'blocked';
}

function parameterList(tooth: TlantiToothState): TlantiDbToothWorkOrder['parameters'] {
  const parameters: TlantiDbToothWorkOrder['parameters'] = [];

  if (typeof tooth.minimalThicknessMm === 'number') {
    parameters.push({ key: 'minimalThicknessMm', kind: 'number', value: tooth.minimalThicknessMm });
  }
  if (typeof tooth.cementGapMm === 'number') {
    parameters.push({ key: 'cementGapMm', kind: 'number', value: tooth.cementGapMm });
  }
  if (typeof tooth.workTimeMinutes === 'number') {
    parameters.push({ key: 'workTimeMinutes', kind: 'number', value: tooth.workTimeMinutes });
  }
  if (tooth.biteSplintMode) {
    parameters.push({ key: 'biteSplintMode', kind: 'text', value: tooth.biteSplintMode });
  }
  if (tooth.biteSplintAntagonistScan) {
    parameters.push({ key: 'biteSplintAntagonistScan', kind: 'text', value: tooth.biteSplintAntagonistScan });
  }
  if (tooth.additionalScans && Object.keys(tooth.additionalScans).length) {
    parameters.push({ key: 'additionalScans', kind: 'json', value: tooth.additionalScans });
  }
  if (typeof tooth.screwHoleCut === 'boolean') {
    parameters.push({ key: 'screwHoleCut', kind: 'boolean', value: tooth.screwHoleCut });
  }

  return parameters;
}

export function buildTlantiDbClinicalOrderGraph(activeCase: TlantiCase): TlantiDbClinicalOrderGraph {
  const workload = inferWorkloadFromCase(activeCase);
  const missingAssetRoles = getMissingRequiredAssets(activeCase);
  const selectedToothEntries = Object.entries(activeCase.toothMap)
    .filter(([, tooth]) => tooth.selected)
    .sort(([a], [b]) => Number(a.replace('tooth-', '')) - Number(b.replace('tooth-', '')));

  const toothWorks = selectedToothEntries.map(([key, tooth]) => {
    const toothCode = key.replace('tooth-', '');
    const workTypeId = tooth.workTypeId ?? tooth.restorationType ?? 'anatomic-crown';
    const workType = resolveWorkType(workTypeId) ?? DENTAL_WORK_TYPES[0];
    return {
      id: `${activeCase.id}:${toothCode}`,
      toothCode,
      workTypeId,
      workTypeLabel: workType?.label ?? workTypeId,
      materialType: tooth.material ?? activeCase.materialShade ?? 'zirconia',
      shade: tooth.shade ?? activeCase.materialShade ?? 'A2',
      productionMethod: tooth.productionMethod ?? 'inhouse-milling',
      implantMode: tooth.implantMode ?? 'none',
      isAntagonist: Boolean(tooth.antagonist),
      parameters: parameterList(tooth),
    };
  });

  const patientLabel = activeCase.patientName || activeCase.clientName || 'No patient assigned';
  const practiceLabel = activeCase.clientName || activeCase.laboratoryName || 'No practice assigned';
  const technicianLabel = activeCase.technicianName || 'No technician assigned';

  return {
    practice: {
      id: activeCase.clientId || 'practice-default',
      label: practiceLabel,
      detail: activeCase.laboratoryName ? `Lab: ${activeCase.laboratoryName}` : 'Maps to practices',
      status: nodeStatus(Boolean(activeCase.clientName || activeCase.laboratoryName), !activeCase.clientId),
    },
    patient: {
      id: activeCase.id,
      label: patientLabel,
      detail: activeCase.patientDateOfBirth ? `DOB ${activeCase.patientDateOfBirth}` : 'Maps to patients',
      status: nodeStatus(Boolean(activeCase.patientName || activeCase.clientName), !activeCase.patientDateOfBirth),
    },
    technician: {
      id: activeCase.technicianId || 'technician-local',
      label: technicianLabel,
      detail: activeCase.technicianId ? `ID ${activeCase.technicianId}` : 'Maps to technicians',
      status: nodeStatus(Boolean(activeCase.technicianName), !activeCase.technicianId),
    },
    caseOrder: {
      id: activeCase.id,
      label: activeCase.caseNumber,
      detail: `${activeCase.name} · ${workload.label}`,
      status: missingAssetRoles.length ? 'partial' : 'ready',
    },
    toothWorks,
    missingAssetRoles,
    moduleTarget: normalizeWorkloadModuleTarget(activeCase.moduleTarget ?? workload.moduleTarget),
    workloadLabel: activeCase.workloadLabel ?? workload.label,
    sqlTables: [
      'practices',
      'patients',
      'technicians',
      'case_order_context',
      'case_tooth_work',
      'tooth_work_parameters',
      'work_parameter_versions',
    ],
  };
}

import type { ClinicalChecklistItem } from '@/lib/tlantidb-clinical-workflow';
import type { TlantiCase, TlantiCaseAsset, TlantiCasePipeline } from '@/stores/tlantidb-case-store';

export type TlantiCaseStatus =
  | 'new'
  | 'case-data-complete'
  | 'scan-required'
  | 'scan-ready'
  | 'design-active'
  | 'design-ready'
  | 'manufacturing-ready'
  | 'exported'
  | 'archived';

export type TlantiCaseStatusTransition =
  | 'case-data-saved'
  | 'assets-imported'
  | 'design-started'
  | 'design-completed'
  | 'manufacturing-ready'
  | 'exported'
  | 'archived'
  | 'reopen';

export const TLANTI_CASE_STATUS_LABELS: Record<TlantiCaseStatus, string> = {
  new: 'Nuevo',
  'case-data-complete': 'Datos completos',
  'scan-required': 'En preparacion',
  'scan-ready': 'Listo para CAD',
  'design-active': 'En diseno',
  'design-ready': 'Diseno completo',
  'manufacturing-ready': 'En fabricacion',
  exported: 'Exportado',
  archived: 'Archivado',
};

export function normalizeTlantiCaseStatus(status?: string | null): TlantiCaseStatus {
  switch (status) {
    case 'case-data-complete':
    case 'scan-required':
    case 'scan-ready':
    case 'design-active':
    case 'design-ready':
    case 'manufacturing-ready':
    case 'exported':
    case 'archived':
    case 'new':
      return status;
    case 'in-progress':
      return 'design-active';
    case 'ready':
      return 'design-ready';
    case 'planned':
    case 'draft':
    default:
      return 'new';
  }
}

function hasCaseData(activeCase: TlantiCase) {
  return Boolean(
    activeCase.name.trim()
      || activeCase.caseNumber.trim()
      || activeCase.clientName.trim()
      || activeCase.patientName?.trim()
      || activeCase.technicianName.trim()
      || activeCase.laboratoryName?.trim()
      || Object.values(activeCase.toothMap).some((tooth) => tooth.selected),
  );
}

function hasClinicalScan(assets: readonly TlantiCaseAsset[]) {
  return assets.some((asset) => (
    asset.role === 'prep-scan'
    || asset.role === 'antagonist-scan'
    || asset.role === 'bite-registration'
    || asset.role === 'dicom-study'
    || asset.role === 'restoration-model'
  ));
}

function areRequiredChecklistItemsComplete(checklist: readonly ClinicalChecklistItem[]) {
  const required = checklist.filter((item) => item.required);
  return required.length > 0 && required.every((item) => item.completed);
}

export function resolveCaseStatusFromClinicalProgress(
  activeCase: TlantiCase,
  assets: readonly TlantiCaseAsset[] = activeCase.assets ?? [],
  pipeline: TlantiCasePipeline = activeCase.pipeline ?? {
    scan: false,
    design: false,
    model: false,
    manufacture: false,
    export: false,
  },
  checklist: readonly ClinicalChecklistItem[] = [],
): TlantiCaseStatus {
  const current = normalizeTlantiCaseStatus(activeCase.status);

  if (current === 'archived') {
    return 'archived';
  }

  if (pipeline.export || activeCase.lastInteropXmlPath || activeCase.lastExportedAt) {
    return 'exported';
  }

  if (pipeline.manufacture) {
    return 'manufacturing-ready';
  }

  if (pipeline.design && areRequiredChecklistItemsComplete(checklist)) {
    return 'design-ready';
  }

  if (pipeline.design) {
    return 'design-active';
  }

  if ((pipeline.scan || hasClinicalScan(assets)) && (checklist.length === 0 || areRequiredChecklistItemsComplete(checklist))) {
    return 'scan-ready';
  }

  if (hasCaseData(activeCase)) {
    return assets.length > 0 || checklist.some((item) => item.required && !item.completed)
      ? 'scan-required'
      : 'case-data-complete';
  }

  return 'new';
}

export function transitionTlantiCaseStatus(
  currentStatus: string | null | undefined,
  transition: TlantiCaseStatusTransition,
): TlantiCaseStatus {
  const current = normalizeTlantiCaseStatus(currentStatus);

  if (current === 'archived' && transition !== 'reopen') {
    return 'archived';
  }

  if (current === 'exported' && !['archived', 'reopen', 'exported'].includes(transition)) {
    return 'exported';
  }

  switch (transition) {
    case 'case-data-saved':
      return current === 'new' ? 'case-data-complete' : current;
    case 'assets-imported':
      return current === 'new' || current === 'case-data-complete' ? 'scan-required' : current;
    case 'design-started':
      return 'design-active';
    case 'design-completed':
      return 'design-ready';
    case 'manufacturing-ready':
      return 'manufacturing-ready';
    case 'exported':
      return 'exported';
    case 'archived':
      return 'archived';
    case 'reopen':
      return 'design-active';
    default:
      return current;
  }
}

export function formatTlantiCaseStatus(status?: string | null) {
  return TLANTI_CASE_STATUS_LABELS[normalizeTlantiCaseStatus(status)];
}

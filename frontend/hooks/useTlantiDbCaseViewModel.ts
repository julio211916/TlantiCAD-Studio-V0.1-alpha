import { useMemo } from 'react';

import {
  deriveChecklistFromCase,
  deriveNextClinicalAction,
  derivePipelineFromClinicalAssets,
} from '@/lib/tlantidb-clinical-workflow';
import {
  formatTlantiCaseStatus,
  resolveCaseStatusFromClinicalProgress,
  type TlantiCaseStatus,
} from '@/lib/tlantidb-case-state-machine';
import type { DentalDbLabQueueStats } from '@/components/tlantidb/DentalDbWelcomeLauncher';
import type { TlantiCase, TlantiDbState } from '@/stores/tlantidb-case-store';

export type TlantiCaseStatusTone = 'neutral' | 'warning' | 'success' | 'info' | 'accent';

export interface TlantiPhaseItem {
  id: 'case' | 'scan' | 'cad' | 'manufacturing';
  label: string;
  complete: boolean;
  active: boolean;
  blocked: boolean;
}

export interface TlantiChecklistSummary {
  completed: number;
  total: number;
  requiredIncomplete: number;
  percent: number;
}

export interface TlantiActionAvailability {
  canLaunchCad: boolean;
  cadBlockedReason: string | null;
}

export interface UseTlantiDbCaseViewModelInput {
  activeCase: TlantiCase;
  databaseState: TlantiDbState;
  isPristineCase: boolean;
  selectedToothCount: number;
}

const STATUS_TONES: Record<TlantiCaseStatus, TlantiCaseStatusTone> = {
  new: 'neutral',
  'case-data-complete': 'info',
  'scan-required': 'warning',
  'scan-ready': 'success',
  'design-active': 'info',
  'design-ready': 'accent',
  'manufacturing-ready': 'accent',
  exported: 'success',
  archived: 'neutral',
};

function buildPhaseItems(status: TlantiCaseStatus, selectedToothCount: number, primaryScanReady: boolean): TlantiPhaseItem[] {
  const hasCaseData = status !== 'new' || selectedToothCount > 0;
  const cadStarted = status === 'design-active' || status === 'design-ready' || status === 'manufacturing-ready' || status === 'exported' || status === 'archived';
  const designComplete = status === 'design-ready' || status === 'manufacturing-ready' || status === 'exported' || status === 'archived';
  const manufacturingReady = status === 'manufacturing-ready' || status === 'exported' || status === 'archived';

  return [
    { id: 'case', label: 'Datos', complete: hasCaseData, active: !hasCaseData, blocked: false },
    { id: 'scan', label: 'Escaneos', complete: primaryScanReady, active: hasCaseData && !primaryScanReady, blocked: !hasCaseData },
    { id: 'cad', label: 'CAD', complete: designComplete, active: primaryScanReady && cadStarted && !designComplete, blocked: !primaryScanReady },
    { id: 'manufacturing', label: 'Fabricacion', complete: manufacturingReady, active: designComplete && !manufacturingReady, blocked: !designComplete },
  ];
}

export function useTlantiDbCaseViewModel({
  activeCase,
  databaseState,
  isPristineCase,
  selectedToothCount,
}: UseTlantiDbCaseViewModelInput) {
  return useMemo(() => {
    const assets = activeCase.assets ?? [];
    const clinicalChecklist = deriveChecklistFromCase(activeCase, assets);
    const pipeline = derivePipelineFromClinicalAssets(activeCase, assets);
    const caseStatus = resolveCaseStatusFromClinicalProgress(activeCase, assets, pipeline, clinicalChecklist);
    const completedChecklistCount = clinicalChecklist.filter((item) => item.completed).length;
    const requiredIncomplete = clinicalChecklist.filter((item) => item.required && !item.completed).length;
    const nextChecklistItem = clinicalChecklist.find((item) => item.required && !item.completed)
      ?? clinicalChecklist.find((item) => !item.completed)
      ?? null;
    const primaryScanChecklistItem = clinicalChecklist.find((item) => item.id === 'primary-scan') ?? null;
    const primaryScanReady = primaryScanChecklistItem?.completed ?? false;
    const canLaunchCad = selectedToothCount > 0 && primaryScanReady;
    const visibleCases = databaseState.cases.filter((item) => !(isPristineCase && item.id === activeCase.id));

    const labQueueStats: DentalDbLabQueueStats = {
      queued: visibleCases.length,
      scanBlocked: visibleCases.filter((item) => {
        const itemChecklist = deriveChecklistFromCase(item, item.assets ?? []);
        return !itemChecklist.find((check) => check.id === 'primary-scan')?.completed;
      }).length,
      fabReady: visibleCases.filter((item) => item.status === 'manufacturing-ready' || item.status === 'exported').length,
    };

    const checklistSummary: TlantiChecklistSummary = {
      completed: completedChecklistCount,
      total: clinicalChecklist.length,
      requiredIncomplete,
      percent: clinicalChecklist.length > 0 ? Math.round((completedChecklistCount / clinicalChecklist.length) * 100) : 0,
    };

    const actionAvailability: TlantiActionAvailability = {
      canLaunchCad,
      cadBlockedReason: canLaunchCad
        ? null
        : primaryScanReady
          ? 'Selecciona al menos un diente antes de abrir CAD.'
          : 'Importa el escaneo primario antes de abrir CAD Module.',
    };

    return {
      clinicalChecklist,
      completedChecklistCount,
      nextChecklistItem,
      primaryScanChecklistItem,
      primaryScanReady,
      canLaunchCadModule: canLaunchCad,
      caseStatus,
      caseStatusLabel: formatTlantiCaseStatus(caseStatus),
      caseStatusTone: STATUS_TONES[caseStatus],
      phaseItems: buildPhaseItems(caseStatus, selectedToothCount, primaryScanReady),
      nextClinicalAction: deriveNextClinicalAction(activeCase, assets),
      checklistSummary,
      actionAvailability,
      recentCases: databaseState.cases
        .filter((item) => item.id !== activeCase.id && (!isPristineCase || item.status !== 'new'))
        .slice(0, 4),
      labQueueStats,
    };
  }, [activeCase, databaseState.cases, isPristineCase, selectedToothCount]);
}

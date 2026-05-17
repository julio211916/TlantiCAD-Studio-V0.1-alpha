import { getRestorationLabel, getToothOrderIndex, type DentalImplantMode } from '@/lib/dental-workflow'
import type { TlantiCase, TlantiCaseAsset, TlantiCaseAssetRole } from '@/stores/tlantidb-case-store'

export interface ImplantTargetSummary {
  toothNumber: string
  implantMode: DentalImplantMode
  restorationLabel: string
  usesPreOpModel: boolean
  needsExtraGingivaScan: boolean
}

export interface ClinicalAssetReadiness {
  counts: Partial<Record<TlantiCaseAssetRole, number>>
  totalAssets: number
  hasDicomStudy: boolean
  hasPrepScan: boolean
  hasGingivaScan: boolean
  hasOpposingRecord: boolean
  hasLabPrescription: boolean
  hasManufacturingReport: boolean
}

export type SplintIndication = 'stabilization' | 'deprogrammer' | 'night-guard' | 'surgical-provisional'

export interface SplintTargetSummary {
  toothNumber: string
  restorationLabel: string
  activeJaw: TlantiCase['activeJaw']
  usesPreOpModel: boolean
  implantRelated: boolean
}

function incrementRoleCount(counts: Partial<Record<TlantiCaseAssetRole, number>>, role: TlantiCaseAssetRole) {
  counts[role] = (counts[role] ?? 0) + 1
}

export function getClinicalAssetReadiness(assets: TlantiCaseAsset[] = []): ClinicalAssetReadiness {
  const counts: Partial<Record<TlantiCaseAssetRole, number>> = {}

  assets.forEach((asset) => {
    incrementRoleCount(counts, asset.role)
  })

  return {
    counts,
    totalAssets: assets.length,
    hasDicomStudy: Boolean(counts['dicom-study']),
    hasPrepScan: Boolean(counts['prep-scan']),
    hasGingivaScan: Boolean(counts['gingiva-scan']),
    hasOpposingRecord: Boolean(counts['antagonist-scan'] || counts['bite-registration']),
    hasLabPrescription: Boolean(counts['lab-prescription']),
    hasManufacturingReport: Boolean(counts['manufacturing-report']),
  }
}

export function getImplantTargets(activeCase?: TlantiCase | null): ImplantTargetSummary[] {
  if (!activeCase) {
    return []
  }

  return Object.entries(activeCase.toothMap)
    .filter(([, toothState]) => {
      const hasImplantMode = toothState.implantMode && toothState.implantMode !== 'none'
      const isImplantRestoration = toothState.restorationType === 'implant-restoration'
      return Boolean(hasImplantMode || isImplantRestoration)
    })
    .map(([toothKey, toothState]) => ({
      toothNumber: toothKey.replace('tooth-', ''),
      implantMode: toothState.implantMode && toothState.implantMode !== 'none' ? toothState.implantMode : 'custom-abutment',
      restorationLabel: getRestorationLabel(toothState.restorationType),
      usesPreOpModel: Boolean(toothState.usePreOpModel),
      needsExtraGingivaScan: Boolean(toothState.useExtraGingivaScan),
    }))
    .sort((left, right) => getToothOrderIndex(left.toothNumber) - getToothOrderIndex(right.toothNumber))
}

export function getSplintTargets(activeCase?: TlantiCase | null): SplintTargetSummary[] {
  if (!activeCase) {
    return []
  }

  return Object.entries(activeCase.toothMap)
    .filter(([, toothState]) => Boolean(toothState.selected && !toothState.antagonist))
    .map(([toothKey, toothState]) => ({
      toothNumber: toothKey.replace('tooth-', ''),
      restorationLabel: getRestorationLabel(toothState.restorationType),
      activeJaw: activeCase.activeJaw,
      usesPreOpModel: Boolean(toothState.usePreOpModel),
      implantRelated: toothState.restorationType === 'implant-restoration' || Boolean(toothState.implantMode && toothState.implantMode !== 'none'),
    }))
    .sort((left, right) => getToothOrderIndex(left.toothNumber) - getToothOrderIndex(right.toothNumber))
}

export function getRecommendedSplintIndication(activeCase?: TlantiCase | null, readiness?: ClinicalAssetReadiness): SplintIndication {
  if (!activeCase) {
    return 'stabilization'
  }

  const toothStates = Object.values(activeCase.toothMap)

  if (toothStates.some((item) => item.implantMode && item.implantMode !== 'none')) {
    return 'surgical-provisional'
  }

  if (readiness?.hasOpposingRecord && activeCase.occlusionScanType === 'bite_registration') {
    return 'deprogrammer'
  }

  if (readiness?.hasOpposingRecord) {
    return 'night-guard'
  }

  return 'stabilization'
}

export function downloadJsonBrief(fileName: string, payload: unknown) {
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = fileName
  anchor.click()
  URL.revokeObjectURL(url)
}
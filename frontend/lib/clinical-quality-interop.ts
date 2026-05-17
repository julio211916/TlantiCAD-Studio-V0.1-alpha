import type { BackendCatalog, PythonBridgeStatus, SystemRuntimeReport } from '@/lib/backend-integrations'
import { downloadJsonBrief } from '@/lib/clinical-module-briefs'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import { deriveChecklistFromCase, derivePipelineFromClinicalAssets, type ClinicalChecklistItem } from '@/lib/tlantidb-clinical-workflow'
import type { TlantiCase, TlantiDbState } from '@/stores/tlantidb-case-store'

export type QualityGateSeverity = 'low' | 'medium' | 'high'
export type QualityGateCategory = 'clinical' | 'interop' | 'security' | 'runtime' | 'reproducibility'
export type InteropCapabilityStatus = 'ready' | 'partial' | 'planned' | 'blocked'

export interface QualityGateRisk {
  id: string
  severity: QualityGateSeverity
  category: QualityGateCategory
  title: string
  detail: string
  blocked: boolean
}

export interface InteropCapability {
  id: string
  label: string
  status: InteropCapabilityStatus
  detail: string
}

export interface QualityGateScorecard {
  clinical: number
  reproducibility: number
  interop: number
  security: number
  overall: number
}

export interface ClinicalQualityGateReport {
  kind: 'clinical-quality-interop-gate'
  sprintRange: '101-120'
  generatedAt: string
  releaseRecommendation: 'ready' | 'hold'
  caseId: string
  caseNumber: string
  caseName: string
  storagePath?: string | null
  interopXmlPath?: string | null
  checklist: ClinicalChecklistItem[]
  checklistSummary: {
    requiredComplete: number
    requiredTotal: number
    optionalComplete: number
    optionalTotal: number
  }
  pipeline: ReturnType<typeof derivePipelineFromClinicalAssets>
  scorecard: QualityGateScorecard
  risks: QualityGateRisk[]
  interopCapabilities: InteropCapability[]
  runtimeSummary: {
    runtimeLoaded: boolean
    pythonLoaded: boolean
    backendCatalogLoaded: boolean
    diagnosticsLoaded: boolean
    platform?: string
    recommendedRenderQuality?: string
    readyWorkflows: number
    totalWorkflows: number
  }
}

export interface InteropReadinessManifest {
  kind: 'interop-readiness-manifest'
  sprintRange: '111-120'
  generatedAt: string
  caseId: string
  caseNumber: string
  numberingSystem: TlantiDbState['preferences']['numberingSystem']
  operatorAlias: string
  clientAlias: string
  directIdentifiersPresent: boolean
  exports: {
    caseFolder?: string | null
    millboxXml?: string | null
  }
  capabilities: InteropCapability[]
  blockers: string[]
}

function roundScore(value: number) {
  return Math.max(0, Math.min(100, Math.round(value)))
}

function scoreFromChecks(checks: boolean[]) {
  if (!checks.length) {
    return 0
  }

  const passed = checks.filter(Boolean).length
  return roundScore((passed / checks.length) * 100)
}

function buildInteropCapabilities(activeCase: TlantiCase, state: TlantiDbState, backendCatalog: BackendCatalog | null): InteropCapability[] {
  const assets = activeCase.assets ?? []
  const hasDicomStudy = assets.some((asset) => asset.role === 'dicom-study')
  const hasConsent = assets.some((asset) => asset.role === 'consent-document')
  const dicomIntegration = backendCatalog?.integrations.find((integration) => integration.id === 'dicom-core')

  return [
    {
      id: 'case-folder',
      label: 'Case folder package',
      status: activeCase.storagePath ? 'ready' : 'partial',
      detail: activeCase.storagePath ? 'Case is persisted locally and can be replayed.' : 'Case can export, but has not been persisted to a local `.tlanticad` folder yet.',
    },
    {
      id: 'millbox-xml',
      label: 'MillBox XML interop',
      status: activeCase.lastInteropXmlPath ? 'ready' : 'partial',
      detail: activeCase.lastInteropXmlPath ? `Last XML written to ${activeCase.lastInteropXmlPath}.` : 'XML generation exists, but there is no saved interop file for the active case yet.',
    },
    {
      id: 'dicom-intake',
      label: 'DICOM intake baseline',
      status: hasDicomStudy && dicomIntegration?.enabled ? 'ready' : hasDicomStudy || dicomIntegration?.enabled ? 'partial' : 'blocked',
      detail: hasDicomStudy ? 'The case already contains DICOM study assets for downstream interoperability.' : 'Attach at least one DICOM study to make PACS-facing work meaningful for this case.',
    },
    {
      id: 'de-identification',
      label: 'De-identification readiness',
      status: !activeCase.clientId && hasConsent ? 'partial' : activeCase.clientId ? 'blocked' : 'planned',
      detail: !activeCase.clientId && hasConsent
        ? 'Consent is attached and there is no explicit client identifier in case metadata, but no automated de-id workflow is wired yet.'
        : activeCase.clientId
          ? 'Client identifier is populated in case metadata. Add a pseudonymization step before external interoperability.'
          : 'Consent/de-identification workflow is still planned for this case.',
    },
    {
      id: 'orthanc-bridge',
      label: 'Orthanc / PACS bridge',
      status: 'planned',
      detail: 'Roadmap target for C-STORE, C-FIND and C-MOVE exists, but the bridge is not productized in this slice yet.',
    },
    {
      id: 'structured-report',
      label: 'Structured report export',
      status: 'planned',
      detail: 'RTSTRUCT / SR reporting is documented in the DICOM roadmap but not shipped in this slice yet.',
    },
  ]
}

function buildQualityGateRisks(
  activeCase: TlantiCase,
  checklist: ClinicalChecklistItem[],
  capabilities: InteropCapability[],
  pythonBridge: PythonBridgeStatus | null,
  diagnostics: RuntimeDiagnostics | null,
): QualityGateRisk[] {
  const requiredIncomplete = checklist.filter((item) => item.required && !item.completed)
  const risks: QualityGateRisk[] = []

  if (requiredIncomplete.length) {
    risks.push({
      id: 'required-checklist',
      severity: 'high',
      category: 'clinical',
      title: 'Clinical checklist still has blockers',
      detail: `${requiredIncomplete.length} required checklist item(s) remain incomplete before the case should move forward.`,
      blocked: true,
    })
  }

  if (!activeCase.storagePath) {
    risks.push({
      id: 'case-folder-missing',
      severity: 'high',
      category: 'reproducibility',
      title: 'Case is not persisted to disk',
      detail: 'Without a local `.tlanticad` folder, reproducibility and audit replay remain fragile.',
      blocked: true,
    })
  }

  if (activeCase.clientId) {
    risks.push({
      id: 'direct-identifier-present',
      severity: 'high',
      category: 'security',
      title: 'Direct patient identifier present',
      detail: 'The active case still carries `clientId`; a de-identification/pseudonymization step is needed before external interoperability.',
      blocked: true,
    })
  }

  if (!activeCase.assets?.some((asset) => asset.role === 'consent-document')) {
    risks.push({
      id: 'consent-missing',
      severity: 'medium',
      category: 'security',
      title: 'Consent document not attached',
      detail: 'Consent/de-identification evidence is still missing from the active case assets.',
      blocked: false,
    })
  }

  if (!activeCase.lastInteropXmlPath) {
    risks.push({
      id: 'interop-xml-missing',
      severity: 'medium',
      category: 'interop',
      title: 'Interop XML not generated',
      detail: 'The case does not yet have a persisted XML export for MillBox or downstream manufacturing systems.',
      blocked: false,
    })
  }

  if (!capabilities.some((capability) => capability.id === 'dicom-intake' && capability.status === 'ready')) {
    risks.push({
      id: 'dicom-interoperability-gap',
      severity: 'medium',
      category: 'interop',
      title: 'DICOM interoperability context is incomplete',
      detail: 'Attach a DICOM study and verify the DICOM core integration before treating PACS readiness as complete.',
      blocked: false,
    })
  }

  if (!pythonBridge?.workflows.some((workflow) => workflow.ready)) {
    risks.push({
      id: 'python-runtime-gap',
      severity: 'low',
      category: 'runtime',
      title: 'Python clinical workflows not ready',
      detail: 'The desktop runtime does not currently report any AI workflows as ready.',
      blocked: false,
    })
  }

  if (!diagnostics) {
    risks.push({
      id: 'runtime-diagnostics-missing',
      severity: 'low',
      category: 'runtime',
      title: 'Runtime diagnostics not captured',
      detail: 'Capture diagnostics before escalating quality or interoperability incidents.',
      blocked: false,
    })
  }

  return risks
}

export function buildClinicalQualityGateReport(input: {
  activeCase: TlantiCase
  state: TlantiDbState
  systemRuntimeReport?: SystemRuntimeReport | null
  pythonBridge?: PythonBridgeStatus | null
  backendCatalog?: BackendCatalog | null
  runtimeDiagnostics?: RuntimeDiagnostics | null
}): ClinicalQualityGateReport {
  const { activeCase, state, systemRuntimeReport, pythonBridge, backendCatalog, runtimeDiagnostics } = input
  const assets = activeCase.assets ?? []
  const checklist = deriveChecklistFromCase(activeCase, assets)
  const pipeline = derivePipelineFromClinicalAssets(activeCase, assets)
  const requiredItems = checklist.filter((item) => item.required)
  const optionalItems = checklist.filter((item) => !item.required)
  const interopCapabilities = buildInteropCapabilities(activeCase, state, backendCatalog ?? null)
  const risks = buildQualityGateRisks(activeCase, checklist, interopCapabilities, pythonBridge ?? null, runtimeDiagnostics ?? null)

  const scorecard: QualityGateScorecard = {
    clinical: scoreFromChecks(requiredItems.map((item) => item.completed)),
    reproducibility: scoreFromChecks([
      Boolean(activeCase.storagePath),
      Boolean(state.preferences.operatorAlias),
      Boolean(state.preferences.numberingSystem),
      assets.length > 0,
      Boolean(activeCase.lastInteropXmlPath),
    ]),
    interop: scoreFromChecks(interopCapabilities.map((capability) => capability.status === 'ready' || capability.status === 'partial')),
    security: scoreFromChecks([
      !activeCase.clientId,
      assets.some((asset) => asset.role === 'consent-document'),
      Boolean(runtimeDiagnostics),
    ]),
    overall: 0,
  }

  scorecard.overall = roundScore((scorecard.clinical + scorecard.reproducibility + scorecard.interop + scorecard.security) / 4)

  return {
    kind: 'clinical-quality-interop-gate',
    sprintRange: '101-120',
    generatedAt: new Date().toISOString(),
    releaseRecommendation: risks.some((risk) => risk.blocked && risk.severity === 'high') ? 'hold' : 'ready',
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    caseName: activeCase.name,
    storagePath: activeCase.storagePath,
    interopXmlPath: activeCase.lastInteropXmlPath,
    checklist,
    checklistSummary: {
      requiredComplete: requiredItems.filter((item) => item.completed).length,
      requiredTotal: requiredItems.length,
      optionalComplete: optionalItems.filter((item) => item.completed).length,
      optionalTotal: optionalItems.length,
    },
    pipeline,
    scorecard,
    risks,
    interopCapabilities,
    runtimeSummary: {
      runtimeLoaded: Boolean(systemRuntimeReport),
      pythonLoaded: Boolean(pythonBridge),
      backendCatalogLoaded: Boolean(backendCatalog),
      diagnosticsLoaded: Boolean(runtimeDiagnostics),
      platform: runtimeDiagnostics?.platform,
      recommendedRenderQuality: systemRuntimeReport?.system.capabilities.recommendedRenderQuality,
      readyWorkflows: pythonBridge?.workflows.filter((workflow) => workflow.ready).length ?? 0,
      totalWorkflows: pythonBridge?.workflows.length ?? 0,
    },
  }
}

export function buildInteropReadinessManifest(report: ClinicalQualityGateReport, state: TlantiDbState, activeCase: TlantiCase): InteropReadinessManifest {
  return {
    kind: 'interop-readiness-manifest',
    sprintRange: '111-120',
    generatedAt: new Date().toISOString(),
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    numberingSystem: state.preferences.numberingSystem,
    operatorAlias: state.preferences.operatorAlias,
    clientAlias: activeCase.clientName,
    directIdentifiersPresent: Boolean(activeCase.clientId),
    exports: {
      caseFolder: activeCase.storagePath,
      millboxXml: activeCase.lastInteropXmlPath,
    },
    capabilities: report.interopCapabilities,
    blockers: report.risks.filter((risk) => risk.blocked).map((risk) => `${risk.category}: ${risk.title}`),
  }
}

export function exportClinicalQualityGateReport(report: ClinicalQualityGateReport) {
  downloadJsonBrief(`${report.caseNumber}-quality-gate-101-120.json`, report)
}

export function exportInteropReadinessManifest(manifest: InteropReadinessManifest) {
  downloadJsonBrief(`${manifest.caseNumber}-interop-manifest-111-120.json`, manifest)
}
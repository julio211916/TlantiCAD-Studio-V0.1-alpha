import { getClinicalAssetReadiness, getImplantTargets } from '@/lib/clinical-module-briefs'
import { downloadJsonBrief } from '@/lib/clinical-module-briefs'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import type {
  TlantiCase,
  TlantiCaseApproval,
  TlantiCaseCollaboratorRole,
  TlantiCaseComment,
  TlantiCaseDecision,
  TlantiCaseNotification,
  TlantiCaseCollaborationState,
  TlantiDbState,
} from '@/stores/tlantidb-case-store'

export interface CollaborationRolePermission {
  role: TlantiCaseCollaboratorRole
  label: string
  canComment: boolean
  canApprove: boolean
  canExportAudit: boolean
  canAcceptAutoPlan: boolean
}

export interface CaseTimelineEvent {
  id: string
  kind: 'case' | 'comment' | 'decision' | 'approval' | 'interop' | 'export' | 'notification'
  title: string
  detail: string
  actor: string
  createdAt: string
}

export interface ImplantAutoPlanningSuggestion {
  toothNumber: string
  implantMode: string
  restorationLabel: string
  suggestedAxis: string
  suggestedPosition: string
  confidence: number
  constraints: string[]
  guardrails: string[]
  requiresHumanReview: boolean
}

export interface CaseCollaborationAuditReport {
  kind: 'case-collaboration-audit'
  sprintRange: '121-130'
  generatedAt: string
  caseId: string
  caseNumber: string
  caseName: string
  operatorAlias: string
  rolePermissions: CollaborationRolePermission[]
  optimisticLock: TlantiCaseCollaborationState['reviewLock']
  counts: {
    comments: number
    decisions: number
    approvals: number
    notifications: number
  }
  timeline: CaseTimelineEvent[]
  runtimeDiagnostics?: RuntimeDiagnostics | null
}

export interface AutoPlanningReviewPackage {
  kind: 'implant-autoplanning-review-package'
  sprintRange: '131-140'
  generatedAt: string
  caseId: string
  caseNumber: string
  caseName: string
  readiness: ReturnType<typeof getClinicalAssetReadiness>
  suggestions: ImplantAutoPlanningSuggestion[]
  feedback: TlantiCaseCollaborationState['autoPlanningFeedback']
  summary: {
    accepted: number
    rejected: number
    review: number
    suggested: number
  }
}

const ROLE_PERMISSIONS: CollaborationRolePermission[] = [
  { role: 'designer', label: 'Designer', canComment: true, canApprove: false, canExportAudit: true, canAcceptAutoPlan: true },
  { role: 'reviewer', label: 'Reviewer', canComment: true, canApprove: true, canExportAudit: true, canAcceptAutoPlan: true },
  { role: 'clinician', label: 'Clinician', canComment: true, canApprove: true, canExportAudit: true, canAcceptAutoPlan: false },
  { role: 'admin', label: 'Admin', canComment: true, canApprove: true, canExportAudit: true, canAcceptAutoPlan: true },
]

function buildCaseEvents(activeCase: TlantiCase): CaseTimelineEvent[] {
  const collaboration = activeCase.collaboration
  const events: CaseTimelineEvent[] = [
    {
      id: `${activeCase.id}-created`,
      kind: 'case',
      title: 'Case created',
      detail: `Case ${activeCase.caseNumber} was created in TlantiDB.`,
      actor: activeCase.technicianName || 'System',
      createdAt: activeCase.createdAt,
    },
  ]

  if (activeCase.lastOpenedAt) {
    events.push({
      id: `${activeCase.id}-opened`,
      kind: 'case',
      title: 'Case opened',
      detail: 'The active case was opened for review or editing.',
      actor: activeCase.technicianName || 'Operator',
      createdAt: activeCase.lastOpenedAt,
    })
  }

  if (activeCase.lastInteropXmlPath) {
    events.push({
      id: `${activeCase.id}-interop`,
      kind: 'interop',
      title: 'Interop XML generated',
      detail: activeCase.lastInteropXmlPath,
      actor: 'TlantiCAD Studio',
      createdAt: activeCase.updatedAt,
    })
  }

  if (activeCase.lastExportedAt) {
    events.push({
      id: `${activeCase.id}-export`,
      kind: 'export',
      title: 'Case export completed',
      detail: activeCase.storagePath ?? 'Export generated',
      actor: 'TlantiCAD Studio',
      createdAt: activeCase.lastExportedAt,
    })
  }

  collaboration?.comments.forEach((comment: TlantiCaseComment) => {
    events.push({
      id: comment.id,
      kind: 'comment',
      title: 'Clinical comment',
      detail: comment.message,
      actor: `${comment.author} · ${comment.role}`,
      createdAt: comment.createdAt,
    })
  })

  collaboration?.decisions.forEach((decision: TlantiCaseDecision) => {
    events.push({
      id: decision.id,
      kind: 'decision',
      title: decision.label,
      detail: `${decision.status} · ${decision.rationale}`,
      actor: decision.actor,
      createdAt: decision.createdAt,
    })
  })

  collaboration?.approvals.forEach((approval: TlantiCaseApproval) => {
    events.push({
      id: approval.id,
      kind: 'approval',
      title: approval.label,
      detail: `${approval.status}${approval.note ? ` · ${approval.note}` : ''}`,
      actor: approval.signer,
      createdAt: approval.signedAt ?? activeCase.updatedAt,
    })
  })

  collaboration?.notifications.forEach((notification: TlantiCaseNotification) => {
    events.push({
      id: notification.id,
      kind: 'notification',
      title: notification.title,
      detail: notification.detail,
      actor: 'Workflow engine',
      createdAt: notification.createdAt,
    })
  })

  return events.sort((left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime())
}

function inferAxisForTooth(toothNumber: string, activeJaw: TlantiCase['activeJaw']) {
  const numeric = Number(toothNumber)
  const quadrant = Math.floor(numeric / 10)
  const anterior = [1, 2, 3].includes(numeric % 10)

  if (activeJaw === 'upper') {
    return anterior ? `Palatal-emergence / quadrant ${quadrant}` : `Axial crest-following / quadrant ${quadrant}`
  }

  return anterior ? `Lingual-emergence / quadrant ${quadrant}` : `Axial crest-following / quadrant ${quadrant}`
}

function inferInitialPosition(toothNumber: string, implantMode: string, hasOpposingRecord: boolean) {
  const anterior = ['11', '12', '13', '21', '22', '23', '31', '32', '33', '41', '42', '43'].includes(toothNumber)
  if (implantMode === 'screw-retained') {
    return anterior ? 'Incisal access bias with restorative screw channel protection' : 'Central fossa bias for screw-retained restoration'
  }

  return anterior
    ? hasOpposingRecord ? 'Cingulum-aligned provisional position with occlusal check' : 'Provisional anterior position pending occlusal confirmation'
    : hasOpposingRecord ? 'Central ridge position with occlusal verification' : 'Central ridge position pending antagonist record'
}

export function deriveImplantAutoPlanningSuggestions(activeCase: TlantiCase): ImplantAutoPlanningSuggestion[] {
  const readiness = getClinicalAssetReadiness(activeCase.assets ?? [])

  return getImplantTargets(activeCase).map((target) => {
    const constraints = [
      !readiness.hasDicomStudy ? 'No DICOM / CBCT study attached yet.' : null,
      !readiness.hasOpposingRecord ? 'Opposing / bite record missing.' : null,
      target.needsExtraGingivaScan && !readiness.hasGingivaScan ? 'Soft tissue scan requested for this tooth.' : null,
      target.usesPreOpModel && !readiness.hasPrepScan ? 'Pre-op / prep scan requested but not attached.' : null,
    ].filter(Boolean) as string[]

    const guardrails = [
      'Human approval required before converting suggestion into a clinical plan.',
      'Do not rely on the suggestion without cross-checking DICOM and restorative intent.',
      target.needsExtraGingivaScan ? 'Review soft tissue emergence before acceptance.' : 'Validate emergence profile before acceptance.',
    ]

    const confidence = Math.max(25, Math.min(92,
      45
      + (readiness.hasDicomStudy ? 20 : 0)
      + (readiness.hasOpposingRecord ? 10 : 0)
      + (readiness.hasPrepScan ? 10 : 0)
      + (readiness.hasGingivaScan ? 7 : 0)
      - (constraints.length * 9),
    ))

    return {
      toothNumber: target.toothNumber,
      implantMode: target.implantMode,
      restorationLabel: target.restorationLabel,
      suggestedAxis: inferAxisForTooth(target.toothNumber, activeCase.activeJaw),
      suggestedPosition: inferInitialPosition(target.toothNumber, target.implantMode, readiness.hasOpposingRecord),
      confidence,
      constraints,
      guardrails,
      requiresHumanReview: true,
    }
  })
}

export function buildCaseCollaborationAuditReport(activeCase: TlantiCase, state: TlantiDbState, diagnostics?: RuntimeDiagnostics | null): CaseCollaborationAuditReport {
  const collaboration = activeCase.collaboration
  return {
    kind: 'case-collaboration-audit',
    sprintRange: '121-130',
    generatedAt: new Date().toISOString(),
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    caseName: activeCase.name,
    operatorAlias: state.preferences.operatorAlias,
    rolePermissions: ROLE_PERMISSIONS,
    optimisticLock: collaboration?.reviewLock ?? null,
    counts: {
      comments: collaboration?.comments.length ?? 0,
      decisions: collaboration?.decisions.length ?? 0,
      approvals: collaboration?.approvals.length ?? 0,
      notifications: collaboration?.notifications.length ?? 0,
    },
    timeline: buildCaseEvents(activeCase),
    runtimeDiagnostics: diagnostics ?? null,
  }
}

export function buildAutoPlanningReviewPackage(activeCase: TlantiCase): AutoPlanningReviewPackage {
  const suggestions = deriveImplantAutoPlanningSuggestions(activeCase)
  const feedback = activeCase.collaboration?.autoPlanningFeedback ?? {}
  const summary = {
    accepted: 0,
    rejected: 0,
    review: 0,
    suggested: 0,
  }

  suggestions.forEach((suggestion) => {
    const status = feedback[suggestion.toothNumber]?.status ?? 'suggested'
    if (status === 'accepted') summary.accepted += 1
    if (status === 'rejected') summary.rejected += 1
    if (status === 'review') summary.review += 1
    if (status === 'suggested') summary.suggested += 1
  })

  return {
    kind: 'implant-autoplanning-review-package',
    sprintRange: '131-140',
    generatedAt: new Date().toISOString(),
    caseId: activeCase.id,
    caseNumber: activeCase.caseNumber,
    caseName: activeCase.name,
    readiness: getClinicalAssetReadiness(activeCase.assets ?? []),
    suggestions,
    feedback,
    summary,
  }
}

export function exportCaseCollaborationAudit(report: CaseCollaborationAuditReport) {
  downloadJsonBrief(`${report.caseNumber}-collaboration-audit-121-130.json`, report)
}

export function exportAutoPlanningReviewPackage(pkg: AutoPlanningReviewPackage) {
  downloadJsonBrief(`${pkg.caseNumber}-autoplanning-review-131-140.json`, pkg)
}

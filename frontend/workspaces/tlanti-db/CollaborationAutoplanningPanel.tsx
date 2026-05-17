import React, { useMemo, useState } from 'react'
import { BadgeCheck, Bot, FileJson2, GitBranch, Lock, MessageSquare, ShieldCheck, Sparkles, TimerReset } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import {
  buildAutoPlanningReviewPackage,
  buildCaseCollaborationAuditReport,
  exportAutoPlanningReviewPackage,
  exportCaseCollaborationAudit,
} from '@/lib/case-collaboration-autoplanning'
import type { RuntimeDiagnostics } from '@/lib/runtime-diagnostics'
import { cn } from '@/lib/utils'
import type {
  TlantiCase,
  TlantiCaseApprovalStatus,
  TlantiCaseAutoPlanStatus,
  TlantiCaseCollaboratorRole,
  TlantiDbState,
} from '@/stores/tlantidb-case-store'

interface CollaborationAutoplanningPanelProps {
  activeCase: TlantiCase
  state: TlantiDbState
  diagnostics: RuntimeDiagnostics | null
  onPatchCollaboration: (updater: (current: NonNullable<TlantiCase['collaboration']>) => NonNullable<TlantiCase['collaboration']>) => void
}

const ROLE_OPTIONS: TlantiCaseCollaboratorRole[] = ['designer', 'reviewer', 'clinician', 'admin']

export function CollaborationAutoplanningPanel({ activeCase, state, diagnostics, onPatchCollaboration }: CollaborationAutoplanningPanelProps) {
  const [comment, setComment] = useState('')
  const [decisionNote, setDecisionNote] = useState('')
  const [approvalNote, setApprovalNote] = useState('')
  const [activeRole, setActiveRole] = useState<TlantiCaseCollaboratorRole>('reviewer')

  const collaboration = activeCase.collaboration!
  const auditReport = useMemo(() => buildCaseCollaborationAuditReport(activeCase, state, diagnostics), [activeCase, diagnostics, state])
  const autoPlanningPackage = useMemo(() => buildAutoPlanningReviewPackage(activeCase), [activeCase])

  const pushNotification = (title: string, detail: string, kind: 'workflow' | 'review' | 'interop' | 'autoplan') => {
    onPatchCollaboration((current) => ({
      ...current,
      notifications: [
        {
          id: crypto.randomUUID(),
          kind,
          title,
          detail,
          createdAt: new Date().toISOString(),
          read: false,
        },
        ...current.notifications,
      ].slice(0, 24),
    }))
  }

  const toggleLock = () => {
    onPatchCollaboration((current) => ({
      ...current,
      reviewLock: current.reviewLock
        ? null
        : {
            holder: state.preferences.operatorAlias,
            role: activeRole,
            acquiredAt: new Date().toISOString(),
            machineSignature: state.preferences.performanceProfile.machineSignature ?? null,
          },
    }))

    pushNotification(
      collaboration.reviewLock ? 'Review lock released' : 'Review lock acquired',
      collaboration.reviewLock
        ? `${state.preferences.operatorAlias} released the optimistic case lock.`
        : `${state.preferences.operatorAlias} acquired the optimistic case lock as ${activeRole}.`,
      'review',
    )
  }

  const addComment = () => {
    const message = comment.trim()
    if (!message) return

    onPatchCollaboration((current) => ({
      ...current,
      comments: [
        {
          id: crypto.randomUUID(),
          author: state.preferences.operatorAlias,
          role: activeRole,
          message,
          createdAt: new Date().toISOString(),
          toothNumber: null,
        },
        ...current.comments,
      ],
    }))
    setComment('')
    pushNotification('New clinical comment', message, 'review')
  }

  const addDecision = (status: 'approved' | 'rejected') => {
    const rationale = decisionNote.trim() || (status === 'approved' ? 'Approved after collaborative review.' : 'Rejected pending changes or more evidence.')
    onPatchCollaboration((current) => ({
      ...current,
      decisions: [
        {
          id: crypto.randomUUID(),
          actor: state.preferences.operatorAlias,
          label: status === 'approved' ? 'Case review approved' : 'Case review rejected',
          rationale,
          status,
          createdAt: new Date().toISOString(),
        },
        ...current.decisions,
      ],
    }))
    setDecisionNote('')
    pushNotification(status === 'approved' ? 'Review approved' : 'Review rejected', rationale, 'workflow')
  }

  const signApproval = (status: TlantiCaseApprovalStatus) => {
    const note = approvalNote.trim()
    onPatchCollaboration((current) => ({
      ...current,
      approvals: [
        {
          id: crypto.randomUUID(),
          label: 'Internal clinical sign-off',
          signer: state.preferences.operatorAlias,
          status,
          note,
          signedAt: new Date().toISOString(),
          machineSignature: state.preferences.performanceProfile.machineSignature ?? null,
        },
        ...current.approvals,
      ],
    }))
    setApprovalNote('')
    pushNotification(status === 'signed' ? 'Internal sign-off completed' : 'Internal sign-off rejected', note || 'No note provided.', 'review')
  }

  const updateSuggestionStatus = (toothNumber: string, status: TlantiCaseAutoPlanStatus) => {
    onPatchCollaboration((current) => ({
      ...current,
      autoPlanningFeedback: {
        ...current.autoPlanningFeedback,
        [toothNumber]: {
          status,
          note: current.autoPlanningFeedback[toothNumber]?.note,
          updatedAt: new Date().toISOString(),
        },
      },
    }))
    pushNotification(`Auto-plan ${status}`, `Tooth ${toothNumber} suggestion marked as ${status}.`, 'autoplan')
  }

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <GitBranch className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary text-balance">Collaboration + auto-planning review</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Tranche operativo de los sprints <span className="font-medium text-text-primary">121–140</span>: comentarios clínicos, lock optimista, decisiones, firma interna y sugerencias automáticas de implant planning con control humano.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">121–130 collaboration</Badge>
          <Badge variant="outline">131–140 auto-plan</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-4 xl:grid-cols-[1.02fr_0.98fr]">
        <div className="space-y-4">
          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center justify-between gap-3">
              <div>
                <p className="text-xs uppercase text-text-secondary">Optimistic review lock</p>
                <p className="text-sm text-text-primary text-pretty">Soft lock for clinical review and handoff control.</p>
              </div>
              <Badge variant="outline">{collaboration.reviewLock ? 'Locked' : 'Unlocked'}</Badge>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Role</span>
                <select value={activeRole} onChange={(event) => setActiveRole(event.target.value as TlantiCaseCollaboratorRole)} className="rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary" title="Collaboration role" aria-label="Collaboration role">
                  {ROLE_OPTIONS.map((role) => <option key={role} value={role}>{role}</option>)}
                </select>
              </label>
              <div className="rounded-2xl border border-border bg-card px-3 py-3">
                <p className="text-[11px] uppercase text-text-secondary">Current holder</p>
                <p className="mt-1 text-sm font-semibold text-text-display">{collaboration.reviewLock?.holder ?? 'Nobody'}</p>
                <p className="mt-1 text-xs text-text-secondary">{collaboration.reviewLock?.acquiredAt ? new Date(collaboration.reviewLock.acquiredAt).toLocaleString() : 'Lock available'}</p>
              </div>
            </div>
            <button type="button" onClick={toggleLock} className="mt-3 inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">
              <Lock className="size-4" /> {collaboration.reviewLock ? 'Release review lock' : 'Acquire review lock'}
            </button>
          </section>

          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <MessageSquare className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Clinical comments + decisions</p>
            </div>
            <textarea
              value={comment}
              onChange={(event) => setComment(event.target.value)}
              placeholder="Add a review note, blocker or clinical observation…"
              className="min-h-24 w-full rounded-xl border border-border-visible bg-card px-3 py-3 text-sm text-text-primary outline-none transition-colors placeholder:text-text-secondary focus:border-text-primary"
            />
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" onClick={addComment} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Add comment</button>
              <input value={decisionNote} onChange={(event) => setDecisionNote(event.target.value)} placeholder="Decision rationale" className="min-w-[12rem] flex-1 rounded-2xl border border-border-visible bg-card px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Decision rationale" />
              <button type="button" onClick={() => addDecision('approved')} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Approve</button>
              <button type="button" onClick={() => addDecision('rejected')} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Reject</button>
            </div>

            <div className="mt-4 space-y-2">
              {collaboration.comments.slice(0, 4).map((item) => (
                <div key={item.id} className="rounded-xl border border-border bg-card px-3 py-3">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-semibold text-text-display">{item.author}</p>
                    <Badge variant="outline">{item.role}</Badge>
                  </div>
                  <p className="mt-2 text-xs text-text-secondary text-pretty">{item.message}</p>
                </div>
              ))}
              {collaboration.decisions.slice(0, 3).map((item) => (
                <div key={item.id} className="rounded-xl border border-border bg-card px-3 py-3">
                  <p className="text-sm font-semibold text-text-display">{item.label}</p>
                  <p className="mt-1 text-xs text-text-secondary text-pretty">{item.rationale}</p>
                </div>
              ))}
            </div>
          </section>

          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <BadgeCheck className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Internal signature + audit export</p>
            </div>
            <input value={approvalNote} onChange={(event) => setApprovalNote(event.target.value)} placeholder="Approval note" className="w-full rounded-2xl border border-border-visible bg-card px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary" aria-label="Approval note" />
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" onClick={() => signApproval('signed')} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Sign internally</button>
              <button type="button" onClick={() => signApproval('rejected')} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">Reject sign-off</button>
              <button type="button" onClick={() => exportCaseCollaborationAudit(auditReport)} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised inline-flex items-center gap-2"><FileJson2 className="size-4" />Export audit JSON</button>
            </div>
          </section>
        </div>

        <div className="space-y-4">
          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <Bot className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Auto-planning suggestions</p>
            </div>
            <div className="space-y-2">
              {autoPlanningPackage.suggestions.length ? autoPlanningPackage.suggestions.map((suggestion) => {
                const feedback = collaboration.autoPlanningFeedback[suggestion.toothNumber]
                return (
                  <div key={suggestion.toothNumber} className="rounded-xl border border-border bg-card px-3 py-3">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="text-sm font-semibold text-text-display">Tooth {suggestion.toothNumber} · {suggestion.restorationLabel}</p>
                        <p className="mt-1 text-xs text-text-secondary text-pretty">{suggestion.suggestedAxis}</p>
                        <p className="mt-1 text-xs text-text-secondary text-pretty">{suggestion.suggestedPosition}</p>
                      </div>
                      <Badge variant="outline">{suggestion.confidence}%</Badge>
                    </div>
                    <div className="mt-2 space-y-1 text-xs text-text-secondary">
                      {suggestion.constraints.map((constraint) => <p key={constraint}>• {constraint}</p>)}
                      {suggestion.guardrails.map((guardrail) => <p key={guardrail}>• {guardrail}</p>)}
                    </div>
                    <div className="mt-3 flex flex-wrap gap-2">
                      <button type="button" onClick={() => updateSuggestionStatus(suggestion.toothNumber, 'accepted')} className="rounded-2xl border border-border bg-surface px-3 py-1.5 text-xs text-text-primary transition-colors hover:bg-surface-raised">Accept</button>
                      <button type="button" onClick={() => updateSuggestionStatus(suggestion.toothNumber, 'review')} className="rounded-2xl border border-border bg-surface px-3 py-1.5 text-xs text-text-primary transition-colors hover:bg-surface-raised">Needs review</button>
                      <button type="button" onClick={() => updateSuggestionStatus(suggestion.toothNumber, 'rejected')} className="rounded-2xl border border-border bg-surface px-3 py-1.5 text-xs text-text-primary transition-colors hover:bg-surface-raised">Reject</button>
                      <Badge variant="outline">{feedback?.status ?? 'suggested'}</Badge>
                    </div>
                  </div>
                )
              }) : (
                <div className="rounded-xl border border-dashed border-border bg-card px-3 py-4 text-sm text-text-secondary">No implant targets exist in the active case yet, so auto-planning suggestions stay intentionally empty.</div>
              )}
            </div>
            <button type="button" onClick={() => exportAutoPlanningReviewPackage(autoPlanningPackage)} className="mt-3 inline-flex items-center gap-2 rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised">
              <Sparkles className="size-4" /> Export auto-plan package
            </button>
          </section>

          <section className="rounded-2xl border border-border bg-surface px-4 py-4">
            <div className="mb-3 flex items-center gap-2">
              <TimerReset className="size-4 text-text-secondary" />
              <p className="text-xs uppercase text-text-secondary">Case timeline + notifications</p>
            </div>
            <div className="space-y-2">
              {auditReport.timeline.slice(0, 8).map((event) => (
                <div key={event.id} className="rounded-xl border border-border bg-card px-3 py-3">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-semibold text-text-display">{event.title}</p>
                    <Badge variant="outline">{event.kind}</Badge>
                  </div>
                  <p className="mt-1 text-xs text-text-secondary text-pretty">{event.detail}</p>
                  <p className="mt-2 text-[11px] uppercase text-text-secondary">{event.actor}</p>
                </div>
              ))}
            </div>

            <div className="mt-4 rounded-xl border border-border bg-card px-3 py-3">
              <div className="flex items-center justify-between gap-2">
                <p className="text-sm font-semibold text-text-display">Unread notifications</p>
                <ShieldCheck className="size-4 text-text-secondary" />
              </div>
              <p className="mt-2 text-xs text-text-secondary">{collaboration.notifications.filter((item) => !item.read).length} unread workflow items</p>
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}
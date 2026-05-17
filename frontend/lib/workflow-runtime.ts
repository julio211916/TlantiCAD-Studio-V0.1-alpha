import { invoke } from '@tauri-apps/api/core';

import type { WorkflowStepId } from '@/core';
import { isTauriRuntime } from '@/platform/desktop-system';

export type WorkflowJobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'manual-review';

export interface WorkflowStartRequest {
  caseId: string;
  moduleId: string;
  workflowStepId: WorkflowStepId;
  inputAssetIds: string[];
  params: Record<string, unknown>;
}

export interface WorkflowJobSnapshot {
  id: string;
  caseId: string;
  kind: string;
  status: WorkflowJobStatus;
  progress: number;
  runtime: string;
  artifacts: unknown[];
  error?: string | null;
}

function disabledSnapshot(stepId: WorkflowStepId, caseId: string): WorkflowJobSnapshot {
  return {
    id: `browser-workflow-${stepId}`,
    caseId,
    kind: `workflow.${stepId}`,
    status: 'manual-review',
    progress: 0,
    runtime: 'browser-fallback',
    artifacts: [],
    error: 'Workflow jobs require the Tauri desktop runtime.',
  };
}

export async function startWorkflow(request: WorkflowStartRequest): Promise<WorkflowJobSnapshot> {
  if (!isTauriRuntime()) {
    return disabledSnapshot(request.workflowStepId, request.caseId);
  }

  return invoke<WorkflowJobSnapshot>('workflow_start', { request });
}

export async function getWorkflowStatus(jobId: string): Promise<WorkflowJobSnapshot> {
  return invoke<WorkflowJobSnapshot>('workflow_status', { jobId });
}

export async function cancelWorkflow(jobId: string): Promise<WorkflowJobSnapshot> {
  return invoke<WorkflowJobSnapshot>('workflow_cancel', { jobId });
}

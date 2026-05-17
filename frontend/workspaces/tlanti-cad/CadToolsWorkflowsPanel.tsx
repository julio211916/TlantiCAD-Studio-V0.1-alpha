import React, { useEffect, useMemo, useState } from 'react';
import { Activity, CheckCircle2, Cpu, Loader2, Play, Route, XCircle } from 'lucide-react';

import {
  CAD_TOOL_DEFINITIONS,
  WORKFLOW_STEP_DEFINITIONS,
  isWorkflowStepId,
  type TlantiCadToolDefinition,
  type WorkflowStepId,
} from '@/core';
import { cn } from '@/lib/utils';
import {
  loadRuntimeToolRegistry,
  type RuntimeToolRegistrySnapshot,
} from '@/lib/tool-registry-runtime';
import { cancelWorkflow, startWorkflow, type WorkflowJobSnapshot } from '@/lib/workflow-runtime';

interface CadToolsWorkflowsPanelProps {
  caseId?: string | null;
  activeModuleId: string;
  activeWorkflowPhaseId: string;
  onClose: () => void;
}

const statusTone = {
  ready: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-100',
  planned: 'border-amber-400/30 bg-amber-400/10 text-amber-100',
  disabled: 'border-white/10 bg-white/5 text-white/45',
} as const;

function summarizeTools(tools: readonly TlantiCadToolDefinition[]) {
  return tools.reduce(
    (summary, tool) => {
      summary[tool.status] += 1;
      return summary;
    },
    { ready: 0, planned: 0, disabled: 0 },
  );
}

export default function CadToolsWorkflowsPanel({
  caseId,
  activeModuleId,
  activeWorkflowPhaseId,
  onClose,
}: CadToolsWorkflowsPanelProps) {
  const [runtimeRegistry, setRuntimeRegistry] = useState<RuntimeToolRegistrySnapshot | null>(null);
  const [loadingRuntime, setLoadingRuntime] = useState(true);
  const [runtimeError, setRuntimeError] = useState<string | null>(null);
  const [activeWorkflowStepId, setActiveWorkflowStepId] = useState<WorkflowStepId>(() => (
    isWorkflowStepId(activeWorkflowPhaseId) ? activeWorkflowPhaseId : 'design'
  ));
  const [workflowJob, setWorkflowJob] = useState<WorkflowJobSnapshot | null>(null);
  const [workflowBusy, setWorkflowBusy] = useState(false);
  const [workflowError, setWorkflowError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoadingRuntime(true);
    setRuntimeError(null);

    loadRuntimeToolRegistry()
      .then((snapshot) => {
        if (!cancelled) setRuntimeRegistry(snapshot);
      })
      .catch((error) => {
        if (!cancelled) setRuntimeError(error instanceof Error ? error.message : String(error));
      })
      .finally(() => {
        if (!cancelled) setLoadingRuntime(false);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const topTools = useMemo(() => {
    return CAD_TOOL_DEFINITIONS
      .filter((tool) => tool.placements.some((placement) => placement === 'top-dock' || placement === 'rail' || placement === 'panel'))
      .slice(0, 18);
  }, []);

  const toolSummary = useMemo(() => summarizeTools(CAD_TOOL_DEFINITIONS), []);
  const runtimeToolIds = useMemo(() => new Set((runtimeRegistry?.tools ?? []).map((tool) => tool.id)), [runtimeRegistry]);
  const selectedWorkflow = useMemo(
    () => WORKFLOW_STEP_DEFINITIONS.find((step) => step.id === activeWorkflowStepId) ?? WORKFLOW_STEP_DEFINITIONS[0],
    [activeWorkflowStepId],
  );

  async function handleStartWorkflow() {
    setWorkflowBusy(true);
    setWorkflowError(null);

    try {
      const snapshot = await startWorkflow({
        caseId: caseId?.trim() || 'local-workspace',
        moduleId: activeModuleId,
        workflowStepId: selectedWorkflow.id,
        inputAssetIds: [],
        params: {
          source: 'cad-tools-workflows-panel',
          performanceContract: selectedWorkflow.performanceContract,
        },
      });
      setWorkflowJob(snapshot);
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error));
    } finally {
      setWorkflowBusy(false);
    }
  }

  async function handleCancelWorkflow() {
    if (!workflowJob) return;
    setWorkflowBusy(true);
    setWorkflowError(null);

    try {
      setWorkflowJob(await cancelWorkflow(workflowJob.id));
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error));
    } finally {
      setWorkflowBusy(false);
    }
  }

  return (
    <section
      className="pointer-events-auto absolute left-[5.7rem] top-[8.4rem] z-40 w-[min(35rem,calc(100vw-7rem))] rounded-md border border-white/10 bg-[#08090b]/95 text-white shadow-2xl backdrop-blur-xl"
      data-visual-qa-tools-workflows="true"
      aria-label="Tools and workflows registry"
    >
      <div className="flex items-start justify-between gap-4 border-b border-white/10 px-4 py-3">
        <div>
          <div className="flex items-center gap-2">
            <Route className="size-4 text-cyan-200" />
            <h2 className="text-sm font-semibold">Tools & Workflows</h2>
          </div>
          <p className="mt-1 text-xs text-white/50">
            {activeModuleId} · {activeWorkflowPhaseId} · React/Three → Tauri → Rust/Python
          </p>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded border border-white/10 p-1.5 text-white/60 transition-colors hover:bg-white/10 hover:text-white"
          aria-label="Close tools and workflows panel"
        >
          <XCircle className="size-4" />
        </button>
      </div>

      <div className="grid gap-3 px-4 py-4">
        <div className="grid grid-cols-3 gap-2 text-xs">
          <div className="rounded border border-emerald-400/20 bg-emerald-400/10 px-3 py-2">
            <p className="font-mono uppercase tracking-[0.14em] text-emerald-100/60">Ready</p>
            <p className="mt-1 text-xl font-semibold">{toolSummary.ready}</p>
          </div>
          <div className="rounded border border-amber-400/20 bg-amber-400/10 px-3 py-2">
            <p className="font-mono uppercase tracking-[0.14em] text-amber-100/60">Planned</p>
            <p className="mt-1 text-xl font-semibold">{toolSummary.planned}</p>
          </div>
          <div className="rounded border border-white/10 bg-white/5 px-3 py-2">
            <p className="font-mono uppercase tracking-[0.14em] text-white/45">Runtime</p>
            <p className="mt-1 flex items-center gap-2 text-sm font-semibold">
              {loadingRuntime ? <Loader2 className="size-4 animate-spin" /> : <Cpu className="size-4" />}
              {runtimeRegistry ? `${runtimeRegistry.tools.length} core` : 'local UI'}
            </p>
          </div>
        </div>

        {runtimeError ? (
          <div className="rounded border border-red-400/30 bg-red-500/10 px-3 py-2 text-xs text-red-100">
            Runtime registry unavailable: {runtimeError}
          </div>
        ) : null}

        <div className="grid gap-2">
          <div className="flex items-center gap-2 text-[11px] font-mono uppercase tracking-[0.18em] text-white/45">
            <Activity className="size-3.5" />
            Clinical flow
          </div>
          <div className="grid grid-cols-6 gap-1">
            {WORKFLOW_STEP_DEFINITIONS.map((step) => (
              <button
                type="button"
                key={step.id}
                onClick={() => setActiveWorkflowStepId(step.id)}
                className={cn(
                  'min-w-0 rounded border px-2 py-2 text-center text-[11px]',
                  step.id === activeWorkflowStepId
                    ? 'border-cyan-300/40 bg-cyan-300/10 text-cyan-100'
                    : 'border-white/10 bg-white/[0.03] text-white/55',
                )}
                title={step.performanceContract}
              >
                <p className="truncate font-semibold">{step.label}</p>
                <p className="mt-1 truncate font-mono uppercase tracking-[0.12em] opacity-60">{step.computeMode}</p>
              </button>
            ))}
          </div>
        </div>

        <div className="rounded border border-white/10 bg-white/[0.025] px-3 py-3">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0">
              <p className="truncate text-sm font-medium">{selectedWorkflow.label} workflow</p>
              <p className="mt-1 truncate text-xs text-white/45">
                {selectedWorkflow.owner} · {selectedWorkflow.computeMode} · {selectedWorkflow.localServices.join(', ')}
              </p>
            </div>
            <div className="flex items-center gap-2">
              {workflowJob && !['cancelled', 'completed', 'failed'].includes(workflowJob.status) ? (
                <button
                  type="button"
                  onClick={() => void handleCancelWorkflow()}
                  disabled={workflowBusy}
                  className="rounded border border-red-400/30 px-3 py-2 text-xs font-medium text-red-100 transition-colors hover:bg-red-500/10 disabled:opacity-50"
                >
                  Cancel
                </button>
              ) : null}
              <button
                type="button"
                onClick={() => void handleStartWorkflow()}
                disabled={workflowBusy}
                className="inline-flex items-center gap-2 rounded border border-cyan-300/35 bg-cyan-300/10 px-3 py-2 text-xs font-medium text-cyan-100 transition-colors hover:bg-cyan-300/15 disabled:opacity-50"
              >
                {workflowBusy ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
                Start job
              </button>
            </div>
          </div>
          {workflowJob ? (
            <div className="mt-3 rounded border border-white/10 bg-black/30 px-3 py-2 text-xs text-white/60">
              <span className="font-mono uppercase tracking-[0.14em] text-white/40">Job</span>{' '}
              {workflowJob.id} · {workflowJob.status} · {Math.round(workflowJob.progress * 100)}% · {workflowJob.runtime}
            </div>
          ) : null}
          {workflowError ? (
            <div className="mt-3 rounded border border-red-400/30 bg-red-500/10 px-3 py-2 text-xs text-red-100">
              {workflowError}
            </div>
          ) : null}
        </div>

        <div className="grid max-h-[19rem] gap-1 overflow-auto pr-1">
          {topTools.map((tool) => {
            const isRuntimeKnown = runtimeToolIds.has(tool.id);
            return (
              <div key={tool.id} className="grid grid-cols-[1fr_auto] gap-3 rounded border border-white/10 bg-white/[0.025] px-3 py-2">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="truncate text-sm font-medium">{tool.label}</p>
                    {isRuntimeKnown ? <CheckCircle2 className="size-3.5 text-emerald-200" /> : null}
                  </div>
                  <p className="mt-1 truncate text-xs text-white/45">
                    {tool.runtime} · {tool.commandId}{tool.runtimeCommand ? ` → ${tool.runtimeCommand}` : ''}
                  </p>
                </div>
                <span className={cn('self-center rounded border px-2 py-1 text-[11px] font-mono uppercase tracking-[0.12em]', statusTone[tool.status])}>
                  {tool.status}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

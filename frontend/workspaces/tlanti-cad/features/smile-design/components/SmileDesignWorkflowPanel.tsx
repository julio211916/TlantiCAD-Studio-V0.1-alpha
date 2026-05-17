import React, { useMemo } from 'react';
import {
  ArrowLeft,
  ArrowRight,
  CheckSquare,
  ClipboardList,
  Layers3,
  RotateCcw,
  Smile,
  Wand2,
} from 'lucide-react';

import type { ToolMode } from '@/types';
import {
  SMILE_DESIGN_PLAYBOOKS,
  getSmileDesignPlaybookById,
  getSmileDesignStageTaskIds,
} from '@/lib/smile-design-playbooks';
import { useSmileDesignWorkflowStore } from '@/stores/smile-design-workflow-store';
import { cn } from '@/lib/utils';

import { CadResponsivePanel } from '@/components/cad/CadResponsivePanel';

interface SmileDesignWorkflowPanelProps {
  compact: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  activeTool: ToolMode;
  onSetTool: (tool: ToolMode) => void;
}

const TOOL_LABELS: Record<ToolMode, string> = {
  SELECT: 'Select',
  MOVE: 'Move',
  ROTATE: 'Rotate',
  SCALE: 'Scale',
  CLIP: 'Clip',
  MEASURE: 'Measure',
  SEGMENT: 'Segment',
  SCULPT: 'Sculpt',
  VOXELIZE: 'Voxelize',
  POINT_CLOUD: 'Point cloud',
  BOOLEAN_CUT: 'Boolean cut',
  CROP: 'Crop',
};

export default function SmileDesignWorkflowPanel({
  compact,
  open = true,
  onOpenChange,
  activeTool,
  onSetTool,
}: SmileDesignWorkflowPanelProps) {
  const selectedPlaybookId = useSmileDesignWorkflowStore((state) => state.selectedPlaybookId);
  const currentStageByPlaybook = useSmileDesignWorkflowStore((state) => state.currentStageByPlaybook);
  const completedTaskIds = useSmileDesignWorkflowStore((state) => state.completedTaskIds);
  const notesByPlaybook = useSmileDesignWorkflowStore((state) => state.notesByPlaybook);
  const selectPlaybook = useSmileDesignWorkflowStore((state) => state.selectPlaybook);
  const goToStage = useSmileDesignWorkflowStore((state) => state.goToStage);
  const nextStage = useSmileDesignWorkflowStore((state) => state.nextStage);
  const previousStage = useSmileDesignWorkflowStore((state) => state.previousStage);
  const toggleTask = useSmileDesignWorkflowStore((state) => state.toggleTask);
  const setNotes = useSmileDesignWorkflowStore((state) => state.setNotes);
  const resetPlaybook = useSmileDesignWorkflowStore((state) => state.resetPlaybook);

  const playbook = getSmileDesignPlaybookById(selectedPlaybookId);
  const currentStageIndex = currentStageByPlaybook[selectedPlaybookId] ?? 0;
  const currentStage = playbook.stages[currentStageIndex];
  const stageTaskIds = useMemo(() => getSmileDesignStageTaskIds(playbook, currentStageIndex), [playbook, currentStageIndex]);
  const completedCount = stageTaskIds.filter((taskId) => completedTaskIds[taskId]).length;
  const stageProgress = stageTaskIds.length ? Math.round((completedCount / stageTaskIds.length) * 100) : 0;
  const notes = notesByPlaybook[selectedPlaybookId] ?? '';
  const progressWidthClass = useMemo(() => {
    if (stageProgress >= 100) return 'w-full';
    if (stageProgress >= 75) return 'w-3/4';
    if (stageProgress >= 50) return 'w-1/2';
    if (stageProgress >= 25) return 'w-1/4';
    if (stageProgress > 0) return 'w-2';
    return 'w-0';
  }, [stageProgress]);

  return (
    <CadResponsivePanel
      title="Smile / Waxup workflow"
      compact={compact}
      open={open}
      onOpenChange={onOpenChange}
      desktopClassName="left-4 top-[7.2rem] bottom-12 w-[20rem]"
      contentClassName="space-y-3"
      headerAction={<Smile className="size-4 text-text-secondary" />}
    >
      <div className="rounded-2xl border border-white/8 bg-[#14181d] px-4 py-4 shadow-sm">
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="text-[11px] uppercase text-text-secondary">Active playbook</p>
            <p className="mt-1 text-sm font-semibold text-text-primary">{playbook.title}</p>
            <p className="mt-1 text-[11px] text-text-secondary">{playbook.subtitle}</p>
          </div>
          <span className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] text-text-secondary">
            {currentStageIndex + 1}/{playbook.stages.length}
          </span>
        </div>
        <div className="mt-3 flex flex-wrap gap-2">
            {SMILE_DESIGN_PLAYBOOKS.map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => selectPlaybook(item.id)}
                className={cn(
                  'min-w-fit rounded-full border px-2.5 py-1 text-[10px] font-mono uppercase transition-colors',
                  item.id === selectedPlaybookId
                    ? 'border-text-display bg-white text-black'
                    : 'border-white/8 bg-[#101215] text-text-secondary hover:bg-surface-raised hover:text-text-primary',
                )}
              >
                {item.title.replace('Direct print mockup ', '').replace('Conversion to ', '').replace('Mockup model', 'Model')}
              </button>
            ))}
        </div>
      </div>

      <div className="grid gap-2 grid-cols-3">
        <div className="rounded-2xl border border-white/8 bg-[#14181d] px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Current stage</p>
          <p className="mt-1 text-sm font-semibold text-text-primary">{currentStage.title}</p>
        </div>
        <div className="rounded-2xl border border-white/8 bg-[#14181d] px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Stage progress</p>
          <p className="mt-1 text-sm font-semibold text-text-primary">{stageProgress}%</p>
          <div className="mt-2 h-2 overflow-hidden rounded-full bg-surface">
            <div className={cn('h-full rounded-full bg-text-display transition-all duration-300', progressWidthClass)} />
          </div>
          <p className="mt-1 text-[11px] text-text-secondary">{completedCount}/{stageTaskIds.length}</p>
        </div>
        <div className="rounded-2xl border border-white/8 bg-[#14181d] px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Target output</p>
          <p className="mt-1 text-sm font-semibold text-text-primary text-pretty line-clamp-4">{playbook.targetOutput}</p>
        </div>
      </div>

      <div className="rounded-2xl border border-white/8 bg-[#14181d] px-4 py-4">
        <div className="mb-3 flex items-center gap-2">
          <Layers3 className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Stage brief</p>
        </div>
        <p className="text-sm text-text-primary text-pretty line-clamp-4">{currentStage.objective}</p>
        <div className="mt-3 grid gap-2">
          <div className="rounded-xl border border-white/8 bg-[#101215] px-3 py-3">
            <p className="text-[11px] uppercase text-text-secondary">Deliverable</p>
            <p className="mt-1 text-sm text-text-primary text-pretty line-clamp-3">{currentStage.deliverable}</p>
          </div>
          <div className="rounded-xl border border-white/8 bg-[#101215] px-3 py-3">
            <p className="text-[11px] uppercase text-text-secondary">Recommended CAD tool</p>
            <div className="mt-2 flex items-center justify-between gap-3">
              <p className="text-sm font-semibold text-text-primary">
                {currentStage.recommendedTool ? TOOL_LABELS[currentStage.recommendedTool] : 'Manual review'}
              </p>
              {currentStage.recommendedTool ? (
                <button
                  type="button"
                  onClick={() => onSetTool(currentStage.recommendedTool!)}
                  className={cn(
                    'rounded-full border px-3 py-1 text-[11px] font-mono uppercase transition-colors',
                    activeTool === currentStage.recommendedTool
                      ? 'border-text-display bg-text-display/10 text-text-display'
                      : 'border-border bg-card text-text-secondary hover:bg-surface-raised hover:text-text-primary',
                  )}
                >
                  {activeTool === currentStage.recommendedTool ? 'Active' : 'Use tool'}
                </button>
              ) : null}
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-white/8 bg-[#14181d] px-4 py-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2">
          <CheckSquare className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Checklist</p>
          <span className="rounded-full border border-border bg-surface px-2 py-0.5 text-[10px] text-text-secondary">{completedCount}/{stageTaskIds.length}</span>
        </div>
        <div className="space-y-2">
          {currentStage.tasks.map((task, taskIndex) => {
            const taskId = stageTaskIds[taskIndex];
            const checked = Boolean(completedTaskIds[taskId]);
            return (
              <label key={taskId} className="flex items-start gap-3 rounded-xl border border-white/8 bg-[#101215] px-3 py-2.5 transition-colors hover:bg-surface-raised">
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(event) => toggleTask(taskId, event.target.checked)}
                  className="mt-0.5 size-4 rounded border-border bg-card"
                />
                <span className={cn('text-sm text-pretty', checked ? 'text-text-secondary line-through' : 'text-text-primary')}>
                  {task}
                </span>
              </label>
            );
          })}
        </div>
      </div>

      <div className="rounded-2xl border border-white/8 bg-[#14181d] px-4 py-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2">
          <ClipboardList className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Evidence / notes</p>
        </div>
        <textarea
          value={notes}
          onChange={(event) => setNotes(selectedPlaybookId, event.target.value)}
          placeholder="Capture blockers, evidence, screenshots expected, printer limits, contact notes, etc."
          className="min-h-24 w-full rounded-xl border border-white/8 bg-[#101215] px-3 py-3 text-sm text-text-primary outline-none transition-colors placeholder:text-text-secondary focus:border-text-display"
        />
      </div>

      <div className="rounded-2xl border border-white/8 bg-[#14181d] px-4 py-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2">
          <Wand2 className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Navigator</p>
        </div>
        <div className="max-h-44 space-y-2 overflow-y-auto pr-1">
          {playbook.stages.map((stage, stageIndex) => (
            <button
              key={stage.id}
              type="button"
              onClick={() => goToStage(selectedPlaybookId, stageIndex)}
              className={cn(
                'flex w-full items-center justify-between rounded-xl border px-3 py-2 text-left text-[10px] font-mono uppercase transition-colors',
                stageIndex === currentStageIndex
                  ? 'border-text-display bg-white text-black'
                  : 'border-white/8 bg-[#101215] text-text-secondary hover:bg-surface-raised hover:text-text-primary',
              )}
            >
              <span>{stageIndex + 1}. {stage.title}</span>
              <span className="text-[10px] opacity-80">{stage.tasks.length} tasks</span>
            </button>
          ))}
        </div>

        <div className="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-3">
          <button
            type="button"
            onClick={() => previousStage(selectedPlaybookId)}
            className="inline-flex items-center justify-center rounded-full border border-border bg-surface px-3 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
          >
            <ArrowLeft className="mr-1 size-3.5" /> Prev
          </button>
          <button
            type="button"
            onClick={() => nextStage(selectedPlaybookId)}
            className="inline-flex items-center justify-center rounded-full border border-border bg-surface px-3 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
          >
            Next <ArrowRight className="ml-1 size-3.5" />
          </button>
          <button
            type="button"
            onClick={() => resetPlaybook(selectedPlaybookId)}
            className="inline-flex items-center justify-center rounded-full border border-border bg-surface px-3 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
          >
            <RotateCcw className="mr-1 size-3.5" /> Reset playbook
          </button>
        </div>
      </div>
    </CadResponsivePanel>
  );
}

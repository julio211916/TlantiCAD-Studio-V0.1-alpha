import React, { useMemo } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { CheckCircle2, ClipboardList, PlayCircle, RotateCcw, Target } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/interfaces-select';
import { Switch } from '@/components/ui/interfaces-switch';
import { DICOM_S01_P01_ACTIONABLE_PHASE } from '@/lib/dicom-dental-roadmap';
import { dicomExecutionBriefSchema, type DicomExecutionBriefForm } from '@/lib/dicom-roadmap-execution.schemas';
import { useDicomRoadmapExecutionStore } from '@/stores/dicom-roadmap-execution-store';

const VALIDATION_LABELS: Record<DicomExecutionBriefForm['validationMode'], string> = {
  clinical: 'Clinical review',
  dataset: 'Dataset readiness',
  ui: 'Viewer / UX validation',
  automation: 'Automation / CI gate',
};

export function DicomDentalExecutionPanel() {
  const brief = useDicomRoadmapExecutionStore((state) => state.brief);
  const taskStates = useDicomRoadmapExecutionStore((state) => state.taskStates);
  const setBrief = useDicomRoadmapExecutionStore((state) => state.setBrief);
  const toggleTask = useDicomRoadmapExecutionStore((state) => state.toggleTask);
  const resetPhase = useDicomRoadmapExecutionStore((state) => state.resetPhase);

  const form = useForm<DicomExecutionBriefForm>({
    resolver: zodResolver(dicomExecutionBriefSchema),
    defaultValues: brief,
    values: brief,
  });

  const completedCount = useMemo(
    () => DICOM_S01_P01_ACTIONABLE_PHASE.kickoffTasks.filter((task) => taskStates[task.id]).length,
    [taskStates],
  );
  const totalCount = DICOM_S01_P01_ACTIONABLE_PHASE.kickoffTasks.length;
  const progressPercent = Math.round((completedCount / totalCount) * 100);
  const nextTask = DICOM_S01_P01_ACTIONABLE_PHASE.kickoffTasks.find((task) => !taskStates[task.id]) ?? null;

  const submitBrief = form.handleSubmit((values) => {
    setBrief(values);
  });

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <PlayCircle className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary">Sprint 01 / Phase 01 execution board</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Conversión operativa de <span className="font-medium text-text-primary">120 tareas documentales</span> a <span className="font-medium text-text-primary">12 lanes accionables</span> para empezar a ejecutar la fase dentro de la app.
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">{DICOM_S01_P01_ACTIONABLE_PHASE.sprintId}</Badge>
          <Badge variant="outline">{DICOM_S01_P01_ACTIONABLE_PHASE.phaseId}</Badge>
          <Badge variant="outline">{completedCount}/{totalCount} lanes done</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-3">
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Source document</p>
          <p className="mt-1 font-mono text-xs text-text-primary">{DICOM_S01_P01_ACTIONABLE_PHASE.sourceDocPath}</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Phase progress</p>
          <p className="mt-1 text-sm font-semibold text-text-display">{progressPercent}%</p>
          <p className="mt-1 text-xs text-text-secondary">{completedCount} de {totalCount} lanes completados</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Next actionable lane</p>
          <p className="mt-1 text-sm font-semibold text-text-display">{nextTask?.stream ?? 'Done'}</p>
          <p className="mt-1 text-xs text-text-secondary text-pretty">{nextTask?.title ?? 'All kickoff lanes are marked complete.'}</p>
        </div>
      </div>

      <form className="mt-4 grid gap-3 rounded-2xl border border-border bg-surface px-4 py-4" onSubmit={submitBrief}>
        <div className="mb-1 flex items-center gap-2">
          <Target className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Execution brief</p>
        </div>

        <div className="grid gap-3 md:grid-cols-2">
          <div className="grid gap-2">
            <label className="text-xs uppercase text-text-secondary">Owner</label>
            <Input {...form.register('owner')} />
            {form.formState.errors.owner ? <p className="text-xs text-rose-300">{form.formState.errors.owner.message}</p> : null}
          </div>

          <div className="grid gap-2">
            <label className="text-xs uppercase text-text-secondary">Validation focus</label>
            <Select value={form.watch('validationMode')} onValueChange={(value) => form.setValue('validationMode', value as DicomExecutionBriefForm['validationMode'], { shouldValidate: true })}>
              <SelectTrigger className="w-full bg-card">
                <SelectValue placeholder="Select validation focus" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="clinical">Clinical review</SelectItem>
                <SelectItem value="dataset">Dataset readiness</SelectItem>
                <SelectItem value="ui">Viewer / UX validation</SelectItem>
                <SelectItem value="automation">Automation / CI gate</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="grid gap-2">
          <label className="text-xs uppercase text-text-secondary">Evidence goal</label>
          <textarea
            className="min-h-20 rounded-md border border-input bg-card px-3 py-2 text-sm text-text-primary"
            {...form.register('evidenceGoal')}
          />
          {form.formState.errors.evidenceGoal ? <p className="text-xs text-rose-300">{form.formState.errors.evidenceGoal.message}</p> : null}
        </div>

        <div className="flex flex-wrap gap-2">
          <Button type="submit" variant="secondary">
            <ClipboardList className="mr-2 size-4" />
            Save execution brief
          </Button>
          <Button type="button" variant="outline" onClick={() => resetPhase()}>
            <RotateCcw className="mr-2 size-4" />
            Reset phase board
          </Button>
        </div>

        <p className="text-xs text-text-secondary text-pretty">
          Focus activo: <span className="font-medium text-text-primary">{VALIDATION_LABELS[brief.validationMode]}</span> · Responsable: <span className="font-medium text-text-primary">{brief.owner}</span>
        </p>
      </form>

      <div className="mt-4 rounded-2xl border border-border bg-surface px-4 py-4">
        <div className="mb-3 flex items-center gap-2">
          <CheckCircle2 className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Actionable kickoff checklist</p>
        </div>

        <div className="space-y-2">
          {DICOM_S01_P01_ACTIONABLE_PHASE.kickoffTasks.map((task) => {
            const checked = Boolean(taskStates[task.id]);
            return (
              <div key={task.id} className="rounded-xl border border-border bg-card px-3 py-3">
                <div className="flex items-start gap-3">
                  <Switch checked={checked} onCheckedChange={(next) => toggleTask(task.id, next)} aria-label={`Toggle ${task.id}`} />
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                      <div>
                        <p className="text-sm font-semibold text-text-display">{task.stream} · {task.title}</p>
                        <p className="mt-1 text-xs text-text-secondary text-pretty">{task.outcome}</p>
                      </div>
                      <Badge variant="outline">{task.sourceTaskRange}</Badge>
                    </div>
                    <p className="mt-2 text-[11px] uppercase text-text-secondary">Lane: {task.lane}</p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
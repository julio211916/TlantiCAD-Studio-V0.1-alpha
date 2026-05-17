import React, { useMemo } from 'react';
import { ArrowRight, Database, FileWarning, FlaskConical, Hospital, ScanLine, UserRound, Wrench } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { buildTlantiDbClinicalOrderGraph, type TlantiDbClinicalOrderNode } from '@/lib/tlantidb-clinical-order';
import type { TlantiCase, TlantiCaseAssetRole } from '@/stores/tlantidb-case-store';

interface ClinicalOrderFlowPanelProps {
  activeCase: TlantiCase;
  onImportMissingAssets: (roles: TlantiCaseAssetRole[]) => void;
  onOpenWorkflow: () => void;
}

const statusClasses: Record<TlantiDbClinicalOrderNode['status'], string> = {
  ready: 'border-success/40 bg-success/10 text-success-foreground',
  partial: 'border-warning/50 bg-warning/10 text-warning',
  blocked: 'border-danger/50 bg-danger/10 text-danger',
};

function FlowNode({
  node,
  icon: Icon,
}: {
  node: TlantiDbClinicalOrderNode;
  icon: React.ComponentType<{ className?: string }>;
}) {
  return (
    <div className="min-w-0 rounded-md border border-border bg-surface px-3 py-3">
      <div className="flex items-start gap-3">
        <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border-visible bg-surface-raised text-text-primary">
          <Icon className="size-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-2">
            <p className="truncate text-sm font-semibold text-text-display">{node.label}</p>
            <span className={`shrink-0 rounded-full border px-2 py-0.5 text-[10px] uppercase ${statusClasses[node.status]}`}>
              {node.status}
            </span>
          </div>
          <p className="mt-1 truncate text-xs text-text-secondary">{node.detail}</p>
        </div>
      </div>
    </div>
  );
}

export function ClinicalOrderFlowPanel({
  activeCase,
  onImportMissingAssets,
  onOpenWorkflow,
}: ClinicalOrderFlowPanelProps) {
  const graph = useMemo(() => buildTlantiDbClinicalOrderGraph(activeCase), [activeCase]);
  const topToothWorks = graph.toothWorks.slice(0, 5);

  return (
    <section className="border-t border-border bg-black px-8 py-4">
      <div className="mb-3 flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
        <div className="min-w-0">
          <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-text-secondary">Clinical order flow</p>
          <h3 className="truncate text-base font-semibold text-text-display">
            {graph.workloadLabel} · {graph.toothWorks.length} tooth prescription{graph.toothWorks.length === 1 ? '' : 's'}
          </h3>
        </div>
        <div className="flex flex-wrap gap-2">
          {graph.missingAssetRoles.length ? (
            <button
              type="button"
              onClick={() => onImportMissingAssets(graph.missingAssetRoles)}
              className="inline-flex h-9 items-center gap-2 rounded-md border border-warning/50 bg-warning/10 px-3 text-xs font-medium text-warning transition-colors hover:bg-warning/15"
            >
              <FileWarning className="size-4" />
              Import missing records
            </button>
          ) : (
            <Badge className="border border-success/40 bg-success/10 text-success-foreground">Records ready</Badge>
          )}
          <button
            type="button"
            onClick={onOpenWorkflow}
            className="inline-flex h-9 items-center gap-2 rounded-md border border-border bg-surface px-3 text-xs font-medium text-text-primary transition-colors hover:bg-surface-raised"
          >
            Tooth workflow
            <ArrowRight className="size-3.5" />
          </button>
        </div>
      </div>

      <div className="grid gap-2 xl:grid-cols-[1fr_auto_1fr_auto_1fr_auto_1fr]">
        <FlowNode node={graph.practice} icon={Hospital} />
        <div className="hidden items-center justify-center text-text-secondary xl:flex"><ArrowRight className="size-4" /></div>
        <FlowNode node={graph.patient} icon={UserRound} />
        <div className="hidden items-center justify-center text-text-secondary xl:flex"><ArrowRight className="size-4" /></div>
        <FlowNode node={graph.caseOrder} icon={Database} />
        <div className="hidden items-center justify-center text-text-secondary xl:flex"><ArrowRight className="size-4" /></div>
        <FlowNode node={graph.technician} icon={Wrench} />
      </div>

      <div className="mt-3 grid gap-3 lg:grid-cols-[minmax(0,1fr)_18rem]">
        <div className="rounded-md border border-border bg-surface px-3 py-3">
          <div className="mb-2 flex items-center justify-between gap-3">
            <p className="text-sm font-semibold text-text-display">Tooth work rows</p>
            <Badge className="border border-border bg-card text-text-primary">case_tooth_work</Badge>
          </div>
          {topToothWorks.length ? (
            <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-5">
              {topToothWorks.map((work) => (
                <button
                  key={work.id}
                  type="button"
                  onClick={onOpenWorkflow}
                  className="min-w-0 rounded-md border border-border bg-card px-3 py-2 text-left transition-colors hover:bg-surface-raised"
                >
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-sm font-semibold text-text-display">{work.toothCode}</p>
                    <ScanLine className="size-3.5 text-text-secondary" />
                  </div>
                  <p className="mt-1 truncate text-xs text-text-secondary">{work.workTypeLabel}</p>
                  <p className="mt-1 truncate text-[11px] text-text-disabled">{work.materialType} · {work.shade}</p>
                </button>
              ))}
            </div>
          ) : (
            <button
              type="button"
              onClick={onOpenWorkflow}
              className="w-full rounded-md border border-dashed border-border px-4 py-5 text-left text-sm text-text-secondary transition-colors hover:border-border-visible hover:text-text-primary"
            >
              Select teeth in the odontogram to create durable tooth work rows.
            </button>
          )}
        </div>

        <div className="rounded-md border border-border bg-surface px-3 py-3">
          <div className="mb-2 flex items-center gap-2">
            <FlaskConical className="size-4 text-text-secondary" />
            <p className="text-sm font-semibold text-text-display">SQLite tables</p>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {graph.sqlTables.map((table) => (
              <span key={table} className="rounded border border-border bg-card px-2 py-1 text-[10px] text-text-secondary">
                {table}
              </span>
            ))}
          </div>
          <p className="mt-3 text-xs leading-5 text-text-secondary">
            Same clinical order idea as DentalDB, mapped to TlantiCAD names and UUID/text keys.
          </p>
        </div>
      </div>
    </section>
  );
}

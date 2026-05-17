import React from 'react';
import { BookOpenCheck, Boxes, FileText, ScanSearch } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { DICOM_CURRENT_CAPABILITIES, DICOM_DENTAL_SPRINTS, DICOM_ROADMAP_DOC_ROOT } from '@/lib/dicom-dental-roadmap';

function tone(status: 'available' | 'partial' | 'planned') {
  switch (status) {
    case 'available':
      return 'text-emerald-300';
    case 'partial':
      return 'text-amber-300';
    default:
      return 'text-sky-300';
  }
}

export function DicomDentalRoadmapPanel() {
  const totalPhases = DICOM_DENTAL_SPRINTS.reduce((sum, sprint) => sum + sprint.phases, 0);
  const totalTasks = DICOM_DENTAL_SPRINTS.reduce((sum, sprint) => sum + sprint.phases * sprint.tasksPerPhase, 0);

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <ScanSearch className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary text-balance">Dental DICOM roadmap</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Plan maestro para incorporar bien el viewer DICOM dental: MPR, 3D, segmentación, RTSTRUCT, upper jaw motion y exportación clínica/manufactura.
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">10 sprints</Badge>
          <Badge variant="outline">{totalPhases} fases</Badge>
          <Badge variant="outline">{totalTasks.toLocaleString('en-US')} tasks</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-3">
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Documentation root</p>
          <p className="mt-1 font-mono text-xs text-text-primary">{DICOM_ROADMAP_DOC_ROOT}</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Current baseline</p>
          <p className="mt-1 text-sm font-semibold text-text-display">OHIF / Cornerstone + Rust + PyDICOM</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Target focus</p>
          <p className="mt-1 text-sm font-semibold text-text-display">Dental CBCT, MPR, 3D, segmentation, upper jaw motion</p>
        </div>
      </div>

      <div className="mt-4 rounded-2xl border border-border bg-surface px-4 py-4">
        <div className="mb-3 flex items-center gap-2">
          <Boxes className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Current capability snapshot</p>
        </div>
        <div className="grid gap-2 xl:grid-cols-2">
          {DICOM_CURRENT_CAPABILITIES.map((capability) => (
            <div key={capability.id} className="rounded-xl border border-border bg-card px-3 py-3">
              <div className="flex items-center justify-between gap-3">
                <p className="text-sm font-semibold text-text-display">{capability.label}</p>
                <span className={`text-[11px] uppercase ${tone(capability.status)}`}>{capability.status}</span>
              </div>
              <p className="mt-2 break-all text-xs text-text-secondary">{capability.evidence}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="mt-4 rounded-2xl border border-border bg-surface px-4 py-4">
        <div className="mb-3 flex items-center gap-2">
          <BookOpenCheck className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Sprint map</p>
        </div>
        <div className="space-y-2">
          {DICOM_DENTAL_SPRINTS.map((sprint) => (
            <div key={sprint.id} className="rounded-xl border border-border bg-card px-3 py-3">
              <div className="flex items-center justify-between gap-3">
                <p className="text-sm font-semibold text-text-display">{sprint.id} · {sprint.title}</p>
                <Badge variant="outline">{sprint.phases} fases · {sprint.tasksPerPhase} tasks/fase</Badge>
              </div>
              <p className="mt-2 text-xs text-text-secondary text-pretty">{sprint.focus}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="mt-4 rounded-2xl border border-border bg-surface px-4 py-4">
        <div className="mb-3 flex items-center gap-2">
          <FileText className="size-4 text-text-secondary" />
          <p className="text-xs uppercase text-text-secondary">Execution note</p>
        </div>
        <p className="text-sm text-text-primary text-pretty">
          Este panel expone el plan documental y el baseline actual. La implementación completa del roadmap requiere varias iteraciones; en esta entrega se documenta el programa completo y se integra su vista dentro de la app para seguimiento y validación continua.
        </p>
      </div>
    </div>
  );
}
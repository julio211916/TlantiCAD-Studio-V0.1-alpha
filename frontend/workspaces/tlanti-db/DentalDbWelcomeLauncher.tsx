import React from 'react';
import { ArrowRight } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { formatTlantiCaseStatus } from '@/lib/tlantidb-case-state-machine';
import type { Language } from '@/types';
import type { TlantiCase } from '@/stores/tlantidb-case-store';

export interface DentalDbLabQueueStats {
  queued: number;
  scanBlocked: number;
  fabReady: number;
}

interface DentalDbWelcomeLauncherProps {
  currentTime: Date;
  language: Language;
  timeZone: string;
  recentCases: TlantiCase[];
  labQueueStats: DentalDbLabQueueStats;
  onCreateCase: () => void;
  onOpenCase: () => void;
  onActivateCase: (caseId: string) => void;
}

export const DentalDbWelcomeLauncher = React.memo(function DentalDbWelcomeLauncher({
  currentTime,
  language,
  timeZone,
  recentCases,
  labQueueStats,
  onCreateCase,
  onOpenCase,
  onActivateCase,
}: DentalDbWelcomeLauncherProps) {
  const copy = language === 'es'
    ? {
        title: 'Workspace TlantiCAD',
        createCase: 'Crear nuevo caso',
        createCaseDescription: 'Datos del caso, indicación y checklist clínico.',
        openCase: 'Abrir caso',
        openCaseDescription: 'Continúa un caso local de TlantiCAD o una carpeta importada.',
        recentCases: 'Casos recientes',
        noRecentCases: 'Aún no hay casos recientes. Crea el primer caso para iniciar la cola del laboratorio.',
        noPatient: 'Sin paciente',
        labStatus: 'Estado del laboratorio hoy',
        casesInQueue: 'Casos en cola',
        missingScan: 'Falta escaneo primario',
        readyForExport: 'Listos para fabricación/exportación',
      }
    : {
        title: 'TlantiCAD Workspace',
        createCase: 'Create new case',
        createCaseDescription: 'Case data, indication and clinical checklist.',
        openCase: 'Open case',
        openCaseDescription: 'Resume a local TlantiCAD case or imported case folder.',
        recentCases: 'Recent cases',
        noRecentCases: 'No recent cases yet. Create the first case to start the lab queue.',
        noPatient: 'No patient',
        labStatus: 'Lab status today',
        casesInQueue: 'Cases in queue',
        missingScan: 'Missing primary scan',
        readyForExport: 'Ready for fabrication/export',
      };

  return (
    <div className="flex min-h-0 flex-1 items-center justify-center overflow-y-auto bg-black px-4 py-8">
      <section className="grid w-full max-w-5xl gap-8">
        <div className="text-center">
          <p className="font-mono text-xs uppercase tracking-[0.32em] text-text-secondary">TlantiCAD Studio</p>
          <h1 className="mt-3 font-display text-3xl text-text-display sm:text-5xl">{copy.title}</h1>
          <p className="mt-3 text-sm text-text-secondary">
            {currentTime.toLocaleDateString(language, { weekday: 'long', month: 'long', day: 'numeric' })} · {currentTime.toLocaleTimeString(language, { hour: '2-digit', minute: '2-digit', timeZone })}
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <button
            type="button"
            onClick={onCreateCase}
            className="min-h-24 rounded-md border border-text-display bg-text-display px-5 py-5 text-left text-black transition-colors hover:bg-white"
          >
            <span className="block font-mono text-xs uppercase tracking-[0.24em]">{copy.createCase}</span>
            <span className="mt-2 block text-sm font-medium">{copy.createCaseDescription}</span>
          </button>
          <button
            type="button"
            onClick={onOpenCase}
            className="min-h-24 rounded-md border border-border bg-surface px-5 py-5 text-left text-text-primary transition-colors hover:bg-surface-raised"
          >
            <span className="block font-mono text-xs uppercase tracking-[0.24em]">{copy.openCase}</span>
            <span className="mt-2 block text-sm text-text-secondary">{copy.openCaseDescription}</span>
          </button>
        </div>

        <div className="grid gap-4 lg:grid-cols-[1.4fr_0.8fr]">
          <div className="rounded-md border border-border bg-surface p-4">
            <div className="flex items-center justify-between gap-3">
              <p className="font-mono text-xs uppercase tracking-[0.24em] text-text-secondary">{copy.recentCases}</p>
              <Badge className="border border-border bg-card text-text-primary">{recentCases.length}</Badge>
            </div>
            <div className="mt-4 grid gap-2">
              {recentCases.length ? recentCases.map((caseItem) => (
                <button
                  key={caseItem.id}
                  type="button"
                  onClick={() => onActivateCase(caseItem.id)}
                  className="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-3 text-left transition-colors hover:bg-surface-raised"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm font-semibold text-text-display">{caseItem.caseNumber} · {caseItem.name}</p>
                    <p className="truncate text-xs text-text-secondary">{caseItem.patientName || caseItem.clientName || copy.noPatient} · {formatTlantiCaseStatus(caseItem.status)}</p>
                  </div>
                  <ArrowRight className="size-4 shrink-0 text-text-secondary" />
                </button>
              )) : (
                <div className="rounded-md border border-dashed border-border px-4 py-6 text-center text-sm text-text-secondary">
                  {copy.noRecentCases}
                </div>
              )}
            </div>
          </div>

          <div className="rounded-md border border-border bg-surface p-4">
            <p className="font-mono text-xs uppercase tracking-[0.24em] text-text-secondary">{copy.labStatus}</p>
            <div className="mt-4 grid gap-2">
              <div className="rounded-md border border-border bg-card px-3 py-3">
                <p className="text-xs text-text-secondary">{copy.casesInQueue}</p>
                <p className="text-2xl font-semibold text-text-display">{labQueueStats.queued}</p>
              </div>
              <div className="rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-3">
                <p className="text-xs text-amber-100/80">{copy.missingScan}</p>
                <p className="text-2xl font-semibold text-amber-100">{labQueueStats.scanBlocked}</p>
              </div>
              <div className="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-3">
                <p className="text-xs text-emerald-100/80">{copy.readyForExport}</p>
                <p className="text-2xl font-semibold text-emerald-100">{labQueueStats.fabReady}</p>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
});

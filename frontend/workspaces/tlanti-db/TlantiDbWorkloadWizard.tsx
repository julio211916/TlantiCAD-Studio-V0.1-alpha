import React, { useMemo, useState } from 'react';
import { CheckCircle2, ChevronRight, CircleAlert, FolderOpen, Layers3, ScanLine, Search, ShieldCheck, Sparkles } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { CLINICAL_ASSET_ROLE_LABELS } from '@/lib/tlanticad-asset-classification';
import { getWorkloadPreset, normalizeWorkloadModuleTarget, TLANTIDB_WORKLOAD_PRESETS, type TlantiWorkloadId } from '@/lib/tlantidb-workloads';
import type { TlantiCaseAssetRole } from '@/lib/tlantidb-case-store';

const TOOTH_ROWS = [
  ['18', '17', '16', '15', '14', '13', '12', '11', '21', '22', '23', '24', '25', '26', '27', '28'],
  ['48', '47', '46', '45', '44', '43', '42', '41', '31', '32', '33', '34', '35', '36', '37', '38'],
];

const STEP_LABELS = ['Case', 'Workload', 'Teeth', 'Assets', 'Launch'];

export interface TlantiDbWorkloadWizardSubmit {
  caseName: string;
  clientName: string;
  workloadId: TlantiWorkloadId;
  toothNumbers: string[];
  activeJaw: 'upper' | 'lower';
  materialShade: string;
  occlusionScanType: string;
  openAfterCreate: boolean;
}

interface TlantiDbWorkloadWizardProps {
  open: boolean;
  defaultCaseName: string;
  defaultClientName: string;
  onOpenChange: (open: boolean) => void;
  onCreateWorkload: (payload: TlantiDbWorkloadWizardSubmit) => void;
}

function RoleChip({ role, required }: { role: TlantiCaseAssetRole; required: boolean }) {
  return (
    <span className={cn(
      'inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-[11px] font-medium',
      required ? 'border-warning/50 bg-warning/10 text-warning' : 'border-glass-border bg-control-bg text-text-secondary',
    )}>
      {required ? <CircleAlert className="size-3" /> : <CheckCircle2 className="size-3" />}
      {CLINICAL_ASSET_ROLE_LABELS[role] ?? role}
    </span>
  );
}

export function TlantiDbWorkloadWizard({ open, defaultCaseName, defaultClientName, onOpenChange, onCreateWorkload }: TlantiDbWorkloadWizardProps) {
  const [caseName, setCaseName] = useState(defaultCaseName);
  const [clientName, setClientName] = useState(defaultClientName);
  const [workloadId, setWorkloadId] = useState<TlantiWorkloadId>('crown-bridge');
  const [selectedTeeth, setSelectedTeeth] = useState<string[]>(['11']);
  const [openAfterCreate, setOpenAfterCreate] = useState(true);

  const preset = useMemo(() => getWorkloadPreset(workloadId), [workloadId]);
  const moduleTarget = useMemo(() => normalizeWorkloadModuleTarget(preset.moduleTarget), [preset.moduleTarget]);
  const activeJaw = useMemo<'upper' | 'lower'>(() => selectedTeeth.some((tooth) => tooth.startsWith('3') || tooth.startsWith('4')) ? 'lower' : 'upper', [selectedTeeth]);

  if (!open) {
    return null;
  }

  const toggleTooth = (tooth: string) => {
    setSelectedTeeth((current) => {
      if (current.includes(tooth)) {
        const next = current.filter((item) => item !== tooth);
        return next.length ? next : current;
      }
      return [...current, tooth].sort((left, right) => Number(left) - Number(right));
    });
  };

  const submit = () => {
    onCreateWorkload({
      caseName,
      clientName,
      workloadId,
      toothNumbers: selectedTeeth,
      activeJaw,
      materialShade: 'A1',
      occlusionScanType: preset.id === 'orthodontics' || preset.id === 'splint-guide' ? 'two_models' : 'single_model',
      openAfterCreate,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-window-bg/72 px-4 py-6">
      <div data-testid="tlantidb-workload-wizard" className="tl-glass flex max-h-[92dvh] w-[min(96vw,72rem)] flex-col overflow-hidden rounded-lg">
        <div className="flex items-start justify-between gap-4 border-b border-glass-border px-5 py-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-text-secondary">TlantiDB workload setup</p>
            <h3 className="text-balance text-2xl font-semibold text-text-display">New clinical workload</h3>
            <p className="mt-1 text-sm text-text-secondary">DentalDB-style setup: patient, indication, teeth, required records and module launch.</p>
          </div>
          <button aria-label="Close workload wizard" onClick={() => onOpenChange(false)} className="tl-control rounded-md px-3 py-2 text-sm text-text-secondary transition-colors hover:text-text-primary">
            Close
          </button>
        </div>

        <div className="grid flex-1 overflow-hidden lg:grid-cols-[17rem_1fr_18rem]">
          <aside className="border-b border-glass-border bg-panel-bg px-4 py-4 lg:border-b-0 lg:border-r">
            <div className="grid gap-2">
              {STEP_LABELS.map((label, index) => (
                <div key={label} className="tl-control flex items-center gap-3 rounded-md px-3 py-2">
                  <span className="tl-control-active flex size-7 items-center justify-center rounded-md text-xs font-semibold">{index + 1}</span>
                  <span className="text-sm text-text-primary">{label}</span>
                </div>
              ))}
            </div>

            <div className="tl-panel mt-4 rounded-md px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Selected workload</p>
              <p className="mt-1 text-sm font-semibold text-text-display">{preset.label}</p>
              <p className="mt-1 text-xs text-text-secondary">{selectedTeeth.length} teeth · {activeJaw} jaw · {moduleTarget}</p>
            </div>
          </aside>

          <main className="overflow-y-auto px-5 py-5">
            <section className="grid gap-4 md:grid-cols-2">
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Case name</span>
                <input value={caseName} onChange={(event) => setCaseName(event.target.value)} className="tl-control rounded-md px-3 py-2 text-sm outline-none focus:border-focus-ring" />
              </label>
              <label className="grid gap-2">
                <span className="text-[11px] uppercase text-text-secondary">Patient / client</span>
                <input value={clientName} onChange={(event) => setClientName(event.target.value)} className="tl-control rounded-md px-3 py-2 text-sm outline-none focus:border-focus-ring" />
              </label>
            </section>

            <section className="mt-5">
              <div className="mb-3 flex items-center justify-between gap-3">
                <div>
                  <p className="text-[11px] uppercase text-text-secondary">Indication</p>
                  <p className="text-sm text-text-primary">Choose the work type before loading heavy modules.</p>
                </div>
                <Badge className="border border-border bg-card text-text-primary">{preset.stage}</Badge>
              </div>

              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {TLANTIDB_WORKLOAD_PRESETS.map((item) => {
                  const active = item.id === workloadId;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => setWorkloadId(item.id)}
                      className={cn(
                        'rounded-md border px-4 py-4 text-left transition-colors',
                        active ? 'tl-control-active' : 'tl-control',
                      )}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <span className="flex size-9 items-center justify-center rounded-md bg-control-bg text-text-primary">
                          {item.id === 'dicom-cbct' ? <ScanLine className="size-4" /> : item.id === 'smile-design' ? <Sparkles className="size-4" /> : <Layers3 className="size-4" />}
                        </span>
                        <ChevronRight className={cn('size-4 text-text-secondary', active && 'text-text-primary')} />
                      </div>
                      <p className="mt-3 text-sm font-semibold text-text-display">{item.label}</p>
                      <p className="mt-1 text-xs leading-5 text-text-secondary">{item.description}</p>
                    </button>
                  );
                })}
              </div>
            </section>

            <section className="mt-5">
              <div className="mb-3 flex items-center justify-between gap-3">
                <div>
                  <p className="text-[11px] uppercase text-text-secondary">Teeth / arch</p>
                  <p className="text-sm text-text-primary">Select FDI teeth for this workload. These become the initial tooth map.</p>
                </div>
                <div className="flex gap-2">
                  <button type="button" onClick={() => setSelectedTeeth(TOOTH_ROWS[0])} className="rounded-lg border border-border px-2.5 py-1 text-xs text-text-secondary hover:text-text-primary">Upper</button>
                  <button type="button" onClick={() => setSelectedTeeth(TOOTH_ROWS[1])} className="rounded-lg border border-border px-2.5 py-1 text-xs text-text-secondary hover:text-text-primary">Lower</button>
                </div>
              </div>

              <div className="tl-panel grid gap-2 rounded-md px-3 py-3">
                {TOOTH_ROWS.map((row) => (
                  <div key={row.join('-')} className="grid gap-2" style={{ gridTemplateColumns: 'repeat(16, minmax(0, 1fr))' }}>
                    {row.map((tooth) => {
                      const active = selectedTeeth.includes(tooth);
                      return (
                        <button
                          key={tooth}
                          type="button"
                          onClick={() => toggleTooth(tooth)}
                          className={cn(
                            'h-9 rounded-md border text-xs font-semibold transition-colors',
                            active ? 'tl-control-active' : 'tl-control text-text-secondary hover:text-text-primary',
                          )}
                        >
                          {tooth}
                        </button>
                      );
                    })}
                  </div>
                ))}
              </div>
            </section>
          </main>

          <aside className="border-t border-glass-border bg-panel-bg px-4 py-4 lg:border-l lg:border-t-0">
            <div className="tl-panel rounded-md px-4 py-4">
              <div className="flex items-center gap-2">
                <ShieldCheck className="size-4 text-text-secondary" />
                <p className="text-sm font-semibold text-text-display">Required records</p>
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                {preset.requiredAssetRoles.map((role) => <RoleChip key={role} role={role} required />)}
              </div>
              <p className="mt-3 text-xs leading-5 text-text-secondary">Clinical export remains blocked until required records are attached. The module may open in preparation mode.</p>
            </div>

            <div className="tl-panel mt-4 rounded-md px-4 py-4">
              <p className="text-sm font-semibold text-text-display">Optional records</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {preset.optionalAssetRoles.map((role) => <RoleChip key={role} role={role} required={false} />)}
              </div>
            </div>

            <label className="tl-control mt-4 flex items-center justify-between gap-3 rounded-md px-4 py-3">
              <span>
                <span className="block text-sm text-text-primary">Open module after create</span>
                <span className="block text-xs text-text-secondary">Launch {moduleTarget} immediately.</span>
              </span>
              <input type="checkbox" checked={openAfterCreate} onChange={(event) => setOpenAfterCreate(event.target.checked)} />
            </label>
          </aside>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-glass-border px-5 py-4">
          <div className="flex items-center gap-2 text-xs text-text-secondary">
            <Search className="size-4" />
            Metadata only. No Three, VTK, DICOM or AI runtime is loaded here.
          </div>
          <div className="flex gap-2">
            <button type="button" onClick={() => onOpenChange(false)} className="tl-control rounded-md px-4 py-2 text-xs uppercase text-text-secondary transition-colors hover:text-text-primary">Cancel</button>
            <button type="button" onClick={submit} className="tl-control-active inline-flex items-center gap-2 rounded-md px-4 py-2 text-xs font-semibold uppercase">
              <FolderOpen className="size-4" />
              Create workload
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

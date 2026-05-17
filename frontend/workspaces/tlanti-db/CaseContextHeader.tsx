import React from 'react';
import {
  Briefcase,
  Building2,
  Calendar,
  ChevronDown,
  Copy,
  FileDown,
  Hash,
  Package,
  Printer,
  Camera,
  User,
  X,
} from 'lucide-react';

import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import type { TlantiCase, TlantiCaseAsset } from '@/stores/tlantidb-case-store';

interface CaseContextHeaderProps {
  activeCase: TlantiCase;
  assets: TlantiCaseAsset[];
  patientCases?: Pick<TlantiCase, 'id' | 'caseNumber' | 'name' | 'status'>[];
  onSelectCase?: (caseId: string) => void;
  onSnapshot?: () => void;
  onDuplicate?: () => void;
  onPrint?: () => void;
  onExport?: () => void;
  onShare?: () => void;
  onUpdateCase?: (patch: Partial<TlantiCase>) => void;
}

/**
 * V143 — Top context bar replacing the lateral Patient panel.
 * 4 collapsible triggers (Caso, Paciente, Clínica, Assets) styled exocad-like.
 * Each trigger expands a Popover with the relevant case fields and quick actions.
 */
export function CaseContextHeader({
  activeCase,
  assets,
  patientCases = [],
  onSelectCase,
  onSnapshot,
  onDuplicate,
  onPrint,
  onExport,
  onShare,
  onUpdateCase,
}: CaseContextHeaderProps) {
  const triggerClass =
    'group flex h-full min-w-[10rem] flex-1 items-start gap-2 border-r border-border/60 bg-transparent px-3 py-2 text-left transition-colors hover:bg-surface-raised data-[state=open]:bg-surface-raised data-[state=open]:border-b-2 data-[state=open]:border-b-cyan-400';

  return (
    <div className="flex h-10 w-full items-stretch border-b border-border bg-[#111] text-text-primary">
      {/* Caso */}
      <Popover>
        <PopoverTrigger className={triggerClass} aria-label="Datos del caso">
          <Briefcase className="mt-0.5 size-3.5 shrink-0 text-text-secondary" />
          <div className="min-w-0 flex-1">
            <p className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">Caso</p>
            <p className="truncate text-[12px] font-semibold text-text-display">{activeCase.caseNumber}</p>
          </div>
          <ChevronDown className="mt-1 size-3 shrink-0 text-text-secondary group-data-[state=open]:rotate-180" />
        </PopoverTrigger>
        <PopoverContent align="start" sideOffset={0} className="w-[640px] border-border bg-[#141414] p-6 text-text-primary">
          <CaseDropdownContent
            activeCase={activeCase}
            onUpdateCase={onUpdateCase}
            onSnapshot={onSnapshot}
            onDuplicate={onDuplicate}
            onPrint={onPrint}
            onExport={onExport}
          />
        </PopoverContent>
      </Popover>

      {/* Paciente */}
      <Popover>
        <PopoverTrigger className={triggerClass} aria-label="Datos del paciente">
          <User className="mt-0.5 size-3.5 shrink-0 text-text-secondary" />
          <div className="min-w-0 flex-1">
            <p className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">Paciente</p>
            <p className="truncate text-[12px] font-semibold text-text-display">{activeCase.patientName ?? activeCase.clientName ?? 'Sin paciente'}</p>
          </div>
          <ChevronDown className="mt-1 size-3 shrink-0 text-text-secondary group-data-[state=open]:rotate-180" />
        </PopoverTrigger>
        <PopoverContent align="start" sideOffset={0} className="w-[680px] border-border bg-[#141414] p-6 text-text-primary">
          <PatientDropdownContent activeCase={activeCase} patientCases={patientCases} onSelectCase={onSelectCase} onUpdateCase={onUpdateCase} />
        </PopoverContent>
      </Popover>

      {/* Clínica */}
      <Popover>
        <PopoverTrigger className={triggerClass} aria-label="Clínica y laboratorio">
          <Building2 className="mt-0.5 size-3.5 shrink-0 text-text-secondary" />
          <div className="min-w-0 flex-1">
            <p className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">Clínica</p>
            <p className="truncate text-[12px] font-semibold text-text-display">{activeCase.laboratoryName ?? activeCase.clientName ?? 'Sin clínica'}</p>
          </div>
          <ChevronDown className="mt-1 size-3 shrink-0 text-text-secondary group-data-[state=open]:rotate-180" />
        </PopoverTrigger>
        <PopoverContent align="start" sideOffset={0} className="w-[600px] border-border bg-[#141414] p-6 text-text-primary">
          <ClinicDropdownContent activeCase={activeCase} onUpdateCase={onUpdateCase} />
        </PopoverContent>
      </Popover>

      {/* Assets */}
      <Popover>
        <PopoverTrigger className={cn(triggerClass, 'border-r-0')} aria-label="Assets del caso">
          <Package className="mt-0.5 size-3.5 shrink-0 text-text-secondary" />
          <div className="min-w-0 flex-1">
            <p className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">Assets</p>
            <p className="truncate text-[12px] font-semibold text-text-display">{assets.length} archivo{assets.length === 1 ? '' : 's'}</p>
          </div>
          <ChevronDown className="mt-1 size-3 shrink-0 text-text-secondary group-data-[state=open]:rotate-180" />
        </PopoverTrigger>
        <PopoverContent align="end" sideOffset={0} className="w-[560px] border-border bg-[#141414] p-6 text-text-primary">
          <AssetsDropdownContent assets={assets} onShare={onShare} />
        </PopoverContent>
      </Popover>
    </div>
  );
}

function FieldLabel({ children }: { children: React.ReactNode }) {
  return <label className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">{children}</label>;
}

function ReadOnlyField({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1">
      <FieldLabel>{label}</FieldLabel>
      <div className="rounded border border-border bg-card px-3 py-2 text-sm text-text-primary">{value || '—'}</div>
    </div>
  );
}

function EditableField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string | undefined | null;
  onChange?: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <div className="flex flex-col gap-1">
      <FieldLabel>{label}</FieldLabel>
      <input
        type="text"
        defaultValue={value ?? ''}
        onBlur={(event) => onChange?.(event.target.value)}
        placeholder={placeholder}
        className="rounded border border-border bg-card px-3 py-2 text-sm text-text-primary outline-none focus:border-cyan-400/60"
      />
    </div>
  );
}

function CaseDropdownContent({
  activeCase,
  onUpdateCase,
  onSnapshot,
  onDuplicate,
  onPrint,
  onExport,
}: {
  activeCase: TlantiCase;
  onUpdateCase?: (patch: Partial<TlantiCase>) => void;
  onSnapshot?: () => void;
  onDuplicate?: () => void;
  onPrint?: () => void;
  onExport?: () => void;
}) {
  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-text-display">Datos del caso</h3>
        <Badge variant="outline" className="border-border bg-card text-text-secondary">{activeCase.status ?? 'Activo'}</Badge>
      </header>

      <section className="grid grid-cols-2 gap-4">
        <EditableField label="Nombre del caso" value={activeCase.name} onChange={(value) => onUpdateCase?.({ name: value })} />
        <ReadOnlyField label="Número" value={activeCase.caseNumber} />
      </section>

      <section className="grid grid-cols-2 gap-4">
        <EditableField label="Orden / Order #" value={activeCase.orderNumber} onChange={(value) => onUpdateCase?.({ orderNumber: value })} placeholder="ORD-2026-####" />
        <ReadOnlyField label="Creado" value={activeCase.createdAt ? new Date(activeCase.createdAt).toLocaleString() : '—'} />
      </section>

      <section className="space-y-2">
        <FieldLabel>Notas clínicas</FieldLabel>
        <textarea
          defaultValue={activeCase.notes ?? ''}
          onBlur={(event) => onUpdateCase?.({ notes: event.target.value })}
          rows={3}
          placeholder="Alergias, prescripción, expectativas de sombra…"
          className="w-full resize-y rounded border border-border bg-card px-3 py-2 text-sm text-text-primary outline-none focus:border-cyan-400/60"
        />
      </section>

      <section>
        <FieldLabel>Acciones rápidas</FieldLabel>
        <div className="mt-2 flex flex-wrap gap-2">
          {onSnapshot && (
            <Button type="button" variant="outline" size="sm" onClick={onSnapshot}>
              <Camera className="mr-1.5 size-3.5" /> Snapshot
            </Button>
          )}
          {onDuplicate && (
            <Button type="button" variant="outline" size="sm" onClick={onDuplicate}>
              <Copy className="mr-1.5 size-3.5" /> Duplicar
            </Button>
          )}
          {onPrint && (
            <Button type="button" variant="outline" size="sm" onClick={onPrint}>
              <Printer className="mr-1.5 size-3.5" /> Imprimir
            </Button>
          )}
          {onExport && (
            <Button type="button" variant="outline" size="sm" onClick={onExport}>
              <FileDown className="mr-1.5 size-3.5" /> Exportar
            </Button>
          )}
        </div>
      </section>
    </div>
  );
}

function PatientDropdownContent({
  activeCase,
  patientCases,
  onSelectCase,
  onUpdateCase,
}: {
  activeCase: TlantiCase;
  patientCases: Pick<TlantiCase, 'id' | 'caseNumber' | 'name' | 'status'>[];
  onSelectCase?: (caseId: string) => void;
  onUpdateCase?: (patch: Partial<TlantiCase>) => void;
}) {
  return (
    <div className="space-y-5">
      <header>
        <h3 className="text-sm font-semibold text-text-display">Paciente</h3>
        <p className="text-xs text-text-secondary">Datos demográficos y casos relacionados.</p>
      </header>

      <section className="grid grid-cols-2 gap-4">
        <EditableField label="Nombre" value={activeCase.patientName} onChange={(value) => onUpdateCase?.({ patientName: value })} />
        <EditableField
          label="Fecha de nacimiento"
          value={activeCase.patientDateOfBirth}
          onChange={(value) => onUpdateCase?.({ patientDateOfBirth: value })}
          placeholder="dd/mm/aaaa"
        />
      </section>

      <section>
        <FieldLabel>Casos del paciente</FieldLabel>
        <div className="mt-2 max-h-48 overflow-y-auto rounded border border-border">
          {patientCases.length ? (
            patientCases.map((entry) => {
              const isCurrent = entry.id === activeCase.id;
              return (
                <button
                  key={entry.id}
                  type="button"
                  onClick={() => !isCurrent && onSelectCase?.(entry.id)}
                  disabled={isCurrent}
                  className={cn(
                    'flex w-full items-center justify-between gap-3 border-b border-border px-3 py-2 text-left text-sm transition-colors last:border-b-0',
                    isCurrent ? 'cursor-default bg-cyan-500/10 text-cyan-200' : 'text-text-primary hover:bg-surface-raised',
                  )}
                >
                  <span className="truncate"><span className="font-mono text-xs text-text-secondary">{entry.caseNumber}</span> · {entry.name}</span>
                  <Badge variant="outline" className="border-border bg-transparent text-[10px] uppercase tracking-wider">{entry.status ?? '—'}</Badge>
                </button>
              );
            })
          ) : (
            <p className="px-3 py-3 text-xs text-text-secondary">Sin casos previos asociados.</p>
          )}
        </div>
      </section>
    </div>
  );
}

function ClinicDropdownContent({
  activeCase,
  onUpdateCase,
}: {
  activeCase: TlantiCase;
  onUpdateCase?: (patch: Partial<TlantiCase>) => void;
}) {
  return (
    <div className="space-y-5">
      <header>
        <h3 className="text-sm font-semibold text-text-display">Clínica & laboratorio</h3>
        <p className="text-xs text-text-secondary">Cliente, laboratorio y técnico responsable.</p>
      </header>

      <section className="grid grid-cols-2 gap-4">
        <EditableField label="Cliente / clínica" value={activeCase.clientName} onChange={(value) => onUpdateCase?.({ clientName: value })} />
        <EditableField label="ID cliente" value={activeCase.clientId} onChange={(value) => onUpdateCase?.({ clientId: value })} />
      </section>

      <section className="grid grid-cols-2 gap-4">
        <EditableField label="Laboratorio" value={activeCase.laboratoryName} onChange={(value) => onUpdateCase?.({ laboratoryName: value })} />
        <EditableField label="Técnico" value={activeCase.technicianName} onChange={(value) => onUpdateCase?.({ technicianName: value })} />
      </section>

      <section>
        <EditableField label="Técnico ID" value={activeCase.technicianId} onChange={(value) => onUpdateCase?.({ technicianId: value })} />
      </section>
    </div>
  );
}

function AssetsDropdownContent({
  assets,
  onShare,
}: {
  assets: TlantiCaseAsset[];
  onShare?: () => void;
}) {
  const grouped = assets.reduce<Record<string, TlantiCaseAsset[]>>((acc, asset) => {
    const key = asset.category ?? 'other';
    (acc[key] ??= []).push(asset);
    return acc;
  }, {});

  return (
    <div className="space-y-4">
      <header className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold text-text-display">Assets del caso</h3>
          <p className="text-xs text-text-secondary">{assets.length} archivo{assets.length === 1 ? '' : 's'} totales.</p>
        </div>
        {onShare && (
          <Button type="button" variant="outline" size="sm" onClick={onShare}>
            Compartir caso
          </Button>
        )}
      </header>

      {assets.length === 0 ? (
        <div className="rounded border border-dashed border-border px-4 py-6 text-center text-xs text-text-secondary">
          Sin archivos clínicos. Importa scans, DICOM o documentos desde el panel principal.
        </div>
      ) : (
        Object.entries(grouped).map(([category, items]) => (
          <section key={category}>
            <FieldLabel>{category.toUpperCase()} · {items.length}</FieldLabel>
            <ul className="mt-2 space-y-1">
              {items.slice(0, 6).map((asset) => (
                <li key={asset.id} className="flex items-center justify-between rounded border border-border bg-card px-3 py-2 text-xs text-text-primary">
                  <span className="truncate">
                    <span className="font-mono text-[10px] text-text-secondary mr-2">
                      <Hash className="inline size-3 align-text-bottom" />
                    </span>
                    {asset.name ?? asset.role ?? asset.id}
                  </span>
                  <span className="ml-3 flex shrink-0 items-center gap-1 text-text-secondary">
                    <Calendar className="size-3" />
                    {asset.importedAt ? new Date(asset.importedAt).toLocaleDateString() : '—'}
                  </span>
                </li>
              ))}
              {items.length > 6 && (
                <li className="text-[11px] italic text-text-secondary">+{items.length - 6} más…</li>
              )}
            </ul>
          </section>
        ))
      )}
    </div>
  );
}

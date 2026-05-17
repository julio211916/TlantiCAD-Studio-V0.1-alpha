import React from 'react';
import clsx from 'clsx';
import { Badge } from '@/components/ui/badge';

export interface CreateCaseWizardDraft {
  caseName: string;
  clientName: string;
  orderNumber: string;
  /** Combined name kept for downstream compatibility (e.g. interop XML). */
  patientName: string;
  /** RealGUIDE-aligned split (V47). Optional — derives `patientName` when set. */
  patientFirstName?: string;
  patientSurname?: string;
  patientBirthDate: string;
  technicianName: string;
  laboratoryName: string;
  dueAt: string;
  clinicalNotes: string;
}

interface CreateCaseWizardProps {
  draft: CreateCaseWizardDraft;
  step: 1 | 2 | 3;
  onChange: (patch: Partial<CreateCaseWizardDraft>) => void;
  onStepChange: (step: 1 | 2 | 3) => void;
  onCancel: () => void;
  onCreate: () => void;
}

const REQUIRED_ASSETS = [
  ['Lab prescription', 'PDF, image or document from the clinician.'],
  ['Primary scan', 'STL/OBJ/DICOM of the prepared tooth or restorative model.'],
  ['Opposing / bite record', 'Antagonist scan or bite registration for occlusion.'],
  ['Shade reference', 'Clinical photo or shade tab for esthetic validation.'],
] as const;

function CaseInput({
  label,
  value,
  onChange,
  placeholder,
  type = 'text',
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: React.HTMLInputTypeAttribute;
}) {
  return (
    <label className="grid gap-2">
      <span className="text-xs uppercase text-text-secondary">{label}</span>
      <input
        title={label}
        type={type}
        placeholder={placeholder}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="rounded-md border border-border bg-surface-raised px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary"
      />
    </label>
  );
}

export const CreateCaseWizard = React.memo(function CreateCaseWizard({
  draft,
  step,
  onChange,
  onStepChange,
  onCancel,
  onCreate,
}: CreateCaseWizardProps) {
  const goBack = () => onStepChange(step === 3 ? 2 : 1);
  const goNext = () => onStepChange(step === 1 ? 2 : 3);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-case-title"
        className="flex max-h-[92dvh] w-[min(94vw,52rem)] flex-col gap-5 overflow-y-auto rounded-lg border border-border bg-surface p-6 shadow-2xl"
      >
        <div className="flex items-center justify-between border-b border-border pb-4">
          <div>
            <h3 id="create-case-title" className="text-balance font-display text-xl text-text-display">Create case</h3>
            <p className="text-pretty text-sm text-text-secondary">
              Step {step} of 3 · case data, dental indication and required assets.
            </p>
          </div>
          <button aria-label="Close create case modal" onClick={onCancel} className="text-text-secondary transition-colors hover:text-text-primary">x</button>
        </div>

        <div className="grid grid-cols-3 gap-2">
          {[1, 2, 3].map((item) => (
            <div key={item} className={clsx('h-1 rounded-full', step >= item ? 'bg-text-display' : 'bg-border')} />
          ))}
        </div>

        {step === 1 && (
          <div className="grid gap-4 md:grid-cols-2">
            <CaseInput label="Case name *" placeholder="New zirconia crown" value={draft.caseName} onChange={(caseName) => onChange({ caseName })} />
            <CaseInput label="Order number" placeholder="ORD-2026-0042" value={draft.orderNumber} onChange={(orderNumber) => onChange({ orderNumber })} />
            <CaseInput label="Client / clinic *" placeholder="Dr. Carlos Mendez" value={draft.clientName} onChange={(clientName) => onChange({ clientName })} />
            <CaseInput
              label="Patient first name *"
              placeholder="Juan"
              value={draft.patientFirstName ?? draft.patientName.split(' ')[0] ?? ''}
              onChange={(patientFirstName) => {
                const surname = draft.patientSurname ?? draft.patientName.split(' ').slice(1).join(' ');
                onChange({
                  patientFirstName,
                  patientName: `${patientFirstName} ${surname}`.trim(),
                });
              }}
            />
            <CaseInput
              label="Patient surname"
              placeholder="Garcia Perez"
              value={draft.patientSurname ?? draft.patientName.split(' ').slice(1).join(' ')}
              onChange={(patientSurname) => {
                const firstName = draft.patientFirstName ?? draft.patientName.split(' ')[0] ?? '';
                onChange({
                  patientSurname,
                  patientName: `${firstName} ${patientSurname}`.trim(),
                });
              }}
            />
            <CaseInput label="Patient birth date" type="date" value={draft.patientBirthDate} onChange={(patientBirthDate) => onChange({ patientBirthDate })} />
            <CaseInput label="Promised date" type="date" value={draft.dueAt} onChange={(dueAt) => onChange({ dueAt })} />
            <CaseInput label="Technician *" placeholder="Ing. Ramirez Soto" value={draft.technicianName} onChange={(technicianName) => onChange({ technicianName })} />
            <CaseInput label="Laboratory" placeholder="Tlanti Lab" value={draft.laboratoryName} onChange={(laboratoryName) => onChange({ laboratoryName })} />
            <label className="grid gap-2 md:col-span-2">
              <span className="text-xs uppercase text-text-secondary">Clinical notes</span>
              <textarea
                title="Clinical notes"
                placeholder="Allergies, prescription notes, shade expectations"
                value={draft.clinicalNotes}
                onChange={(event) => onChange({ clinicalNotes: event.target.value })}
                className="min-h-20 rounded-md border border-border bg-surface-raised px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary"
              />
            </label>
          </div>
        )}

        {step === 2 && (
          <div className="grid gap-4">
            <div className="rounded-md border border-border bg-surface-raised p-4">
              <p className="text-sm font-semibold text-text-display">Dental indication</p>
              <p className="mt-1 text-sm text-text-secondary">
                After creating the case, the TlantiCAD Workspace opens the odontogram to select teeth, restoration type, material, shade, implant mode and bridge connectors.
              </p>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              <button type="button" className="rounded-md border border-text-display bg-text-display px-3 py-3 text-left text-sm font-medium text-black">
                FDI numbering
              </button>
              <button type="button" className="rounded-md border border-border bg-card px-3 py-3 text-left text-sm text-text-primary">
                Unit case first
              </button>
              <button type="button" className="rounded-md border border-border bg-card px-3 py-3 text-left text-sm text-text-primary">
                Multi-die available
              </button>
            </div>
            <div className="rounded-md border border-amber-500/40 bg-amber-500/10 p-4 text-sm text-amber-100">
              Bridge rule: when 2+ adjacent teeth are selected, TlantiCAD proposes a bridge span and marks connectors visually in the odontogram.
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="grid gap-3">
            <div className="rounded-md border border-border bg-surface-raised p-4">
              <p className="text-sm font-semibold text-text-display">Required asset checklist</p>
              <p className="mt-1 text-sm text-text-secondary">You can create the case now and import files immediately from the visible Clinical Checklist.</p>
            </div>
            {REQUIRED_ASSETS.map(([title, detail]) => (
              <div key={title} className="flex items-start justify-between gap-3 rounded-md border border-border bg-card px-3 py-2.5">
                <div>
                  <p className="text-sm font-medium text-text-primary">{title}</p>
                  <p className="text-xs text-text-secondary">{detail}</p>
                </div>
                <Badge className="border border-amber-500/40 bg-amber-500/10 text-amber-100">Pending</Badge>
              </div>
            ))}
          </div>
        )}

        <div className="flex justify-end gap-2 border-t border-border pt-4">
          <button onClick={onCancel} className="rounded-md border border-border px-3 py-2 text-xs uppercase text-text-secondary transition-colors hover:text-text-primary">Cancel</button>
          {step > 1 && (
            <button onClick={goBack} className="rounded-md border border-border px-3 py-2 text-xs uppercase text-text-secondary transition-colors hover:text-text-primary">Back</button>
          )}
          {step < 3 ? (
            <button onClick={goNext} className="rounded-md bg-text-display px-4 py-2 text-xs font-semibold uppercase text-black">Next</button>
          ) : (
            <button onClick={onCreate} className="rounded-md bg-text-display px-4 py-2 text-xs font-semibold uppercase text-black">Create case</button>
          )}
        </div>
      </div>
    </div>
  );
});

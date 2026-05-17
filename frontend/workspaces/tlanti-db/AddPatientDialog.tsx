/**
 * AddPatientDialog — V263.
 *
 * Standalone modal matching the RealGUIDE Add-Patient flow:
 *   - First name, Surname, DoB, Notes
 *   - OK button confirms; Cancel discards
 *   - When the DICOM dataset is already available, the form can be submitted
 *     empty and the patient data will be read from the DICOM headers.
 *
 * Pure presentational. Parent persists the result.
 *
 * i18n keys under `patient.add.*`.
 */

import React, { useEffect, useState } from 'react';

import { useT } from '../../lib/i18n';

export interface AddPatientForm {
    firstName: string;
    surname: string;
    dateOfBirth: string;
    notes: string;
}

export interface AddPatientDialogProps {
    open: boolean;
    initial?: Partial<AddPatientForm>;
    onCancel: () => void;
    onConfirm: (form: AddPatientForm) => void;
    /** Show a hint when the DICOM headers will fill the data automatically. */
    canReadFromDicom?: boolean;
}

export function AddPatientDialog({
    open,
    initial,
    onCancel,
    onConfirm,
    canReadFromDicom = false,
}: AddPatientDialogProps) {
    const t = useT();
    const [form, setForm] = useState<AddPatientForm>(() => ({
        firstName: initial?.firstName ?? '',
        surname: initial?.surname ?? '',
        dateOfBirth: initial?.dateOfBirth ?? '',
        notes: initial?.notes ?? '',
    }));

    useEffect(() => {
        if (open) {
            setForm({
                firstName: initial?.firstName ?? '',
                surname: initial?.surname ?? '',
                dateOfBirth: initial?.dateOfBirth ?? '',
                notes: initial?.notes ?? '',
            });
        }
    }, [open, initial?.firstName, initial?.surname, initial?.dateOfBirth, initial?.notes]);

    if (!open) return null;

    const updateField = <K extends keyof AddPatientForm>(key: K, value: AddPatientForm[K]) =>
        setForm((prev) => ({ ...prev, [key]: value }));

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label={t('patient.add.title')}
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onCancel();
            }}
        >
            <div className="flex w-full max-w-md flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        {t('patient.add.title')}
                    </h2>
                    {canReadFromDicom ? (
                        <p className="mt-1 text-[11px] text-text-secondary">
                            Si el conjunto DICOM ya está disponible, puedes pulsar OK sin
                            llenar los campos — los datos se leerán automáticamente del DICOM.
                        </p>
                    ) : null}
                </header>

                <form
                    className="flex flex-col gap-3 px-4 py-4"
                    onSubmit={(e) => {
                        e.preventDefault();
                        onConfirm(form);
                    }}
                >
                    <Field
                        label={t('patient.add.first-name')}
                        value={form.firstName}
                        onChange={(v) => updateField('firstName', v)}
                        required
                    />
                    <Field
                        label={t('patient.add.surname')}
                        value={form.surname}
                        onChange={(v) => updateField('surname', v)}
                    />
                    <Field
                        label={t('patient.add.dob')}
                        type="date"
                        value={form.dateOfBirth}
                        onChange={(v) => updateField('dateOfBirth', v)}
                    />
                    <label className="flex flex-col gap-1 text-[11px]">
                        <span className="text-text-secondary uppercase tracking-wider">
                            {t('patient.add.notes')}
                        </span>
                        <textarea
                            value={form.notes}
                            onChange={(e) => updateField('notes', e.currentTarget.value)}
                            className="min-h-20 rounded-md border border-border bg-surface-sunken px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-text-primary"
                            placeholder=""
                        />
                    </label>

                    <footer className="flex items-center gap-2 pt-2">
                        <button
                            type="button"
                            onClick={onCancel}
                            className="rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                        >
                            {t('patient.add.cancel')}
                        </button>
                        <button
                            type="submit"
                            className="ml-auto rounded-md bg-sky-500 px-4 py-1.5 text-xs font-semibold text-white"
                        >
                            {t('patient.add.ok')}
                        </button>
                    </footer>
                </form>
            </div>
        </div>
    );
}

function Field({
    label,
    value,
    onChange,
    type = 'text',
    required,
}: {
    label: string;
    value: string;
    onChange: (next: string) => void;
    type?: React.HTMLInputTypeAttribute;
    required?: boolean;
}) {
    return (
        <label className="flex flex-col gap-1 text-[11px]">
            <span className="text-text-secondary uppercase tracking-wider">
                {label}
                {required ? ' *' : ''}
            </span>
            <input
                type={type}
                value={value}
                required={required}
                onChange={(e) => onChange(e.currentTarget.value)}
                className="rounded-md border border-border bg-surface-sunken px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-text-primary"
            />
        </label>
    );
}

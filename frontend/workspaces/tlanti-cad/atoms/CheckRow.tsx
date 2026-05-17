/**
 * CheckRow — atom checkbox row with optional helper text.
 */

import React from 'react';

export interface CheckRowProps {
    label: string;
    checked: boolean;
    onChange: (next: boolean) => void;
    helpText?: string;
    disabled?: boolean;
}

export function CheckRow({ label, checked, onChange, helpText, disabled }: CheckRowProps) {
    return (
        <label className={['flex flex-col gap-0.5 text-[11px]', disabled ? 'opacity-50' : ''].join(' ')}>
            <span className="flex items-center gap-2">
                <input
                    type="checkbox"
                    className="accent-sky-400"
                    checked={checked}
                    disabled={disabled}
                    onChange={(e) => onChange(e.currentTarget.checked)}
                />
                <span>{label}</span>
            </span>
            {helpText ? <span className="text-[10px] text-slate-400">{helpText}</span> : null}
        </label>
    );
}

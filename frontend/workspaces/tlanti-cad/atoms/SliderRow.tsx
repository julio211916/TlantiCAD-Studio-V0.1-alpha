/**
 * SliderRow — atom (V173 atomic-design scaffold).
 *
 * Reusable slider with label + numeric badge. Used by abutment/articulator/
 * crown-bottoms/freeforming panels — currently each has its own copy. New
 * panels should import this one.
 */

import React from 'react';

export interface SliderRowProps {
    label: string;
    unit: string;
    value: number;
    min: number;
    max: number;
    step: number;
    onChange: (next: number) => void;
    disabled?: boolean;
    /** Highlight the row when the value differs from the stored default. */
    isOverridden?: boolean;
    helpText?: string;
}

export function SliderRow({
    label,
    unit,
    value,
    min,
    max,
    step,
    onChange,
    disabled,
    isOverridden,
    helpText,
}: SliderRowProps) {
    const decimals = step < 1 ? 2 : 0;
    return (
        <label
            className={[
                'flex flex-col gap-1 rounded px-1.5 py-1 text-[11px] transition',
                isOverridden ? 'bg-amber-500/10' : '',
                disabled ? 'opacity-50' : '',
            ].join(' ')}
        >
            <span className="flex items-center justify-between">
                <span>{label}</span>
                <span className="font-mono tabular-nums text-slate-300">
                    {value.toFixed(decimals)} {unit}
                </span>
            </span>
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                disabled={disabled}
                onChange={(e) => onChange(parseFloat(e.currentTarget.value))}
                className="accent-sky-400"
            />
            {helpText ? (
                <span className="text-[10px] text-slate-400">{helpText}</span>
            ) : null}
        </label>
    );
}

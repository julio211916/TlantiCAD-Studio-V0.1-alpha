/**
 * ClippingPlanesPanel — right-docked collapsible panel with one row per
 * anatomical plane. Pure presentational; parent owns the state and wires
 * changes into Cornerstone/three.js via `onChange`.
 */

import React, { useState } from 'react';

import { AppIcon } from '../../app-icons';

import {
    DEFAULT_CLIPPING_PLANES,
    invertPlane,
    offsetPlane,
    resetPlanes,
    togglePlane,
    type ClipAxis,
    type ClippingPlane,
} from '../domain/clipping-plane';

interface ClippingPlanesPanelProps {
    planes?: ClippingPlane[];
    onChange: (planes: ClippingPlane[]) => void;
    onClose?: () => void;
}

const AXIS_LABELS: Record<ClipAxis, string> = {
    axial: 'Axial (Z)',
    sagittal: 'Sagittal (X)',
    coronal: 'Coronal (Y)',
};

export function ClippingPlanesPanel({
    planes: initialPlanes,
    onChange,
    onClose,
}: ClippingPlanesPanelProps) {
    const [planes, setPlanes] = useState<ClippingPlane[]>(
        initialPlanes ?? DEFAULT_CLIPPING_PLANES,
    );
    const [expanded, setExpanded] = useState(true);

    const update = (next: ClippingPlane[]) => {
        setPlanes(next);
        onChange(next);
    };

    return (
        <aside
            role="complementary"
            aria-labelledby="clipping-planes-title"
            className="pointer-events-auto flex w-60 flex-col gap-3 rounded-lg border border-border bg-sky-500/90 p-3 text-slate-50 shadow-xl backdrop-blur"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="viewer.clipping-plane" size={16} aria-hidden />
                <h3 id="clipping-planes-title" className="flex-1 text-xs font-semibold">
                    Clipping Planes
                </h3>
                <button
                    type="button"
                    aria-label={expanded ? 'Collapse' : 'Expand'}
                    onClick={() => setExpanded((v) => !v)}
                    className="rounded p-1 text-slate-100 hover:bg-white/10"
                >
                    <AppIcon
                        name={expanded ? 'common.chevron-down' : 'common.add'}
                        size={14}
                        aria-hidden
                    />
                </button>
                {onClose ? (
                    <button
                        type="button"
                        aria-label="Close panel"
                        onClick={onClose}
                        className="rounded p-1 text-slate-100 hover:bg-white/10"
                    >
                        <AppIcon name="common.close" size={14} aria-hidden />
                    </button>
                ) : null}
            </header>

            {expanded ? (
                <>
                    <ul className="flex flex-col gap-3">
                        {planes.map((plane) => (
                            <li key={plane.id} className="flex flex-col gap-1.5">
                                <div className="flex items-center gap-2 text-[0.6875rem]">
                                    <button
                                        type="button"
                                        role="switch"
                                        aria-checked={plane.enabled}
                                        onClick={() => update(togglePlane(planes, plane.id))}
                                        className={[
                                            'relative h-4 w-7 rounded-full border transition-colors',
                                            plane.enabled
                                                ? 'border-white bg-white/80'
                                                : 'border-white/50 bg-transparent',
                                        ].join(' ')}
                                    >
                                        <span
                                            className={[
                                                'absolute top-[1px] h-[14px] w-[14px] rounded-full bg-sky-600 transition-transform',
                                                plane.enabled ? 'translate-x-[12px]' : 'translate-x-[1px]',
                                            ].join(' ')}
                                        />
                                    </button>
                                    <span className="flex-1 font-medium">
                                        {AXIS_LABELS[plane.axis]}
                                    </span>
                                    <button
                                        type="button"
                                        aria-label="Invert plane direction"
                                        onClick={() => update(invertPlane(planes, plane.id))}
                                        className={[
                                            'rounded border px-1.5 text-[0.625rem] uppercase tracking-wider',
                                            plane.inverted
                                                ? 'border-white bg-white text-sky-700'
                                                : 'border-white/50 text-white/80',
                                        ].join(' ')}
                                    >
                                        Inv
                                    </button>
                                </div>
                                <input
                                    type="range"
                                    min={-100}
                                    max={100}
                                    value={Math.round(plane.offset * 100)}
                                    disabled={!plane.enabled}
                                    onChange={(event) =>
                                        update(
                                            offsetPlane(
                                                planes,
                                                plane.id,
                                                Number.parseInt(event.currentTarget.value, 10) / 100,
                                            ),
                                        )
                                    }
                                    className="h-1 w-full appearance-none rounded bg-white/30 accent-white disabled:opacity-50"
                                />
                            </li>
                        ))}
                    </ul>
                    <button
                        type="button"
                        onClick={() => update(resetPlanes(planes))}
                        className="self-end rounded bg-white/10 px-2 py-1 text-[0.6875rem] text-white hover:bg-white/20"
                    >
                        Reset
                    </button>
                </>
            ) : null}
        </aside>
    );
}

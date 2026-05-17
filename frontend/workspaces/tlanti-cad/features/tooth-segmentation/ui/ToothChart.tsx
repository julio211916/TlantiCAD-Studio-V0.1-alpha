/**
 * ToothChart — presentational FDI (11..48) dental arch picker.
 *
 * Parent owns `states`; this component only paints and raises click events.
 * Does not know about segmentation backends or jobs.
 */

import React, { useMemo } from 'react';

import {
    PERMANENT_TEETH,
    type JawKind,
    type ToothDefinition,
    type ToothState,
    type ToothStatus,
} from '../domain/fdi-chart';

interface ToothChartProps {
    states: Record<number, ToothState>;
    onToothClick: (fdi: number, event: React.MouseEvent) => void;
    onToothAltClick?: (fdi: number, event: React.MouseEvent) => void;
    jaw: JawKind | 'both';
    compact?: boolean;
}

const ROW_Y_MAXILLA = 28;
const ROW_Y_MANDIBLE = 96;
const TOOTH_WIDTH = 18;
const TOOTH_GAP = 1.5;
const ORIGIN_X = 12;

function xFor(quadrant: ToothDefinition['quadrant'], position: number): number {
    // Layout: quadrant 1 runs right-to-left (positions 8..1), quadrant 2 runs left-to-right (1..8).
    // We render them as a single row, midline at centre.
    const slotWidth = TOOTH_WIDTH + TOOTH_GAP;
    const half = 8 * slotWidth;
    if (quadrant === 1 || quadrant === 4) {
        // Right side → positions 8,7,6,5,4,3,2,1 from left to right.
        return ORIGIN_X + (8 - position) * slotWidth;
    }
    // Left side (quadrant 2 or 3) → positions 1..8 from midline.
    return ORIGIN_X + half + (position - 1) * slotWidth;
}

function yFor(quadrant: ToothDefinition['quadrant']): number {
    return quadrant === 1 || quadrant === 2 ? ROW_Y_MAXILLA : ROW_Y_MANDIBLE;
}

function fillFor(status: ToothStatus, color: string | null): string {
    switch (status) {
        case 'segmented':
            return color ?? '#A6C9E3';
        case 'locked':
            return color ?? '#A6C9E3';
        case 'missing':
            return 'rgba(148, 163, 184, 0.15)';
        case 'unsegmented':
        default:
            return 'rgba(255, 255, 255, 0.08)';
    }
}

function strokeFor(status: ToothStatus): string {
    switch (status) {
        case 'segmented':
            return 'rgba(255,255,255,0.9)';
        case 'locked':
            return 'rgba(16,185,129,0.9)';
        case 'missing':
            return 'rgba(148,163,184,0.4)';
        case 'unsegmented':
        default:
            return 'rgba(203,213,225,0.6)';
    }
}

export function ToothChart({ states, onToothClick, onToothAltClick, jaw, compact }: ToothChartProps) {
    const visibleTeeth = useMemo(() => {
        if (jaw === 'both') return PERMANENT_TEETH;
        return PERMANENT_TEETH.filter((t) => t.jaw === jaw);
    }, [jaw]);

    const width = ORIGIN_X * 2 + 16 * (TOOTH_WIDTH + TOOTH_GAP);
    const height = jaw === 'both' ? 128 : 60;

    return (
        <svg
            role="img"
            aria-label="FDI tooth chart"
            viewBox={`0 0 ${width} ${height}`}
            className={[compact ? 'h-24' : 'h-32', 'w-full select-none'].join(' ')}
        >
            {visibleTeeth.map((tooth) => {
                const s: ToothState = states[tooth.fdi] ?? {
                    fdi: tooth.fdi,
                    status: 'unsegmented',
                    color: null,
                };
                const x = xFor(tooth.quadrant, tooth.position);
                const y = jaw === 'both' ? yFor(tooth.quadrant) : 16;
                const fill = fillFor(s.status, s.color);
                const stroke = strokeFor(s.status);
                return (
                    <g
                        key={tooth.fdi}
                        transform={`translate(${x} ${y})`}
                        onClick={(event) => {
                            if (event.altKey && onToothAltClick) {
                                onToothAltClick(tooth.fdi, event);
                            } else {
                                onToothClick(tooth.fdi, event);
                            }
                        }}
                        style={{ cursor: s.status === 'missing' ? 'not-allowed' : 'pointer' }}
                    >
                        <title>{`Tooth ${tooth.fdi} — ${tooth.label}`}</title>
                        <rect
                            width={TOOTH_WIDTH}
                            height={18}
                            rx={3}
                            ry={3}
                            fill={fill}
                            stroke={stroke}
                            strokeWidth={1}
                        />
                        <text
                            x={TOOTH_WIDTH / 2}
                            y={12}
                            textAnchor="middle"
                            fontSize={8}
                            fontFamily="ui-sans-serif, system-ui"
                            fill={
                                s.status === 'segmented' || s.status === 'locked'
                                    ? 'rgba(15,23,42,0.9)'
                                    : 'rgba(226,232,240,0.8)'
                            }
                        >
                            {tooth.fdi}
                        </text>
                    </g>
                );
            })}
        </svg>
    );
}

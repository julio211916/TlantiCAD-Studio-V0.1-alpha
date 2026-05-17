/**
 * BrushCursorOverlay — V230.
 *
 * R3F overlay that visualises the active freeform brush:
 *   - round-ball   → translucent sphere
 *   - pointed-knife → cone tip
 *   - flat-cylinder → disk
 *
 * Position is supplied by the parent (raycast hit) so the overlay stays
 * pure data-in / data-out. Visible only while the brush is actively
 * tracked (e.g. pointer over a tooth mesh).
 */

import React from 'react';

import type { FreeformBrushState, FreeformBrushType } from '../domain/freeform-brush';

export interface BrushCursorOverlayProps {
    brush: FreeformBrushState;
    /** World-space hit point. Hide cursor by passing null. */
    position: [number, number, number] | null;
    /** Mode color — defaults to add=green / remove=red / smooth=blue. */
    color?: string;
}

const COLOR_BY_TYPE: Record<FreeformBrushType, string> = {
    'round-ball': '#22c55e',
    'pointed-knife': '#fbbf24',
    'flat-cylinder': '#38bdf8',
};

export function BrushCursorOverlay({ brush, position, color }: BrushCursorOverlayProps) {
    if (!position) return null;
    const c = color ?? COLOR_BY_TYPE[brush.brushType];
    const radius = brush.sizeMm / 2;
    return (
        <group position={position}>
            {brush.brushType === 'round-ball' && (
                <mesh>
                    <sphereGeometry args={[radius, 18, 18]} />
                    <meshBasicMaterial color={c} transparent opacity={0.25} />
                </mesh>
            )}
            {brush.brushType === 'pointed-knife' && (
                <mesh rotation={[Math.PI, 0, 0]}>
                    <coneGeometry args={[radius, brush.sizeMm * 1.4, 16]} />
                    <meshBasicMaterial color={c} transparent opacity={0.35} />
                </mesh>
            )}
            {brush.brushType === 'flat-cylinder' && (
                <mesh rotation={[Math.PI / 2, 0, 0]}>
                    <cylinderGeometry args={[radius, radius, 0.3, 24]} />
                    <meshBasicMaterial color={c} transparent opacity={0.3} />
                </mesh>
            )}
            {/* Strength ring — outer wire to hint at falloff (mm). */}
            <mesh rotation={[Math.PI / 2, 0, 0]}>
                <ringGeometry args={[radius * 1.2, radius * 1.25, 32]} />
                <meshBasicMaterial color={c} transparent opacity={0.5 * brush.strength} side={2} />
            </mesh>
        </group>
    );
}

/**
 * TruSmileMaterial — V225.
 *
 * Wraps R3F's MeshPhysicalMaterial with parameters tuned for ceramic/zirconia
 * subsurface scattering. Three render modes mirror exocad:
 *   - 'trusmile' — final esthetic preview (default)
 *   - 'plaster'   — matte chalky look for design clarity
 *   - 'outline'   — flat shaded with edge outline (debug)
 *
 * Pure JSX — no internal state. Apply via `<mesh material={<TruSmileMaterial />}>`.
 */

import React from 'react';

export type TruSmileMode = 'trusmile' | 'plaster' | 'outline';

export interface TruSmileMaterialProps {
    mode?: TruSmileMode;
    /** Hex color of the restoration body. Default = warm enamel (#f4ead4). */
    color?: string;
    /** Tooth shade (Vita) — when set, picks a curated body color. */
    shade?: string;
}

const SHADE_RGB: Record<string, string> = {
    A1: '#fdf6dc',
    A2: '#f4ead4',
    A3: '#e8dcc1',
    A35: '#dac9aa',
    A4: '#c8b48d',
    B1: '#f7eedc',
    B2: '#ebdcc1',
    B3: '#dcc8a4',
    C1: '#e9dcc4',
    C2: '#d8c4a3',
    C3: '#c1a87f',
    D2: '#dcc8a8',
    D3: '#c4a884',
    BL1: '#fcfaf2',
    BL2: '#f8f4e6',
};

export function TruSmileMaterial({ mode = 'trusmile', color, shade }: TruSmileMaterialProps) {
    const baseColor = color ?? (shade ? SHADE_RGB[shade] ?? '#f4ead4' : '#f4ead4');

    if (mode === 'plaster') {
        return <meshStandardMaterial color={baseColor} roughness={0.95} metalness={0} />;
    }
    if (mode === 'outline') {
        return <meshBasicMaterial color={baseColor} wireframe={false} />;
    }
    // trusmile (default) — physical material with subsurface-ish settings.
    return (
        <meshPhysicalMaterial
            color={baseColor}
            roughness={0.32}
            metalness={0.02}
            transmission={0.18}
            thickness={2.5}
            ior={1.45}
            attenuationColor="#fff7e0"
            attenuationDistance={6.0}
            clearcoat={0.4}
            clearcoatRoughness={0.18}
            sheen={0.05}
        />
    );
}

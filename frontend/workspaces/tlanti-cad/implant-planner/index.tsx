/**
 * 3D implant planner (V270) — domain + R3F overlay primitive.
 *
 * Replaces the slider-only ImplantModule with a real 3D placement: each
 * implant carries a center (world coords) + axis vector + length + diameter.
 * The overlay renders a parametric cylinder + apex sphere; the parent owns
 * placement mutations (drag handles will land in V271+).
 *
 * The domain shape mirrors the existing TlantiToothState.implantMode value
 * so a case authored in DentalDB (DentalDb-style implant flag) projects
 * naturally into a list of `PlannedImplant` instances.
 */

import React from 'react';

export type Vec3 = [number, number, number];

export interface PlannedImplant {
    fdi: number;
    /** Apex (tip) center in world coordinates. */
    apex: Vec3;
    /** Coronal (top) center in world coordinates. */
    coronal: Vec3;
    diameterMm: number;
    /** Length is derived from |coronal - apex| but stored for fast access. */
    lengthMm: number;
    libraryRef?: { manufacturer: string; sku: string };
    /** "ok" green / "warning" amber / "error" red — feeds the safety-zone widget. */
    safety: 'ok' | 'warning' | 'error';
}

export interface ImplantPlannerState {
    implants: PlannedImplant[];
    activeFdi: number | null;
}

export function defaultImplantPlannerState(): ImplantPlannerState {
    return { implants: [], activeFdi: null };
}

/** Compute axis (apex → coronal direction) + length for a planned implant. */
export function implantAxis(implant: PlannedImplant): { dir: Vec3; lengthMm: number } {
    const [ax, ay, az] = implant.apex;
    const [cx, cy, cz] = implant.coronal;
    const dx = cx - ax;
    const dy = cy - ay;
    const dz = cz - az;
    const len = Math.hypot(dx, dy, dz) || 1;
    return { dir: [dx / len, dy / len, dz / len], lengthMm: len };
}

// ─── R3F overlay ──────────────────────────────────────────────────────

export interface ImplantPlannerOverlayProps {
    state: ImplantPlannerState;
    onSelect?: (fdi: number) => void;
    visible?: boolean;
}

const SAFETY_COLOR: Record<PlannedImplant['safety'], string> = {
    ok: '#22c55e',
    warning: '#fbbf24',
    error: '#ef4444',
};

export function ImplantPlannerOverlay({
    state,
    onSelect,
    visible = true,
}: ImplantPlannerOverlayProps) {
    if (!visible || state.implants.length === 0) return null;
    return (
        <group name="implant-planner-overlay">
            {state.implants.map((implant) => (
                <ImplantPrimitive
                    key={implant.fdi}
                    implant={implant}
                    selected={state.activeFdi === implant.fdi}
                    onSelect={() => onSelect?.(implant.fdi)}
                />
            ))}
        </group>
    );
}

function ImplantPrimitive({
    implant,
    selected,
    onSelect,
}: {
    implant: PlannedImplant;
    selected: boolean;
    onSelect: () => void;
}) {
    const { dir, lengthMm } = implantAxis(implant);
    const midX = (implant.apex[0] + implant.coronal[0]) / 2;
    const midY = (implant.apex[1] + implant.coronal[1]) / 2;
    const midZ = (implant.apex[2] + implant.coronal[2]) / 2;

    // Default cylinder geometry has its axis along Y; we compute the rotation
    // that aligns +Y with `dir`.
    const yaw = Math.atan2(dir[0], dir[2]);
    const pitch = Math.atan2(Math.hypot(dir[0], dir[2]), dir[1]);

    return (
        <group
            position={[midX, midY, midZ]}
            rotation={[pitch, yaw, 0]}
            onPointerDown={(e) => {
                e.stopPropagation();
                onSelect();
            }}
        >
            <mesh>
                <cylinderGeometry args={[implant.diameterMm / 2, implant.diameterMm / 2, lengthMm, 24]} />
                <meshStandardMaterial
                    color={SAFETY_COLOR[implant.safety]}
                    transparent
                    opacity={selected ? 0.85 : 0.55}
                    metalness={0.4}
                    roughness={0.35}
                    emissive={selected ? SAFETY_COLOR[implant.safety] : '#000'}
                    emissiveIntensity={selected ? 0.25 : 0}
                />
            </mesh>
            {/* Apex hint */}
            <mesh position={[0, -lengthMm / 2, 0]}>
                <sphereGeometry args={[implant.diameterMm / 2, 16, 16]} />
                <meshStandardMaterial color={SAFETY_COLOR[implant.safety]} />
            </mesh>
        </group>
    );
}

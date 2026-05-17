/**
 * ScrewChannelOverlay — V219 + V220 + V221.
 *
 * R3F overlay that draws per-tooth screw-channel widgets:
 *   - 4 cardinal arrows (mesial / distal / buccal / lingual) the user drags
 *     to adjust the channel thickness (V219)
 *   - 1 toggle disk per channel (black=locked, green=free) — click cycles
 *     stick/unstick (V220)
 *   - center sphere with Ctrl+click → flatten-top gesture (V221)
 *
 * Mounted as `overlayChildren` of CanvasScene so the widgets live in the
 * same R3F scene as the meshes; positions come from props (caller computes
 * channel center per FDI). Pure presentational.
 */

import React from 'react';
import * as THREE from 'three';

import type { ScrewHoleToothState } from '../domain/screw-holes';

export interface ChannelAnchor {
    fdi: string;
    /** World-space position of the channel center. */
    position: [number, number, number];
    /** Effective thickness for this channel (mm). */
    thicknessMm: number;
}

export interface ScrewChannelOverlayProps {
    teeth: readonly ScrewHoleToothState[];
    anchors: readonly ChannelAnchor[];
    visible?: boolean;
    onThicknessChange: (fdi: string, thicknessMm: number, all?: boolean) => void;
    onToggleLock: (fdi: string, all?: boolean) => void;
    onFlattenTop: (fdi: string) => void;
}

const CARDINALS: Array<{ id: 'mesial' | 'distal' | 'buccal' | 'lingual'; vec: THREE.Vector3 }> = [
    { id: 'mesial', vec: new THREE.Vector3(1, 0, 0) },
    { id: 'distal', vec: new THREE.Vector3(-1, 0, 0) },
    { id: 'buccal', vec: new THREE.Vector3(0, 0, 1) },
    { id: 'lingual', vec: new THREE.Vector3(0, 0, -1) },
];

export function ScrewChannelOverlay(props: ScrewChannelOverlayProps) {
    const { teeth, anchors, visible = true, onThicknessChange, onToggleLock, onFlattenTop } = props;
    if (!visible || anchors.length === 0) return null;
    return (
        <group name="screw-channel-widgets">
            {anchors.map((anchor) => {
                const tooth = teeth.find((t) => t.fdi === anchor.fdi);
                if (!tooth || tooth.mode === 'none') return null;
                return (
                    <ChannelWidget
                        key={anchor.fdi}
                        anchor={anchor}
                        tooth={tooth}
                        onThicknessChange={onThicknessChange}
                        onToggleLock={onToggleLock}
                        onFlattenTop={onFlattenTop}
                    />
                );
            })}
        </group>
    );
}

function ChannelWidget({
    anchor,
    tooth,
    onThicknessChange,
    onToggleLock,
    onFlattenTop,
}: {
    anchor: ChannelAnchor;
    tooth: ScrewHoleToothState;
    onThicknessChange: (fdi: string, mm: number, all?: boolean) => void;
    onToggleLock: (fdi: string, all?: boolean) => void;
    onFlattenTop: (fdi: string) => void;
}) {
    const [px, py, pz] = anchor.position;
    const t = anchor.thicknessMm;

    return (
        <group position={[px, py, pz]}>
            {/* Center sphere — Ctrl+click flattens top (V221), plain click toggles lock (V220). */}
            <mesh
                onPointerDown={(e) => {
                    e.stopPropagation();
                    if (e.ctrlKey || e.metaKey) {
                        onFlattenTop(anchor.fdi);
                    } else {
                        onToggleLock(anchor.fdi, e.shiftKey);
                    }
                }}
            >
                <sphereGeometry args={[0.6, 12, 12]} />
                <meshStandardMaterial
                    color={tooth.locked ? '#1f2937' : '#10b981'}
                    emissive={tooth.locked ? '#000' : '#10b981'}
                    emissiveIntensity={tooth.locked ? 0 : 0.4}
                />
            </mesh>
            {/* 4 cardinal arrows (V219) — drag to scale thickness. The actual
              drag math runs in the parent component listening to pointer
              events. Here we only render the visual handles. */}
            {CARDINALS.map((c) => {
                const ax = c.vec.x * t;
                const ay = c.vec.y * t;
                const az = c.vec.z * t;
                return (
                    <group key={c.id}>
                        <mesh position={[ax, ay, az]}>
                            <coneGeometry args={[0.25, 0.6, 8]} />
                            <meshStandardMaterial color="#facc15" emissive="#facc15" emissiveIntensity={0.2} />
                        </mesh>
                    </group>
                );
            })}
            {/* Toggle disk below the center sphere (V220). */}
            <mesh
                position={[0, -1.0, 0]}
                rotation={[Math.PI / 2, 0, 0]}
                onPointerDown={(e) => {
                    e.stopPropagation();
                    onToggleLock(anchor.fdi, e.ctrlKey || e.metaKey);
                }}
            >
                <circleGeometry args={[0.45, 16]} />
                <meshBasicMaterial color={tooth.locked ? '#0f172a' : '#22c55e'} />
            </mesh>
            {/* Helper labels — subtle, only when widget is hovered (no built-in
              hover state here; future sprint can add). */}
        </group>
    );
}

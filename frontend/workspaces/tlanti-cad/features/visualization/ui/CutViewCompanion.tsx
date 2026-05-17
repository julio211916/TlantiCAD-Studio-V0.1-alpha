/**
 * CutViewCompanion — V226.
 *
 * Floating side panel that mirrors the active sectional plane in a small
 * R3F mini-canvas. The user can navigate the section without moving the
 * main camera. Reuses `SectionalPlane` state (V224) so changes anywhere
 * propagate immediately.
 */

import React from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';

import type { SectionalPlane } from '../domain/sectional-plane';

export interface CutViewCompanionProps {
    open: boolean;
    onClose: () => void;
    plane: SectionalPlane;
    /** Geometry to render in the side canvas — the same restoration mesh. */
    children?: React.ReactNode;
}

export function CutViewCompanion({ open, onClose, plane, children }: CutViewCompanionProps) {
    if (!open) return null;
    return (
        <aside
            role="complementary"
            aria-label="Sectional cut view"
            className="pointer-events-auto fixed bottom-4 left-4 z-30 flex w-[280px] flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-xl"
        >
            <header className="flex items-center justify-between border-b border-border px-3 py-2">
                <div>
                    <h3 className="text-[11px] font-semibold text-text-primary">Cut view</h3>
                    <p className="text-[10px] text-text-secondary">
                        offset {plane.offsetMm.toFixed(2)} mm · normal {plane.normal.map((v) => v.toFixed(1)).join(', ')}
                    </p>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    className="rounded border border-border bg-surface-sunken px-1.5 py-0.5 font-mono text-[10px] text-text-secondary"
                    title="Close cut view"
                >
                    ✕
                </button>
            </header>
            <div className="aspect-square w-full">
                <Canvas
                    shadows={false}
                    camera={{ position: [0, 0, 60], fov: 35, near: 0.1, far: 5000 }}
                    dpr={[1, 1.5]}
                    gl={{ preserveDrawingBuffer: false, localClippingEnabled: true }}
                >
                    <ambientLight intensity={0.7} />
                    <directionalLight position={[10, 20, 10]} intensity={0.8} />
                    {children}
                    <OrbitControls makeDefault />
                </Canvas>
            </div>
        </aside>
    );
}

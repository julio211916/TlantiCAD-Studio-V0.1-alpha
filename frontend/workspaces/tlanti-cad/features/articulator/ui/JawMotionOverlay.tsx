/**
 * JawMotionOverlay — V210.
 *
 * R3F overlay that renders the jaw-motion path as a polyline + frame markers.
 * Mounted inside CanvasScene; shows the trajectory returned by
 * `/cad/articulator/simulate`.
 */

import React, { useMemo } from 'react';
import * as THREE from 'three';

import type { JawFrame } from '../domain/articulator-config';

export interface JawMotionOverlayProps {
    frames: readonly JawFrame[];
    /** mm — visual scale multiplier (default 10× since translations are small). */
    visualScale?: number;
    /** Hex color for the trajectory line. */
    color?: string;
    /** Hide overlay without unmounting. */
    visible?: boolean;
}

export function JawMotionOverlay({
    frames,
    visualScale = 10,
    color = '#fb923c',
    visible = true,
}: JawMotionOverlayProps) {
    const geometry = useMemo(() => {
        const geom = new THREE.BufferGeometry();
        const points = frames.map(
            (f) =>
                new THREE.Vector3(
                    f.translationMm[0] * visualScale,
                    f.translationMm[1] * visualScale,
                    f.translationMm[2] * visualScale,
                ),
        );
        geom.setFromPoints(points);
        return geom;
    }, [frames, visualScale]);

    if (!visible || frames.length < 2) return null;

    return (
        <group name="jaw-motion-overlay">
            <line>
                <primitive object={geometry} attach="geometry" />
                <lineBasicMaterial color={color} linewidth={2} />
            </line>
            {frames.map((f, i) => (
                <mesh
                    key={i}
                    position={[
                        f.translationMm[0] * visualScale,
                        f.translationMm[1] * visualScale,
                        f.translationMm[2] * visualScale,
                    ]}
                >
                    <sphereGeometry args={[0.4, 8, 8]} />
                    <meshBasicMaterial color={i === 0 ? '#22c55e' : i === frames.length - 1 ? '#ef4444' : color} />
                </mesh>
            ))}
        </group>
    );
}

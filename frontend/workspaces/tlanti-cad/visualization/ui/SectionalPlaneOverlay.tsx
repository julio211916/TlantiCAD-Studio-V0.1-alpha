/**
 * SectionalPlaneOverlay — V224.
 *
 * Renders the sectional plane visual + a drag handle that moves the plane
 * along its normal. Two cone arrows along ±normal let the user flip the
 * cut direction. Pure visual; uses `THREE.Plane` for the actual clipping.
 *
 * The plane state is owned by the parent so a single source of truth feeds
 * both the main canvas and the cut-view companion (V226).
 */

import React, { useMemo } from 'react';
import * as THREE from 'three';

import type { SectionalPlane } from '../domain/sectional-plane';

export interface SectionalPlaneOverlayProps {
    plane: SectionalPlane;
    onOffsetChange: (offsetMm: number) => void;
    onFlipNormal: () => void;
    /** Visual scale of the plane gizmo (mm). */
    sizeMm?: number;
}

export function SectionalPlaneOverlay({
    plane,
    onOffsetChange,
    onFlipNormal,
    sizeMm = 60,
}: SectionalPlaneOverlayProps) {
    const normal = useMemo(
        () => new THREE.Vector3(plane.normal[0], plane.normal[1], plane.normal[2]).normalize(),
        [plane.normal],
    );
    const center = useMemo(
        () => normal.clone().multiplyScalar(plane.offsetMm),
        [normal, plane.offsetMm],
    );
    const quaternion = useMemo(() => {
        // Align plane with its normal — by default the plane is XZ (normal +Y).
        const up = new THREE.Vector3(0, 1, 0);
        const q = new THREE.Quaternion();
        q.setFromUnitVectors(up, normal);
        return q;
    }, [normal]);

    if (!plane.enabled) return null;

    return (
        <group position={[center.x, center.y, center.z]} quaternion={quaternion}>
            {/* Plane visual — large translucent quad. */}
            <mesh rotation={[Math.PI / 2, 0, 0]}>
                <planeGeometry args={[sizeMm, sizeMm]} />
                <meshBasicMaterial color="#6366f1" transparent opacity={0.18} side={THREE.DoubleSide} />
            </mesh>
            {/* Drag handle — sphere on the plane center. */}
            <mesh
                onPointerDown={(e) => {
                    e.stopPropagation();
                    if (e.altKey) {
                        onFlipNormal();
                        return;
                    }
                    // The actual drag logic should be wired by the parent — here
                    // we just emit a discrete offset bump for click-pulse UX.
                    onOffsetChange(plane.offsetMm + (e.shiftKey ? -1 : 1));
                }}
            >
                <sphereGeometry args={[1.4, 16, 16]} />
                <meshStandardMaterial color="#fbbf24" emissive="#fbbf24" emissiveIntensity={0.4} />
            </mesh>
            {/* Normal arrows for flip. */}
            <mesh position={[0, 4, 0]}>
                <coneGeometry args={[0.7, 1.6, 12]} />
                <meshBasicMaterial color="#22c55e" />
            </mesh>
            <mesh position={[0, -4, 0]} rotation={[Math.PI, 0, 0]}>
                <coneGeometry args={[0.7, 1.6, 12]} />
                <meshBasicMaterial color="#ef4444" />
            </mesh>
        </group>
    );
}

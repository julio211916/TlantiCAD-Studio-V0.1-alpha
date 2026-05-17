/**
 * DistanceShaderOverlay — V223.
 *
 * R3F primitive that overlays a translucent shaded mesh on the active
 * restoration to visualise per-vertex distance to antagonist / adjacents.
 * The shader maps the distance scalar (mm) to a blue→green→red gradient
 * matching the ShowDistances color reference bar.
 *
 * The vertex distance attribute is supplied by the parent (computed
 * client-side from the canvas geometry, or fetched from
 * /cad/show-distances/compute in a future iteration).
 *
 * Props are pure data — no Three.js refs. Mounted via
 * `CanvasScene.overlayChildren`.
 */

import React, { useMemo } from 'react';
import * as THREE from 'three';

export interface DistanceShaderOverlayProps {
    /** Existing geometry of the restoration. */
    geometry: THREE.BufferGeometry | null;
    /** Per-vertex signed distance (mm). length === geometry.attributes.position.count. */
    distancesMm: Float32Array | null;
    /** Half-range for the gradient (mm). */
    colorScaleMm: number;
    visible?: boolean;
    /** Mesh transform in world space. */
    matrix?: THREE.Matrix4;
}

const VERTEX_SHADER = /* glsl */ `
varying float vDist;
attribute float aDist;
void main() {
  vDist = aDist;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const FRAGMENT_SHADER = /* glsl */ `
uniform float uScale;
varying float vDist;

vec3 colorRamp(float t) {
  // t ∈ [-1, 1] → blue (-1) → green (0) → red (+1)
  float r = clamp(t, 0.0, 1.0);
  float b = clamp(-t, 0.0, 1.0);
  float g = 1.0 - r - b;
  return vec3(r, g, b);
}

void main() {
  float t = clamp(vDist / max(uScale, 1e-3), -1.0, 1.0);
  gl_FragColor = vec4(colorRamp(t), 0.55);
}
`;

export function DistanceShaderOverlay({
    geometry,
    distancesMm,
    colorScaleMm,
    visible = true,
    matrix,
}: DistanceShaderOverlayProps) {
    const enriched = useMemo(() => {
        if (!geometry || !distancesMm) return null;
        const cloned = geometry.clone();
        cloned.setAttribute('aDist', new THREE.BufferAttribute(distancesMm, 1));
        return cloned;
    }, [geometry, distancesMm]);

    const material = useMemo(() => {
        return new THREE.ShaderMaterial({
            vertexShader: VERTEX_SHADER,
            fragmentShader: FRAGMENT_SHADER,
            uniforms: { uScale: { value: colorScaleMm } },
            transparent: true,
            depthWrite: false,
        });
    }, [colorScaleMm]);

    if (!visible || !enriched) return null;

    return (
        <mesh geometry={enriched} material={material} matrixAutoUpdate={!matrix} matrix={matrix} />
    );
}

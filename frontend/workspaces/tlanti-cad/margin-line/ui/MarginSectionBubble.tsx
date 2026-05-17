/**
 * MarginSectionBubble — V232.
 *
 * Floating 2D cross-section preview that follows the cursor while the user
 * is hovering the scan in the margin-detection step. Helps locate the
 * correct seed-point for the magnetic line by showing the curvature
 * profile under the pointer.
 *
 * Pure presentational: parent supplies the polyline of the cut (computed
 * from a CPU-side mesh slice or from the GPU). When `polyline` is null the
 * bubble shows a dashed placeholder so the lab tech sees that the
 * preview is wired even before raycast lands on a triangle.
 */

import React from 'react';

export interface MarginSectionBubbleProps {
    /** Screen-space anchor in pixels (relative to the page). */
    anchorPx: { x: number; y: number } | null;
    /**
     * 2D polyline (mm) — the cross-section curve. Each entry is
     * `[u, v]` where u = horizontal offset along the cut plane, v = depth.
     */
    polyline: ReadonlyArray<readonly [number, number]> | null;
    /** Visual half-width of the bubble (mm). Default 4mm. */
    rangeMm?: number;
    /** Hide bubble without unmounting. */
    visible?: boolean;
}

const SIZE_PX = 110;
const PADDING = 8;

export function MarginSectionBubble({
    anchorPx,
    polyline,
    rangeMm = 4,
    visible = true,
}: MarginSectionBubbleProps) {
    if (!visible || !anchorPx) return null;

    const left = anchorPx.x + 18;
    const top = anchorPx.y - SIZE_PX - 18;

    return (
        <div
            role="img"
            aria-label="Sectional preview under cursor"
            className="pointer-events-none fixed z-[140] rounded-lg border border-white/15 bg-slate-900/90 p-1 shadow-xl backdrop-blur"
            style={{ left, top, width: SIZE_PX, height: SIZE_PX }}
            data-visual-qa="margin-section-bubble"
        >
            <svg
                viewBox={`0 0 ${SIZE_PX} ${SIZE_PX}`}
                width={SIZE_PX}
                height={SIZE_PX}
                className="block"
            >
                {/* Crosshair — the cursor projection. */}
                <line
                    x1={SIZE_PX / 2}
                    y1={0}
                    x2={SIZE_PX / 2}
                    y2={SIZE_PX}
                    stroke="rgba(255,255,255,0.18)"
                    strokeWidth={1}
                />
                <line
                    x1={0}
                    y1={SIZE_PX / 2}
                    x2={SIZE_PX}
                    y2={SIZE_PX / 2}
                    stroke="rgba(255,255,255,0.18)"
                    strokeWidth={1}
                />
                {/* Polyline — projects mm into pixels around the center. */}
                {polyline && polyline.length >= 2 ? (
                    <polyline
                        fill="none"
                        stroke="#22d3ee"
                        strokeWidth={1.8}
                        points={polyline
                            .map(([u, v]) => {
                                const px = SIZE_PX / 2 + (u / rangeMm) * (SIZE_PX / 2 - PADDING);
                                const py = SIZE_PX / 2 + (v / rangeMm) * (SIZE_PX / 2 - PADDING);
                                return `${px.toFixed(1)},${py.toFixed(1)}`;
                            })
                            .join(' ')}
                    />
                ) : (
                    <text
                        x={SIZE_PX / 2}
                        y={SIZE_PX / 2 + 4}
                        textAnchor="middle"
                        fontFamily="Space Mono, monospace"
                        fontSize="9"
                        fill="rgba(255,255,255,0.35)"
                    >
                        no section
                    </text>
                )}
                {/* Scale tick. */}
                <text
                    x={6}
                    y={SIZE_PX - 4}
                    fontFamily="Space Mono, monospace"
                    fontSize="9"
                    fill="rgba(255,255,255,0.45)"
                >
                    ±{rangeMm} mm
                </text>
            </svg>
        </div>
    );
}

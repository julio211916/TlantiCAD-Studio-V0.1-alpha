/**
 * Public surface of the margin-line feature (V23).
 */

export type {
    DrawMode,
    MarginCorrectInput,
    MarginDetectInput,
    MarginLine,
    MarginMode,
    MarginRepairInput,
    MarginTool,
    Vec3,
} from './domain/margin-line';
export {
    polylineBoundsDiagonal,
    polylinePerimeterMm,
} from './domain/margin-line';

export type { MarginDetectionPort } from './application/margin-detection-port';
export { createBackendMarginAdapter } from './infrastructure/backend-margin-adapter';

export { MarginDetectPanel } from './ui/MarginDetectPanel';
export { useMarginDetection } from './ui/useMarginDetection';
export { MarginSectionBubble } from './ui/MarginSectionBubble';
export type { MarginSectionBubbleProps } from './ui/MarginSectionBubble';

export type { InsertionAxis, UndercutStats, Vec3 } from './domain/insertion-axis';
export { UNDERCUT_LEGEND, axisAngleDeg, axisDot, axisLength, normaliseAxis } from './domain/insertion-axis';

export type { InsertionDetectionPort, InsertionDetectInput } from './application/insertion-direction-port';
export { createBackendInsertionAdapter } from './infrastructure/backend-insertion-adapter';

export { InsertionDirectionPanel } from './ui/InsertionDirectionPanel';

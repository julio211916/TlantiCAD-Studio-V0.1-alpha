/**
 * Port contract for insertion-direction detection + bridge axis unification.
 */

import type { InsertionAxis, Vec3 } from '../domain/insertion-axis';

export interface InsertionDetectInput {
    meshPath: string;
    toothFdi: number;
    marginPolyline?: Vec3[];
}

export interface InsertionDetectionPort {
    detect(input: InsertionDetectInput): Promise<InsertionAxis>;
    unifyBridge(axes: Vec3[], weights?: number[]): Promise<{ axis: Vec3; maxDeviationDegrees: number }>;
}

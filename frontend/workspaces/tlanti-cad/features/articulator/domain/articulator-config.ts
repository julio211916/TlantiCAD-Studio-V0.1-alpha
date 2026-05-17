/**
 * Virtual articulator domain (V123).
 *
 * Mirrors the exocad articulator parameters: condyle inclination, Bennett
 * angle, immediate side shift, intercondylar distance, incisal guidance.
 */

export type JawMovement = 'protrusive' | 'laterotrusive' | 'retrusive';

export interface ArticulatorConfig {
    condyleInclinationDeg: number;
    bennettAngleDeg: number;
    immediateSideShiftMm: number;
    intercondylarDistanceMm: number;
    incisalGuidanceDeg: number;
}

export interface JawFrame {
    t: number;
    translationMm: [number, number, number];
    rotationDeg: [number, number, number];
}

export function defaultArticulatorConfig(): ArticulatorConfig {
    // Anatomical mean (Bonwill triangle).
    return {
        condyleInclinationDeg: 35,
        bennettAngleDeg: 7.5,
        immediateSideShiftMm: 0.5,
        intercondylarDistanceMm: 110,
        incisalGuidanceDeg: 15,
    };
}

export function frameDistanceMm(frame: JawFrame): number {
    const [x, y, z] = frame.translationMm;
    return Math.sqrt(x * x + y * y + z * z);
}

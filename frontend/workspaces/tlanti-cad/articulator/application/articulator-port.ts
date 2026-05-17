/**
 * Port for the virtual articulator backend.
 */

import type { ArticulatorConfig, JawFrame, JawMovement } from '../domain/articulator-config';

export interface ArticulatorSimulateInput {
    config: ArticulatorConfig;
    movement: JawMovement;
    frames: number;
}

export interface ArticulatorSimulateOutput {
    frames: JawFrame[];
    backend: string;
}

export interface ArticulatorPort {
    getConfig(): Promise<ArticulatorConfig>;
    simulate(input: ArticulatorSimulateInput): Promise<ArticulatorSimulateOutput>;
    setInfluencingTeeth(fdis: number[]): Promise<{ ok: boolean; count: number }>;
}

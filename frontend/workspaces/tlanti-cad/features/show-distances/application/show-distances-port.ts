import type { DistanceStats } from '../domain/distance-vis';

export interface ComputeDistancesInput {
    restorationPath: string;
    antagonistPath?: string | null;
    mesialPath?: string | null;
    distalPath?: string | null;
    includeHealthy?: boolean;
    dynamic?: boolean;
    colorScaleMm: number;
}

export interface ComputeDistancesOutput {
    stats: DistanceStats[];
    colorScaleMm: number;
    backend: string;
}

export interface ShowDistancesPort {
    compute(input: ComputeDistancesInput): Promise<ComputeDistancesOutput>;
}

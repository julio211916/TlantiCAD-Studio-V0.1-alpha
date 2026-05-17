/**
 * Port for the screw-holes CSG operation.
 */

export interface ScrewHolesApplyInput {
    caseFolderPath: string;
    offsetMm: number;
    minDiameterMm: number;
    toothMask: Record<string, boolean>;
}

export interface ScrewHolesApplyOutput {
    appliedTeeth: string[];
    skippedTeeth: string[];
    watertight: boolean;
    effectiveMinDiameterMm: number;
    backend: string;
}

export interface ScrewHolesPort {
    apply(input: ScrewHolesApplyInput): Promise<ScrewHolesApplyOutput>;
    clear(caseFolderPath: string): Promise<boolean>;
}

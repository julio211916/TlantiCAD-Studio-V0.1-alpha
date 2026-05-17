/**
 * Abutment port — V218.
 *
 * Application boundary for custom abutment work.
 *
 * Validation can still be delegated to the Python sidecar while heavy CAD
 * mesh generation and screw-channel planning are native Rust/Tauri calls.
 */

export type AbutmentPoint3 = readonly [number, number, number];

export type AbutmentProfilePreset =
    | 'Default'
    | 'Round'
    | 'Rectangle'
    | 'Square'
    | 'Shoulder'
    | 'Clip';

export interface AbutmentValidationIssue {
    code: string;
    severity: 'error' | 'warning';
    message: string;
}

export interface AbutmentValidationResponse {
    ok: boolean;
    issues: AbutmentValidationIssue[];
    backend: string;
}

export interface AbutmentValidationRequest {
    outputPath: string;
    minThicknessMm: number;
    angulatedScrewChannelDeg: number;
    implantLibraryMaxDeg?: number;
}

export interface AbutmentGenerateMeshRequest {
    caseFolderPath: string;
    outputFileName?: string;
    marginPolyline: AbutmentPoint3[];
    implantAxis: AbutmentPoint3;
    implantDiameterMm: number;
    emergenceHeightMm: number;
    shoulderWidthMm: number;
    taperDegrees: number;
    axialRings?: number;
    profile?: AbutmentProfilePreset;
}

export interface AbutmentGenerateMeshResponse {
    outputPath: string;
    vertexCount: number;
    triangleCount: number;
    watertight: boolean;
    volumeMm3: number;
    warnings: string[];
    backend: string;
}

export interface AbutmentScrewChannelRequest {
    implantPosition: AbutmentPoint3;
    implantAxis: AbutmentPoint3;
    prostheticAxis: AbutmentPoint3;
    lengthMm: number;
    diameterMm: number;
    libraryAngleLimitDeg: number;
}

export interface AbutmentScrewChannelPlan {
    points: AbutmentPoint3[];
    angleDegrees: number;
    diameterMm: number;
    validForLibraryLimit: boolean;
    warnings: string[];
}

export interface AbutmentPort {
    validate(input: AbutmentValidationRequest): Promise<AbutmentValidationResponse>;
    generateMesh(input: AbutmentGenerateMeshRequest): Promise<AbutmentGenerateMeshResponse>;
    planScrewChannel(input: AbutmentScrewChannelRequest): Promise<AbutmentScrewChannelPlan>;
}

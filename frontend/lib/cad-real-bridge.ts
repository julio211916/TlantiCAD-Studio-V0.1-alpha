/**
 * AR-V363 → AR-V378 — Real CAD engine bridge.
 *
 * One adapter per Tauri command surfaced by the porting work from `artifacts/`.
 * All wrappers share the same pattern: bail out with a typed `bridge-not-available`
 * error when not running inside Tauri so call-sites can fall back to the Python
 * mock or the legacy backend.
 */

import { isTauriRuntime } from '../platform/desktop-system';
import { logger } from './logger';

export interface CadBridgeError {
    kind: string;
    message?: string;
}

async function invokeOrThrow<T>(command: string, args: Record<string, unknown>): Promise<T> {
    if (!isTauriRuntime()) {
        throw { kind: 'bridge-not-available', message: `${command} requires Tauri` } as CadBridgeError;
    }
    const { invoke } = await import('@tauri-apps/api/core');
    try {
        return await invoke<T>(command, args);
    } catch (err) {
        logger.warn(`${command} invoke failed`, err);
        throw err as CadBridgeError;
    }
}

// ─── AR-V363 mesh kernel ops ──────────────────────────────────────────────

export type AddRemoveOp = 'add' | 'remove' | 'drop-faces';

export interface AddRemoveRequest {
    input: string;
    output: string;
    op: AddRemoveOp;
    center: [number, number, number];
    radiusMm: number;
    amountMm?: number;
    falloff?: number;
}

export interface AddRemoveResponse {
    output: string;
    verticesModified: number;
    facesRemoved: number;
    maxDisplacementMm: number;
    backend: string;
}

export const meshKernelAddRemove = (request: AddRemoveRequest) =>
    invokeOrThrow<AddRemoveResponse>('mesh_kernel_add_remove', { request });

export interface CompareRequest { a: string; b: string; }

export interface CompareResponse {
    hausdorffAToBMm: number;
    hausdorffBToAMm: number;
    hausdorffSymmetricMm: number;
    rmsAToBMm: number;
    rmsBToAMm: number;
    meanAToBMm: number;
    meanBToAMm: number;
    vertexCountA: number;
    vertexCountB: number;
    backend: string;
}

export const meshKernelCompare = (request: CompareRequest) =>
    invokeOrThrow<CompareResponse>('mesh_kernel_compare', { request });

export interface AdaptToGingivaRequest {
    source: string;
    target: string;
    output: string;
    axisOcclusal: [number, number, number];
    minDistanceMm?: number;
    snapToGingiva?: boolean;
    evenOutIterations?: number;
    evenOutLambda?: number;
}

export interface AdaptToGingivaResponse {
    output: string;
    verticesMoved: number;
    maxDisplacementMm: number;
    meanDisplacementMm: number;
    backend: string;
}

export const meshKernelAdaptToGingiva = (request: AdaptToGingivaRequest) =>
    invokeOrThrow<AdaptToGingivaResponse>('mesh_kernel_adapt_to_gingiva', { request });

// ─── AR-V364 insertion direction ──────────────────────────────────────────

export interface InsertionComputeRequest {
    input: string;
    occlusalHint: [number, number, number];
    mesialDistal?: [number, number, number];
    thresholds?: { warnDot: number; errorDot: number };
}

export interface InsertionComputeResponse {
    axis: { axis: [number, number, number]; directionality: number };
    secondary?: [number, number, number];
    severityCounts: { ok: number; warning: number; error: number };
    vertexCount: number;
    backend: string;
}

export const cadInsertionCompute = (request: InsertionComputeRequest) =>
    invokeOrThrow<InsertionComputeResponse>('cad_insertion_compute', { request });

export interface UnifyBridgeRequest {
    axes: Array<[number, number, number]>;
    weights?: number[];
    maxDeviationDeg?: number;
}

export interface UnifyBridgeResponse {
    unified: [number, number, number];
    maxDeviationDeg: number;
    backend: string;
}

export const cadInsertionUnifyBridge = (request: UnifyBridgeRequest) =>
    invokeOrThrow<UnifyBridgeResponse>('cad_insertion_unify_bridge', { request });

// ─── AR-V365 margin ──────────────────────────────────────────────────────

export type MarginDetectMode = 'boundary' | 'curvature';

export interface MarginPolyline { points: Array<[number, number, number]>; isClosed: boolean; }

export interface MarginDetectRequest {
    input: string;
    mode: MarginDetectMode;
    insertionAxis?: [number, number, number];
    curvatureThreshold?: number;
    perpendicularTolDot?: number;
}

export interface MarginDetectResponse {
    polylines: MarginPolyline[];
    backend: string;
}

export const cadMarginDetectReal = (request: MarginDetectRequest) =>
    invokeOrThrow<MarginDetectResponse>('cad_margin_detect_real', { request });

export interface MarginCorrectRequest { polyline: MarginPolyline; iterations?: number; lambda?: number; }
export interface MarginCorrectResponse {
    polyline: MarginPolyline;
    originalLengthMm: number;
    correctedLengthMm: number;
    backend: string;
}

export const cadMarginCorrectReal = (request: MarginCorrectRequest) =>
    invokeOrThrow<MarginCorrectResponse>('cad_margin_correct_real', { request });

export interface MarginRepairRequest { polyline: MarginPolyline; gapThresholdMm?: number; }
export interface MarginRepairResponse {
    polyline: MarginPolyline;
    pointsInserted: number;
    backend: string;
}

export const cadMarginRepairReal = (request: MarginRepairRequest) =>
    invokeOrThrow<MarginRepairResponse>('cad_margin_repair_real', { request });

// ─── AR-V366 connector ───────────────────────────────────────────────────

export interface ConnectorRequest {
    crownA: string;
    crownB: string;
    output: string;
    widthMm?: number;
    heightMm?: number;
    occlusalUp?: [number, number, number];
    kind?: 'rigid' | 'semi-precision' | 'precision';
    axialSegments?: number;
    radialSegments?: number;
}

export interface ConnectorResponse {
    output: string;
    anchorA: [number, number, number];
    anchorB: [number, number, number];
    triangles: number;
    crossSectionMm2: number;
    backend: string;
}

export const cadBridgeConnectorCreate = (request: ConnectorRequest) =>
    invokeOrThrow<ConnectorResponse>('cad_bridge_connector_create', { request });

// ─── AR-V367 crown bottom ────────────────────────────────────────────────

export interface CrownBottomRequest {
    prepStl: string;
    output: string;
    marginPolyline: Array<[number, number, number]>;
    marginClosed?: boolean;
    insertionAxis: [number, number, number];
    gapCementMm?: number;
    gapBorderMm?: number;
    borderWidthMm?: number;
    rampMm?: number;
    maxOffsetMm?: number;
    material?: string;
}

export interface CrownBottomResponse {
    output: string;
    triangles: number;
    verticesOffset: number;
    maxDisplacementMm: number;
    meanDisplacementMm: number;
    gapCementMm: number;
    gapBorderMm: number;
    backend: string;
}

export const cadCrownBottomGenerate = (request: CrownBottomRequest) =>
    invokeOrThrow<CrownBottomResponse>('cad_crown_bottom_generate', { request });

// ─── AR-V371 abutment ────────────────────────────────────────────────────

export type AbutmentStyle = 'cylindrical' | 'angular' | 'standard' | 'legacy';

export interface AbutmentLimitWarning {
    kind: string;
    severity: 'Ok' | 'Warning' | 'Error';
    message: string;
}

export interface AbutmentGenerateRequest {
    marginPolyline: Array<[number, number, number]>;
    insertionAxis: [number, number, number];
    output: string;
    style?: AbutmentStyle;
    heightMm?: number;
    topRadiusMm?: number;
    axialSegments?: number;
    radialSegments?: number;
    screwChannelDiameterMm?: number;
    screwChannelAngleDeg?: number;
    anatomicBulge?: number;
}

export interface AbutmentGenerateResponse {
    output: string;
    triangles: number;
    vertices: number;
    volumeMm3: number;
    warnings: AbutmentLimitWarning[];
    backend: string;
}

export const cadAbutmentGenerateReal = (request: AbutmentGenerateRequest) =>
    invokeOrThrow<AbutmentGenerateResponse>('cad_abutment_generate_real', { request });

export interface AbutmentValidateResponse {
    warnings: AbutmentLimitWarning[];
    backend: string;
}

export const cadAbutmentValidateReal = (request: Omit<AbutmentGenerateRequest, 'output' | 'insertionAxis'>) =>
    invokeOrThrow<AbutmentValidateResponse>('cad_abutment_validate_real', { request });

// ─── AR-V374 freeform brush ──────────────────────────────────────────────

export interface PaintPullRequest {
    input: string;
    output: string;
    center: [number, number, number];
    radiusMm: number;
    amountMm: number;
    strength?: number;
    falloff?: number;
}
export interface BrushOpResponse {
    output: string;
    verticesAffected: number;
    maxDisplacementMm: number;
    meanDisplacementMm: number;
    backend: string;
}

export const cadFreeformPaintPull = (request: PaintPullRequest) =>
    invokeOrThrow<BrushOpResponse>('cad_freeform_paint_pull', { request });

export interface PaintSmoothRequest {
    input: string;
    output: string;
    center: [number, number, number];
    radiusMm: number;
    iterations?: number;
    strength?: number;
    falloff?: number;
}

export const cadFreeformPaintSmooth = (request: PaintSmoothRequest) =>
    invokeOrThrow<BrushOpResponse>('cad_freeform_paint_smooth', { request });

export interface PaintDrapeRequest {
    input: string;
    output: string;
    center: [number, number, number];
    radiusMm: number;
    directionMm: [number, number, number];
    strength?: number;
    falloff?: number;
}

export const cadFreeformPaintDrape = (request: PaintDrapeRequest) =>
    invokeOrThrow<BrushOpResponse>('cad_freeform_paint_drape', { request });

export interface EmergenceProfileRequest {
    marginPolyline: Array<[number, number, number]>;
    insertionAxis: [number, number, number];
    output: string;
    heightMm: number;
    topRadiusMm: number;
    axialSegments?: number;
}
export interface EmergenceProfileResponse {
    output: string;
    triangles: number;
    vertices: number;
    backend: string;
}

export const cadFreeformEmergenceProfile = (request: EmergenceProfileRequest) =>
    invokeOrThrow<EmergenceProfileResponse>('cad_freeform_emergence_profile', { request });

// ─── AR-V377 articulator ─────────────────────────────────────────────────

export interface BonwillTriangle {
    incisor: [number, number, number];
    condyleRight: [number, number, number];
    condyleLeft: [number, number, number];
}

export interface DefaultTriangleRequest {
    sideLengthMm?: number;
    balkwillAngleDeg?: number;
    curveOfSpeeRadiusMm?: number;
}
export interface DefaultTriangleResponse { triangle: BonwillTriangle; backend: string; }

export const cadArticulatorDefaultTriangle = (request: DefaultTriangleRequest = {}) =>
    invokeOrThrow<DefaultTriangleResponse>('cad_articulator_default_triangle', { request });

export interface RegisterRequest {
    incisor: [number, number, number];
    condyleRight: [number, number, number];
    condyleLeft: [number, number, number];
    sideLengthMm?: number;
}
export interface RegisterResponse {
    registration: {
        triangle: BonwillTriangle;
        rotationMatrix: number[][];
        translation: [number, number, number];
        fitErrorMm: number;
    };
    backend: string;
}

export const cadArticulatorRegister = (request: RegisterRequest) =>
    invokeOrThrow<RegisterResponse>('cad_articulator_register', { request });

export interface JawMotionState {
    openingDeg: number;
    protrusionMm: number;
    bennettAngleDeg: number;
    excursionSide: number;
    excursionMm: number;
}

export interface SimulateRequest {
    triangle: BonwillTriangle;
    state: JawMotionState;
    samplePath?: number;
}
export interface AffineTransform {
    rotationMatrix: number[][];
    translation: [number, number, number];
}
export interface SimulateResponse {
    transform: AffineTransform;
    path?: AffineTransform[];
    backend: string;
}

export const cadArticulatorSimulate = (request: SimulateRequest) =>
    invokeOrThrow<SimulateResponse>('cad_articulator_simulate', { request });

export type PlaneKind = 'occlusal' | 'frankfort' | 'camper' | 'least-squares';

export interface FitPlaneRequest {
    kind: PlaneKind;
    points: Array<[number, number, number]>;
}
export interface FitPlaneResponse {
    plane: { origin: [number, number, number]; normal: [number, number, number] };
    backend: string;
}

export const cadArticulatorFitPlane = (request: FitPlaneRequest) =>
    invokeOrThrow<FitPlaneResponse>('cad_articulator_fit_plane', { request });

// ─── AR-V378 endo ────────────────────────────────────────────────────────

export interface EndoChamberRequest {
    output: string;
    center: [number, number, number];
    axis: [number, number, number];
    diameterMm: number;
    depthMm: number;
    taperDeg?: number;
    radialSegments?: number;
    axialSegments?: number;
}
export interface EndoChamberResponse {
    output: string;
    triangles: number;
    vertices: number;
    volumeMm3: number;
    watertight: boolean;
    backend: string;
}

export const cadEndoChamberBuild = (request: EndoChamberRequest) =>
    invokeOrThrow<EndoChamberResponse>('cad_endo_chamber_build', { request });

export interface EndoCanalAxisRequest {
    points: Array<[number, number, number]>;
    occlusalDown: [number, number, number];
}
export interface EndoCanalAxis {
    origin: [number, number, number];
    axis: [number, number, number];
    lengthMm: number;
    linearity: number;
}
export interface EndoCanalAxisResponse { axis: EndoCanalAxis; backend: string; }

export const cadEndoEstimateCanalAxis = (request: EndoCanalAxisRequest) =>
    invokeOrThrow<EndoCanalAxisResponse>('cad_endo_estimate_canal_axis', { request });

// ─── MP-101 / MP-110 compute router ──────────────────────────────────────

export type ComputeKindId =
    | 'cpu-rayon'
    | 'cpu-simd'
    | 'gpu-wgpu'
    | 'gpu-cuda'
    | 'gpu-metal'
    | 'ane-coreml'
    | 'trt-tensorrt'
    | 'rocm-amd'
    | 'directml-npu';

export type EnergyMode = 'performance' | 'low-power';

export interface BenchResult {
    backend: ComputeKindId;
    op: string;
    elapsedMs: number;
    error: string | null;
}

export interface BenchProfile {
    generatedAt: string;
    hostId: string;
    energyMode: EnergyMode | null;
    ranking: Record<string, ComputeKindId[]>;
    results: BenchResult[];
}

export interface RunBenchRequest { energyMode?: EnergyMode; }
export interface RunBenchResponse {
    profile: BenchProfile;
    pickedForDistance: ComputeKindId;
    pickedForSmooth: ComputeKindId;
    backend: string;
}

export const cadComputeRunBench = (request: RunBenchRequest = {}) =>
    invokeOrThrow<RunBenchResponse>('cad_compute_run_bench', { request });

export interface ComputeStatusResponse {
    hostId: string;
    energyMode: EnergyMode;
    statusPerVertexDistance: string;
    statusLaplacianSmooth: string;
    backendsAvailable: ComputeKindId[];
    profilePersisted: boolean;
}

export const cadComputeStatus = () =>
    invokeOrThrow<ComputeStatusResponse>('cad_compute_status', {});

export interface SetEnergyModeRequest { mode: EnergyMode; }
export interface SetEnergyModeResponse { mode: EnergyMode; }

export const cadComputeSetEnergyMode = (request: SetEnergyModeRequest) =>
    invokeOrThrow<SetEnergyModeResponse>('cad_compute_set_energy_mode', { request });

export type BoundaryRecommendation =
    | 'keep-three-render-only'
    | 'rust-compute-three-render'
    | 'rust-wgpu-candidate';

export type BoundaryOwner =
    | 'react-ui'
    | 'three-renderer'
    | 'tauri-command'
    | 'rust-compute'
    | 'rust-wgpu-candidate';

export interface BoundaryBenchResult {
    op: string;
    elapsedMs: number;
    itemsProcessed: number;
    payloadBytes: number;
    owner: BoundaryOwner;
}

export interface BoundaryBenchProfile {
    generatedAt: string;
    hostId: string;
    energyMode: EnergyMode;
    sampleVertices: number;
    sampleTriangles: number;
    meshBufferBytes: number;
    jsonIpcBytes: number;
    transformIpcBytes: number;
    jsonSerialiseMs: number;
    cpuSmoothMs: number;
    cpuItemsPerMs: number;
    ipcPayloadRatio: number;
    frameBudgetMs: number;
    recommendedBoundary: BoundaryRecommendation;
    results: BoundaryBenchResult[];
    notes: string[];
}

export interface RunBoundaryBenchRequest {
    sampleVertices?: number;
    energyMode?: EnergyMode;
}

export interface RunBoundaryBenchResponse {
    profile: BoundaryBenchProfile;
    profilePath: string;
    backend: string;
}

export const cadComputeRunBoundaryBench = (request: RunBoundaryBenchRequest = {}) =>
    invokeOrThrow<RunBoundaryBenchResponse>('cad_compute_run_boundary_bench', { request });

// ─── AR-V376 show distances real (cierra audit no-stubs #10) ─────────────

export interface DistanceShaderOptionsDto {
    redThresholdMm: number;
    greenThresholdMm: number;
    flagInterpenetration?: boolean;
}

export interface ShowDistancesRequest {
    source: string;
    target: string;
    bucketCount?: number;
    includePerVertex?: boolean;
    options?: DistanceShaderOptionsDto;
}

export interface DistanceStats {
    minMm: number;
    maxMm: number;
    meanMm: number;
    medianMm: number;
    p5Mm: number;
    p95Mm: number;
    p99Mm: number;
    vertexCount: number;
}

export interface HistogramBucket { lowMm: number; highMm: number; count: number; }

export interface ShowDistancesResponse {
    stats: DistanceStats;
    histogram: HistogramBucket[];
    perVertexMm: number[] | null;
    perVertexSeverity: number[] | null;
    redThresholdMm: number;
    greenThresholdMm: number;
    backend: string;
}

export const cadShowDistancesReal = (request: ShowDistancesRequest) =>
    invokeOrThrow<ShowDistancesResponse>('cad_show_distances_real', { request });

// ─── AR-V373 implant manager ─────────────────────────────────────────────

export interface ImplantPlacement {
    fdi: number;
    sku: string;
    position: [number, number, number];
    axis: [number, number, number];
    attachedReconstructions?: string[];
}

export interface ImplantPlanningState {
    implants: ImplantPlacement[];
    referenceFdis?: number[];
    orphanedReconstructions?: string[];
}

export interface ManagerWarning {
    kind: string;
    severity: 'Info' | 'Warning' | 'Error';
    message: string;
}

export interface ChangeTypeRequest {
    state: ImplantPlanningState;
    fdi: number;
    newSku: string;
    invalidatesReconstructions?: boolean;
}
export interface ChangeTypeResponse {
    state: ImplantPlanningState;
    orphanedReconstructions: string[];
    backend: string;
}

export const cadImplantChangeType = (request: ChangeTypeRequest) =>
    invokeOrThrow<ChangeTypeResponse>('cad_implant_change_type', { request });

export interface ImplantDeleteRequest { state: ImplantPlanningState; fdi: number; }
export interface ImplantDeleteResponse {
    state: ImplantPlanningState;
    orphanedReconstructions: string[];
    backend: string;
}

export const cadImplantDelete = (request: ImplantDeleteRequest) =>
    invokeOrThrow<ImplantDeleteResponse>('cad_implant_delete', { request });

export interface DefineReferencesRequest { state: ImplantPlanningState; fdis: number[]; }
export interface DefineReferencesResponse {
    state: ImplantPlanningState;
    warnings: ManagerWarning[];
    backend: string;
}

export const cadImplantDefineReferences = (request: DefineReferencesRequest) =>
    invokeOrThrow<DefineReferencesResponse>('cad_implant_define_references', { request });

export interface ValidatePlacementRequest {
    state: ImplantPlanningState;
    proposal: ImplantPlacement;
    proposedRadiusMm?: number;
    existingRadiusMm?: number;
}
export interface ValidatePlacementResponse { warnings: ManagerWarning[]; backend: string; }

export const cadImplantValidatePlacement = (request: ValidatePlacementRequest) =>
    invokeOrThrow<ValidatePlacementResponse>('cad_implant_validate_placement', { request });

// ─── AR-V369 crown feedback ──────────────────────────────────────────────

export interface ConstraintBounds {
    minThicknessMm: number;
    maxUndercutMm: number;
    minOcclusalClearanceMm: number;
    minConnectorAreaMm2: number;
}

export interface CrownWarning {
    kind: string;
    severity: 'Ok' | 'Warning' | 'Error';
    message: string;
    vertexIndices?: number[];
}

export interface ToothFeedbackReport {
    fdi: number;
    material: string;
    bounds: ConstraintBounds;
    warnings: CrownWarning[];
    hasBlockingError: boolean;
    measuredMinThicknessMm: number;
    measuredMinClearanceMm: number;
}

export interface ValidateCrownRequest {
    fdi: number;
    material: string;
    crownOuter: string;
    crownBottom: string;
    antagonist?: string;
    overrides?: ConstraintBounds;
}

export interface ValidateCrownResponse { report: ToothFeedbackReport; backend: string; }

export const cadCrownValidateReal = (request: ValidateCrownRequest) =>
    invokeOrThrow<ValidateCrownResponse>('cad_crown_validate_real', { request });

export interface ConstraintBoundsRequest { material: string; }
export interface ConstraintBoundsResponse { bounds: ConstraintBounds; backend: string; }

export const cadCrownConstraintBounds = (request: ConstraintBoundsRequest) =>
    invokeOrThrow<ConstraintBoundsResponse>('cad_crown_constraint_bounds', { request });

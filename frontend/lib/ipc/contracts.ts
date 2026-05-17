/**
 * Catálogo central de contratos IPC.
 *
 * Una entrada por comando registrado en `apps/desktop/src-tauri/src/lib.rs`
 * (`tauri::generate_handler![…]`). Las firmas siguen este patrón:
 *
 *   export interface CmdNombreArgs { … }
 *   export type CmdNombreResult = …
 *
 * Para comandos sin parámetros: `Args = void`.
 *
 * Cuando el DTO Rust es complejo (estructuras grandes con muchos campos
 * opcionales, enums anidados, paths) y todavía no hay caller frontend, el
 * tipo del request o response queda como `JsonObject` / `JsonValue`. Esto
 * mantiene la cadena tipada (`unknown` no atraviesa la frontera Tauri sin
 * cast) sin obligar a copiar 1:1 cada `serde::Deserialize` del Rust. Los
 * services concretos pueden refinar localmente con un `as` o un nuevo
 * `interface` en su carpeta `infrastructure/`.
 *
 * Total comandos: 126. Marcados como `// orphan` los que el audit detectó
 * como registrados pero sin caller literal — están listos para usar
 * cuando un sprint AR-V### los descubra.
 */

/* eslint-disable @typescript-eslint/no-empty-object-type */

// ---------------------------------------------------------------------------
// Primitivos compartidos
// ---------------------------------------------------------------------------

export type JsonValue =
    | string
    | number
    | boolean
    | null
    | JsonValue[]
    | { [key: string]: JsonValue };

export type JsonObject = { [key: string]: JsonValue };

export type Vec3 = [number, number, number];

// ---------------------------------------------------------------------------
// Runtime / sistema
// ---------------------------------------------------------------------------

export type CmdGetRuntimeInfoArgs = void;
export interface CmdGetRuntimeInfoResult {
    appName: string;
    company: string;
    location: string;
    version: string;
    buildUid: string;
    profile: string;
}

export type CmdGetSystemRuntimeReportArgs = void;
export type CmdGetSystemRuntimeReportResult = JsonObject;

export type CmdGetPythonBridgeStatusArgs = void;
export type CmdGetPythonBridgeStatusResult = JsonObject;
export type CmdTrameSlicerSidecarStatusArgs = void;
export type CmdTrameSlicerSidecarStatusResult = JsonObject;
export type CmdTrameSlicerSidecarStartArgs = void;
export type CmdTrameSlicerSidecarStartResult = JsonObject;
export type CmdTrameSlicerSidecarStopArgs = void;
export type CmdTrameSlicerSidecarStopResult = JsonObject;
export type CmdSlicerRuntimeStatusArgs = void;
export type CmdSlicerRuntimeStatusResult = JsonObject;
export type CmdSlicerRuntimeDownloadArgs = void;
export type CmdSlicerRuntimeDownloadResult = JsonObject;
export type CmdSlicerModelsStatusArgs = void;
export type CmdSlicerModelsStatusResult = JsonObject;
export type CmdSlicerFixturesStatusArgs = void;
export type CmdSlicerFixturesStatusResult = JsonObject;
export interface CmdSlicerFixturesDownloadArgs {
    request: {
        fixtureId: string;
    };
}
export type CmdSlicerFixturesDownloadResult = JsonObject;
export interface CmdSlicerModelsDownloadAllArgs {
    request: {
        includeOptional: boolean;
    };
}
export type CmdSlicerModelsDownloadAllResult = JsonObject;
export interface CmdSlicerClinicalJobStartArgs {
    request: {
        caseId: string;
        workflowId: string;
        sourcePath: string;
        outputDir?: string | null;
        modelId?: string | null;
        options?: JsonObject;
    };
}
export type CmdSlicerClinicalJobStartResult = JsonObject;
export interface CmdSlicerClinicalJobStatusArgs {
    jobId: string;
}
export type CmdSlicerClinicalJobStatusResult = JsonObject;
export interface CmdSlicerClinicalJobCancelArgs {
    jobId: string;
}
export type CmdSlicerClinicalJobCancelResult = JsonObject;

export type CmdGetBackendIntegrationCatalogArgs = void;
export type CmdGetBackendIntegrationCatalogResult = JsonObject;

export type CmdInspectBackendWorkspaceArgs = void;
export type CmdInspectBackendWorkspaceResult = JsonObject;

export type CmdInspectBackendTopologyArgs = void;
export type CmdInspectBackendTopologyResult = JsonObject;

export type CmdRunBackendGeometryProbeArgs = void;
export type CmdRunBackendGeometryProbeResult = JsonObject;

export type CmdRunBackendManifoldCsgProbeArgs = void;
export type CmdRunBackendManifoldCsgProbeResult = JsonObject;

export interface CmdInspectDicomMetadataArgs {
    path: string;
}
export type CmdInspectDicomMetadataResult = JsonObject;

export interface CmdDicomImportPrepareFromPathArgs {
    request: JsonObject;
}
export type CmdDicomImportPrepareFromPathResult = JsonObject;

export interface CmdDicomSeriesImportStartArgs {
    request: JsonObject;
}
export type CmdDicomSeriesImportStartResult = JsonObject;

export interface CmdDicomSeriesJobStatusArgs {
    jobId: string;
}
export type CmdDicomSeriesJobStatusResult = JsonObject;

export interface CmdDicomSeriesImportCancelArgs {
    jobId: string;
}
export type CmdDicomSeriesImportCancelResult = JsonObject;

export interface CmdDicomVolumeBuildStartArgs {
    request: JsonObject;
}
export type CmdDicomVolumeBuildStartResult = JsonObject;

export interface CmdDicomVolumeJobStatusArgs {
    jobId: string;
}
export type CmdDicomVolumeJobStatusResult = JsonObject;

export interface CmdDicomSegmentationStartArgs {
    request: JsonObject;
}
export type CmdDicomSegmentationStartResult = JsonObject;

export interface CmdDicomSegmentationJobStatusArgs {
    jobId: string;
}
export type CmdDicomSegmentationJobStatusResult = JsonObject;

export interface CmdDicomSegmentationCancelArgs {
    jobId: string;
}
export type CmdDicomSegmentationCancelResult = JsonObject;

export interface CmdDicomSegmentationToMeshStartArgs {
    request: JsonObject;
}
export type CmdDicomSegmentationToMeshStartResult = JsonObject;

export interface CmdOpenWorkspaceWindowArgs {
    request: {
        label: string;
        workspace: string;
        caseId?: string;
        module?: string;
        title?: string;
    };
}
export type CmdOpenWorkspaceWindowResult = void;

// ---------------------------------------------------------------------------
// Toolkit codec
// ---------------------------------------------------------------------------

export interface CmdCompressTextLzmaArgs {
    payload: string;
}
export type CmdCompressTextLzmaResult = string;

export interface CmdDecompressTextLzmaArgs {
    payloadBase64: string;
}
export type CmdDecompressTextLzmaResult = string;

export interface CmdEncryptTextPayloadArgs {
    payload: string;
    passphrase: string;
}
export interface CmdEncryptTextPayloadResult {
    ciphertextBase64: string;
    nonceBase64: string;
}

export interface CmdDecryptTextPayloadArgs {
    ciphertextBase64: string;
    nonceBase64: string;
    passphrase: string;
}
export type CmdDecryptTextPayloadResult = string;

// ---------------------------------------------------------------------------
// Public asset manifest
// ---------------------------------------------------------------------------

export type CmdGetPublicAssetManifestArgs = JsonObject | void;
export type CmdGetPublicAssetManifestResult = JsonObject;

export interface CmdInspectPublicAssetArgs {
    relativePath: string;
}
export type CmdInspectPublicAssetResult = JsonObject;

// ---------------------------------------------------------------------------
// Case repository
// ---------------------------------------------------------------------------

export interface CmdCaseCreateArgs {
    request: JsonObject;
}
export type CmdCaseCreateResult = JsonObject;

export interface CmdCaseSaveArgs {
    dentalCase: JsonObject;
}
export type CmdCaseSaveResult = JsonObject;

export interface CmdCaseGetGraphArgs {
    caseId: string;
}
export type CmdCaseGetGraphResult = JsonObject | null;

export type CmdCaseListArgs = void;
export type CmdCaseListResult = JsonObject[];

export interface CmdCaseOpenArgs {
    caseId: string;
}
export type CmdCaseOpenResult = JsonObject;

export interface CmdCaseSaveAssetArgs {
    request: JsonObject;
}
export type CmdCaseSaveAssetResult = JsonObject;

export interface CmdAssetWriteArgs {
    caseId: string;
    relativePath: string;
    bytes: number[];
}
export type CmdAssetWriteResult = string;

export interface CmdAssetReadArgs {
    localPath: string;
}
export type CmdAssetReadResult = number[];

// ---------------------------------------------------------------------------
// Case storage
// ---------------------------------------------------------------------------

export type CmdResolvePathsArgs = void;
export type CmdResolvePathsResult = JsonObject;

export type CmdTlanticadDataRootGetArgs = void;
export type CmdTlanticadDataRootGetResult = JsonObject;

export interface CmdValidateCaseArgs {
    folder: string;
}
export type CmdValidateCaseResult = JsonObject;

export interface CmdCaseBlobEncryptArgs {
    request: JsonObject;
}
export type CmdCaseBlobEncryptResult = JsonObject;

export interface CmdCaseBlobDecryptArgs {
    request: JsonObject;
}
export type CmdCaseBlobDecryptResult = string;

export interface CmdAuditAppendArgs {
    request: JsonObject;
}
export type CmdAuditAppendResult = JsonObject;

// ---------------------------------------------------------------------------
// Clinical jobs / commands
// ---------------------------------------------------------------------------

export interface CmdClinicalJobRecordArgs {
    request: JsonObject;
}
export type CmdClinicalJobRecordResult = JsonObject;

export interface CmdClinicalJobGetArgs {
    jobId: string;
}
export type CmdClinicalJobGetResult = JsonObject;

export interface CmdClinicalJobListArgs {
    request?: JsonObject;
}
export type CmdClinicalJobListResult = JsonObject[];

export interface CmdClinicalJobCancelArgs {
    jobId: string;
}
export type CmdClinicalJobCancelResult = JsonObject;

export interface CmdClinicalArtifactRecordArgs {
    request: JsonObject;
}
export type CmdClinicalArtifactRecordResult = JsonObject;

export interface CmdClinicalCommandRecordArgs {
    request: JsonObject;
}
export type CmdClinicalCommandRecordResult = JsonObject;

export interface CmdClinicalCommandUndoArgs {
    request: JsonObject;
}
export type CmdClinicalCommandUndoResult = JsonObject;

export interface CmdClinicalCommandRedoArgs {
    request: JsonObject;
}
export type CmdClinicalCommandRedoResult = JsonObject;

export interface CmdClinicalCommandEventGetArgs {
    eventId: string;
}
export type CmdClinicalCommandEventGetResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD core
// ---------------------------------------------------------------------------

export interface CmdCadBootstrapArgs {
    request?: JsonObject;
}
export type CmdCadBootstrapResult = JsonObject;

export type CmdToolRegistryGetArgs = void;
export type CmdToolRegistryGetResult = JsonObject;

export interface CmdAssetImportPrepareArgs {
    request: JsonObject;
}
export type CmdAssetImportPrepareResult = JsonObject;

export interface CmdCadJobStartArgs {
    request: JsonObject;
}
export type CmdCadJobStartResult = JsonObject;

export interface CmdCadJobStatusArgs {
    jobId: string;
}
export type CmdCadJobStatusResult = JsonObject;

export interface CmdCadJobCancelArgs {
    jobId: string;
}
export type CmdCadJobCancelResult = JsonObject;

export interface CmdModulePermissionsGetArgs {
    request?: JsonObject;
}
export type CmdModulePermissionsGetResult = JsonObject;

// ---------------------------------------------------------------------------
// Mesh vault
// ---------------------------------------------------------------------------

export interface CmdMeshVaultImportStartArgs {
    request: JsonObject;
}
export type CmdMeshVaultImportStartResult = JsonObject;

export interface CmdMeshVaultJobStatusArgs {
    jobId: string;
}
export type CmdMeshVaultJobStatusResult = JsonObject;

export interface CmdMeshVaultCancelArgs {
    jobId: string;
}
export type CmdMeshVaultCancelResult = JsonObject;

export interface CmdMeshVaultFindArgs {
    request?: JsonObject;
}
export type CmdMeshVaultFindResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD shell
// ---------------------------------------------------------------------------

export interface CmdCadShellBootstrapArgs {
    request: JsonObject;
}
export type CmdCadShellBootstrapResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD abutment (legacy + real + production)
// ---------------------------------------------------------------------------

export interface CmdAbutmentGenerateMeshArgs {
    request: JsonObject;
}
export type CmdAbutmentGenerateMeshResult = JsonObject;

export interface CmdAbutmentPlanScrewChannelArgs {
    request: JsonObject;
}
export type CmdAbutmentPlanScrewChannelResult = JsonObject;

export interface CmdCadAbutmentGenerateRealArgs {
    request: JsonObject;
}
export type CmdCadAbutmentGenerateRealResult = JsonObject;

export interface CmdCadAbutmentValidateRealArgs {
    request: JsonObject;
}
export type CmdCadAbutmentValidateRealResult = JsonObject;

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadAbutmentProductionBlankArgs {
    request: {
        output: string;
        origin: Vec3;
        axis: Vec3;
        diameterMm: number;
        heightMm: number;
        taperDeg: number;
        radialSegments: number;
        axialSegments: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadAbutmentProductionBlankResult {
    output: string;
    triangles: number;
    vertices: number;
    volumeMm3: number;
    backend: string;
}

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadAbutmentScrewChannelArgs {
    request: {
        output: string;
        origin: Vec3;
        axis: Vec3;
        diameterMm: number;
        lengthMm: number;
        angleDeg: number;
        radialSegments: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadAbutmentScrewChannelResult = CmdCadAbutmentProductionBlankResult;

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadAbutmentNestingPuckArgs {
    request: {
        output: string;
        diameterMm: number;
        thicknessMm: number;
        slotCount: number;
        slotRadiusMm: number;
        slotDiameterMm: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadAbutmentNestingPuckResult {
    output: string;
    triangles: number;
    vertices: number;
    volumeMm3: number;
    slotPositions: Vec3[];
    backend: string;
}

// ---------------------------------------------------------------------------
// Abutment presets
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdAbutmentPresetSaveArgs {
    preset: JsonObject;
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetSaveResult = string;

// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetListArgs = void;
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetListResult = JsonObject[];

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdAbutmentPresetDeleteArgs {
    id: string;
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetDeleteResult = void;

// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetOpenFolderArgs = void;
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAbutmentPresetOpenFolderResult = string;

// ---------------------------------------------------------------------------
// CAD alignment
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdAlignmentRegisterLandmarksArgs {
    request: {
        movingPoints: Vec3[];
        fixedPoints: Vec3[];
        caseFolderPath?: string;
        outputFileName?: string;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdAlignmentRegisterLandmarksResult = JsonObject;

export interface CmdAlignmentRegisterMeshesArgs {
    request: JsonObject;
}
export type CmdAlignmentRegisterMeshesResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD articulator
// ---------------------------------------------------------------------------

export interface CmdCadArticulatorDefaultTriangleArgs {
    request?: JsonObject;
}
export type CmdCadArticulatorDefaultTriangleResult = JsonObject;

export interface CmdCadArticulatorRegisterArgs {
    request: JsonObject;
}
export type CmdCadArticulatorRegisterResult = JsonObject;

export interface CmdCadArticulatorSimulateArgs {
    request: JsonObject;
}
export type CmdCadArticulatorSimulateResult = JsonObject;

export interface CmdCadArticulatorFitPlaneArgs {
    request: JsonObject;
}
export type CmdCadArticulatorFitPlaneResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD bridge / connector
// ---------------------------------------------------------------------------

export interface CmdCadBridgeConnectorCreateArgs {
    request: JsonObject;
}
export type CmdCadBridgeConnectorCreateResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD compute router
// ---------------------------------------------------------------------------

export interface CmdCadComputeRunBenchArgs {
    request?: JsonObject;
}
export type CmdCadComputeRunBenchResult = JsonObject;

export interface CmdCadComputeRunBoundaryBenchArgs {
    request?: JsonObject;
}
export type CmdCadComputeRunBoundaryBenchResult = JsonObject;

export type CmdCadComputeStatusArgs = void;
export type CmdCadComputeStatusResult = JsonObject;

export interface CmdCadComputeSetEnergyModeArgs {
    request: JsonObject;
}
export type CmdCadComputeSetEnergyModeResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD crown
// ---------------------------------------------------------------------------

export interface CmdCadCrownBottomGenerateArgs {
    request: JsonObject;
}
export type CmdCadCrownBottomGenerateResult = JsonObject;

export interface CmdCadCrownValidateRealArgs {
    request: JsonObject;
}
export type CmdCadCrownValidateRealResult = JsonObject;

export interface CmdCadCrownConstraintBoundsArgs {
    request: JsonObject;
}
export type CmdCadCrownConstraintBoundsResult = JsonObject;

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadCrownPipelineRunArgs {
    request: {
        prepStl: string;
        libraryToothStl?: string;
        antagonistStl?: string;
        mesialNeighbourStl?: string;
        distalNeighbourStl?: string;
        output: string;
        params?: JsonObject;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadCrownPipelineRunResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD CSG / mesh kernel
// ---------------------------------------------------------------------------

export interface CmdMeshOpArgs {
    request: JsonObject;
}
export type CmdMeshOpResult = JsonObject;

export interface CmdMeshKernelAddRemoveArgs {
    request: JsonObject;
}
export type CmdMeshKernelAddRemoveResult = JsonObject;

export interface CmdMeshKernelCompareArgs {
    request: JsonObject;
}
export type CmdMeshKernelCompareResult = JsonObject;

export interface CmdMeshKernelAdaptToGingivaArgs {
    request: JsonObject;
}
export type CmdMeshKernelAdaptToGingivaResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD DICOM seg + guide
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadDicomThresholdVoxelsArgs {
    request: {
        volume: number[];
        low: number;
        high: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadDicomThresholdVoxelsResult {
    mask: number[];
    voxelCount: number;
    backend: string;
}

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadDicomRegionGrow3dArgs {
    request: {
        sizeX: number;
        sizeY: number;
        sizeZ: number;
        mask: number[];
        seed: [number, number, number];
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadDicomRegionGrow3dResult {
    visited: number[];
    report: JsonObject;
    backend: string;
}

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadDicomMarchingCubesArgs {
    request: {
        sizeX: number;
        sizeY: number;
        sizeZ: number;
        mask: number[];
        voxelSizeMm: number;
        output: string;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadDicomMarchingCubesResult = JsonObject;

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadGuideExtractGingivaArgs {
    request: {
        baseStl: string;
        output: string;
        intoTissueAxis: Vec3;
        normalDotThreshold: number;
        minComponentFaces: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadGuideExtractGingivaResult = JsonObject;

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadGuideBuildSleeveArgs {
    request: {
        output: string;
        center: Vec3;
        axis: Vec3;
        diameterMm: number;
        lengthMm: number;
        radialSegments: number;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadGuideBuildSleeveResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD endo
// ---------------------------------------------------------------------------

export interface CmdCadEndoChamberBuildArgs {
    request: JsonObject;
}
export type CmdCadEndoChamberBuildResult = JsonObject;

export interface CmdCadEndoEstimateCanalAxisArgs {
    request: JsonObject;
}
export type CmdCadEndoEstimateCanalAxisResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD freeform brush
// ---------------------------------------------------------------------------

export interface CmdCadFreeformPaintPullArgs {
    request: JsonObject;
}
export type CmdCadFreeformPaintPullResult = JsonObject;

export interface CmdCadFreeformPaintSmoothArgs {
    request: JsonObject;
}
export type CmdCadFreeformPaintSmoothResult = JsonObject;

export interface CmdCadFreeformPaintDrapeArgs {
    request: JsonObject;
}
export type CmdCadFreeformPaintDrapeResult = JsonObject;

export interface CmdCadFreeformEmergenceProfileArgs {
    request: JsonObject;
}
export type CmdCadFreeformEmergenceProfileResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD freeform specialty (orphans)
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadFreeformBarCreateArgs {
    request: {
        anchors: Vec3[];
        profile: string;
        widthMm: number;
        heightMm: number;
        occlusalUp: Vec3;
        radialSegments: number;
        output: string;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadFreeformBarCreateResult {
    output: string;
    triangles: number;
    vertices: number;
    volumeMm3: number;
    watertightHint: boolean;
    backend: string;
}

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadFreeformTelescopeCreateArgs {
    request: {
        base: Vec3;
        occlusalAxis: Vec3;
        primaryHeightMm: number;
        primaryRadiusMm: number;
        primaryTaperDeg: number;
        gapMm: number;
        secondaryThicknessMm: number;
        radialSegments: number;
        outputPrimary: string;
        outputSecondary: string;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadFreeformTelescopeCreateResult {
    primary: CmdCadFreeformBarCreateResult;
    secondary: CmdCadFreeformBarCreateResult;
}

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCadFreeformPostAndCoreCreateArgs {
    request: {
        canalEntrance: Vec3;
        canalAxis: Vec3;
        postLengthMm: number;
        coreHeightMm: number;
        postDiameterMm: number;
        coreDiameterMm: number;
        postTaperDeg: number;
        coreTaperDeg: number;
        radialSegments: number;
        output: string;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCadFreeformPostAndCoreCreateResult = CmdCadFreeformBarCreateResult;

// ---------------------------------------------------------------------------
// CAD implant manager
// ---------------------------------------------------------------------------

export interface CmdCadImplantChangeTypeArgs {
    request: JsonObject;
}
export type CmdCadImplantChangeTypeResult = JsonObject;

export interface CmdCadImplantDeleteArgs {
    request: JsonObject;
}
export type CmdCadImplantDeleteResult = JsonObject;

export interface CmdCadImplantDefineReferencesArgs {
    request: JsonObject;
}
export type CmdCadImplantDefineReferencesResult = JsonObject;

export interface CmdCadImplantValidatePlacementArgs {
    request: JsonObject;
}
export type CmdCadImplantValidatePlacementResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD insertion axis
// ---------------------------------------------------------------------------

export interface CmdCadInsertionComputeArgs {
    request: JsonObject;
}
export type CmdCadInsertionComputeResult = JsonObject;

export interface CmdCadInsertionUnifyBridgeArgs {
    request: JsonObject;
}
export type CmdCadInsertionUnifyBridgeResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD margin
// ---------------------------------------------------------------------------

export interface CmdCadMarginDetectRealArgs {
    request: JsonObject;
}
export type CmdCadMarginDetectRealResult = JsonObject;

export interface CmdCadMarginCorrectRealArgs {
    request: JsonObject;
}
export type CmdCadMarginCorrectRealResult = JsonObject;

export interface CmdCadMarginRepairRealArgs {
    request: JsonObject;
}
export type CmdCadMarginRepairRealResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD show distances
// ---------------------------------------------------------------------------

export interface CmdCadShowDistancesRealArgs {
    request: JsonObject;
}
export type CmdCadShowDistancesRealResult = JsonObject;

// ---------------------------------------------------------------------------
// CAD parameters store
// ---------------------------------------------------------------------------

export interface CmdParametersLoadArgs {
    request: JsonObject;
}
export type CmdParametersLoadResult = JsonObject;

export interface CmdParametersSaveArgs {
    request: JsonObject;
}
export type CmdParametersSaveResult = JsonObject;

export interface CmdParametersResetArgs {
    request: JsonObject;
}
export type CmdParametersResetResult = JsonObject;

export type CmdParametersListScopesArgs = void;
export type CmdParametersListScopesResult = JsonObject[];

export interface CmdParametersExportJsonArgs {
    request: JsonObject;
}
export type CmdParametersExportJsonResult = JsonObject;

// ---------------------------------------------------------------------------
// Case watcher (orphans)
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCaseWatcherStartArgs {
    request: {
        directory: string;
        debounceMs?: number;
        followSymlinks?: boolean;
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCaseWatcherStartResult = JsonObject;

// orphan: no caller yet — reserved for AR-V### sprint
export type CmdCaseWatcherStopArgs = void;
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdCaseWatcherStopResult {
    stopped: boolean;
}

// ---------------------------------------------------------------------------
// Dental model segmentation
// ---------------------------------------------------------------------------

export type CmdGetDentalModelSegStatusArgs = void;
export type CmdGetDentalModelSegStatusResult = JsonObject;

export interface CmdRunDentalModelSegmentationArgs {
    request: JsonObject;
}
export type CmdRunDentalModelSegmentationResult = JsonObject;

export type CmdStopDentalModelSegSidecarArgs = void;
export type CmdStopDentalModelSegSidecarResult = JsonObject;

// ---------------------------------------------------------------------------
// Local share (P2P)
// ---------------------------------------------------------------------------

export interface CmdLocalShareAdvertiseArgs {
    request: JsonObject;
}
export type CmdLocalShareAdvertiseResult = JsonObject;

export type CmdLocalShareStopAdvertisingArgs = void;
export type CmdLocalShareStopAdvertisingResult = JsonObject;

export interface CmdLocalShareBrowseStartArgs {
    request?: JsonObject;
}
export type CmdLocalShareBrowseStartResult = JsonObject;

export type CmdLocalShareBrowseStopArgs = void;
export type CmdLocalShareBrowseStopResult = JsonObject;

export interface CmdLocalShareListenArgs {
    request: JsonObject;
}
export type CmdLocalShareListenResult = JsonObject;

export type CmdLocalShareStopListeningArgs = void;
export type CmdLocalShareStopListeningResult = JsonObject;

export interface CmdLocalShareSendArgs {
    request: JsonObject;
}
export type CmdLocalShareSendResult = JsonObject;

export interface CmdLocalShareAcceptArgs {
    request: JsonObject;
}
export type CmdLocalShareAcceptResult = JsonObject;

export interface CmdLocalShareRejectArgs {
    request: JsonObject;
}
export type CmdLocalShareRejectResult = JsonObject;

export type CmdLocalShareGeneratePinArgs = void;
export type CmdLocalShareGeneratePinResult = string;

export type CmdLocalShareLocalTransportsArgs = void;
export type CmdLocalShareLocalTransportsResult = string[];

// ---------------------------------------------------------------------------
// Mesh rasterizer (orphan)
// ---------------------------------------------------------------------------

// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdMeshRasterizeArgs {
    req: {
        meshPath: string;
        outputDir: string;
        width: number;
        height: number;
        viewProj: number[];
    };
}
// orphan: no caller yet — reserved for AR-V### sprint
export interface CmdMeshRasterizeResult {
    idMapPath: string;
    normalMapPath: string;
    depthMapPath: string;
    width: number;
    height: number;
}

// ---------------------------------------------------------------------------
// Tabla maestra: nombre Rust → tupla [Args, Result].
// Útil para herramientas que iteran el catálogo (codegen, audits) sin
// volver a parsear lib.rs. Los tipos quedan inferibles vía indexación:
//
//   type Args = IpcCatalog["get_runtime_info"]["args"]
//   type Result = IpcCatalog["get_runtime_info"]["result"]
// ---------------------------------------------------------------------------

export interface IpcCatalog {
    abutment_generate_mesh: { args: CmdAbutmentGenerateMeshArgs; result: CmdAbutmentGenerateMeshResult };
    abutment_plan_screw_channel: {
        args: CmdAbutmentPlanScrewChannelArgs;
        result: CmdAbutmentPlanScrewChannelResult;
    };
    abutment_preset_delete: { args: CmdAbutmentPresetDeleteArgs; result: CmdAbutmentPresetDeleteResult };
    abutment_preset_list: { args: CmdAbutmentPresetListArgs; result: CmdAbutmentPresetListResult };
    abutment_preset_open_folder: {
        args: CmdAbutmentPresetOpenFolderArgs;
        result: CmdAbutmentPresetOpenFolderResult;
    };
    abutment_preset_save: { args: CmdAbutmentPresetSaveArgs; result: CmdAbutmentPresetSaveResult };
    alignment_register_landmarks: {
        args: CmdAlignmentRegisterLandmarksArgs;
        result: CmdAlignmentRegisterLandmarksResult;
    };
    alignment_register_meshes: {
        args: CmdAlignmentRegisterMeshesArgs;
        result: CmdAlignmentRegisterMeshesResult;
    };
    asset_import_prepare: { args: CmdAssetImportPrepareArgs; result: CmdAssetImportPrepareResult };
    asset_read: { args: CmdAssetReadArgs; result: CmdAssetReadResult };
    asset_write: { args: CmdAssetWriteArgs; result: CmdAssetWriteResult };
    audit_append: { args: CmdAuditAppendArgs; result: CmdAuditAppendResult };
    cad_abutment_generate_real: {
        args: CmdCadAbutmentGenerateRealArgs;
        result: CmdCadAbutmentGenerateRealResult;
    };
    cad_abutment_nesting_puck: {
        args: CmdCadAbutmentNestingPuckArgs;
        result: CmdCadAbutmentNestingPuckResult;
    };
    cad_abutment_production_blank: {
        args: CmdCadAbutmentProductionBlankArgs;
        result: CmdCadAbutmentProductionBlankResult;
    };
    cad_abutment_screw_channel: {
        args: CmdCadAbutmentScrewChannelArgs;
        result: CmdCadAbutmentScrewChannelResult;
    };
    cad_abutment_validate_real: {
        args: CmdCadAbutmentValidateRealArgs;
        result: CmdCadAbutmentValidateRealResult;
    };
    cad_articulator_default_triangle: {
        args: CmdCadArticulatorDefaultTriangleArgs;
        result: CmdCadArticulatorDefaultTriangleResult;
    };
    cad_articulator_fit_plane: {
        args: CmdCadArticulatorFitPlaneArgs;
        result: CmdCadArticulatorFitPlaneResult;
    };
    cad_articulator_register: {
        args: CmdCadArticulatorRegisterArgs;
        result: CmdCadArticulatorRegisterResult;
    };
    cad_articulator_simulate: {
        args: CmdCadArticulatorSimulateArgs;
        result: CmdCadArticulatorSimulateResult;
    };
    cad_bootstrap: { args: CmdCadBootstrapArgs; result: CmdCadBootstrapResult };
    cad_bridge_connector_create: {
        args: CmdCadBridgeConnectorCreateArgs;
        result: CmdCadBridgeConnectorCreateResult;
    };
    cad_compute_run_bench: { args: CmdCadComputeRunBenchArgs; result: CmdCadComputeRunBenchResult };
    cad_compute_run_boundary_bench: {
        args: CmdCadComputeRunBoundaryBenchArgs;
        result: CmdCadComputeRunBoundaryBenchResult;
    };
    cad_compute_set_energy_mode: {
        args: CmdCadComputeSetEnergyModeArgs;
        result: CmdCadComputeSetEnergyModeResult;
    };
    cad_compute_status: { args: CmdCadComputeStatusArgs; result: CmdCadComputeStatusResult };
    cad_crown_bottom_generate: {
        args: CmdCadCrownBottomGenerateArgs;
        result: CmdCadCrownBottomGenerateResult;
    };
    cad_crown_constraint_bounds: {
        args: CmdCadCrownConstraintBoundsArgs;
        result: CmdCadCrownConstraintBoundsResult;
    };
    cad_crown_pipeline_run: { args: CmdCadCrownPipelineRunArgs; result: CmdCadCrownPipelineRunResult };
    cad_crown_validate_real: {
        args: CmdCadCrownValidateRealArgs;
        result: CmdCadCrownValidateRealResult;
    };
    cad_dicom_marching_cubes: {
        args: CmdCadDicomMarchingCubesArgs;
        result: CmdCadDicomMarchingCubesResult;
    };
    cad_dicom_region_grow_3d: {
        args: CmdCadDicomRegionGrow3dArgs;
        result: CmdCadDicomRegionGrow3dResult;
    };
    cad_dicom_threshold_voxels: {
        args: CmdCadDicomThresholdVoxelsArgs;
        result: CmdCadDicomThresholdVoxelsResult;
    };
    cad_endo_chamber_build: { args: CmdCadEndoChamberBuildArgs; result: CmdCadEndoChamberBuildResult };
    cad_endo_estimate_canal_axis: {
        args: CmdCadEndoEstimateCanalAxisArgs;
        result: CmdCadEndoEstimateCanalAxisResult;
    };
    cad_freeform_bar_create: {
        args: CmdCadFreeformBarCreateArgs;
        result: CmdCadFreeformBarCreateResult;
    };
    cad_freeform_emergence_profile: {
        args: CmdCadFreeformEmergenceProfileArgs;
        result: CmdCadFreeformEmergenceProfileResult;
    };
    cad_freeform_paint_drape: {
        args: CmdCadFreeformPaintDrapeArgs;
        result: CmdCadFreeformPaintDrapeResult;
    };
    cad_freeform_paint_pull: {
        args: CmdCadFreeformPaintPullArgs;
        result: CmdCadFreeformPaintPullResult;
    };
    cad_freeform_paint_smooth: {
        args: CmdCadFreeformPaintSmoothArgs;
        result: CmdCadFreeformPaintSmoothResult;
    };
    cad_freeform_post_and_core_create: {
        args: CmdCadFreeformPostAndCoreCreateArgs;
        result: CmdCadFreeformPostAndCoreCreateResult;
    };
    cad_freeform_telescope_create: {
        args: CmdCadFreeformTelescopeCreateArgs;
        result: CmdCadFreeformTelescopeCreateResult;
    };
    cad_guide_build_sleeve: { args: CmdCadGuideBuildSleeveArgs; result: CmdCadGuideBuildSleeveResult };
    cad_guide_extract_gingiva: {
        args: CmdCadGuideExtractGingivaArgs;
        result: CmdCadGuideExtractGingivaResult;
    };
    cad_implant_change_type: {
        args: CmdCadImplantChangeTypeArgs;
        result: CmdCadImplantChangeTypeResult;
    };
    cad_implant_define_references: {
        args: CmdCadImplantDefineReferencesArgs;
        result: CmdCadImplantDefineReferencesResult;
    };
    cad_implant_delete: { args: CmdCadImplantDeleteArgs; result: CmdCadImplantDeleteResult };
    cad_implant_validate_placement: {
        args: CmdCadImplantValidatePlacementArgs;
        result: CmdCadImplantValidatePlacementResult;
    };
    cad_insertion_compute: { args: CmdCadInsertionComputeArgs; result: CmdCadInsertionComputeResult };
    cad_insertion_unify_bridge: {
        args: CmdCadInsertionUnifyBridgeArgs;
        result: CmdCadInsertionUnifyBridgeResult;
    };
    cad_job_cancel: { args: CmdCadJobCancelArgs; result: CmdCadJobCancelResult };
    cad_job_start: { args: CmdCadJobStartArgs; result: CmdCadJobStartResult };
    cad_job_status: { args: CmdCadJobStatusArgs; result: CmdCadJobStatusResult };
    cad_margin_correct_real: {
        args: CmdCadMarginCorrectRealArgs;
        result: CmdCadMarginCorrectRealResult;
    };
    cad_margin_detect_real: { args: CmdCadMarginDetectRealArgs; result: CmdCadMarginDetectRealResult };
    cad_margin_repair_real: { args: CmdCadMarginRepairRealArgs; result: CmdCadMarginRepairRealResult };
    cad_shell_bootstrap: { args: CmdCadShellBootstrapArgs; result: CmdCadShellBootstrapResult };
    cad_show_distances_real: {
        args: CmdCadShowDistancesRealArgs;
        result: CmdCadShowDistancesRealResult;
    };
    case_blob_decrypt: { args: CmdCaseBlobDecryptArgs; result: CmdCaseBlobDecryptResult };
    case_blob_encrypt: { args: CmdCaseBlobEncryptArgs; result: CmdCaseBlobEncryptResult };
    case_create: { args: CmdCaseCreateArgs; result: CmdCaseCreateResult };
    case_get_graph: { args: CmdCaseGetGraphArgs; result: CmdCaseGetGraphResult };
    case_list: { args: CmdCaseListArgs; result: CmdCaseListResult };
    case_open: { args: CmdCaseOpenArgs; result: CmdCaseOpenResult };
    case_save: { args: CmdCaseSaveArgs; result: CmdCaseSaveResult };
    case_save_asset: { args: CmdCaseSaveAssetArgs; result: CmdCaseSaveAssetResult };
    case_watcher_start: { args: CmdCaseWatcherStartArgs; result: CmdCaseWatcherStartResult };
    case_watcher_stop: { args: CmdCaseWatcherStopArgs; result: CmdCaseWatcherStopResult };
    clinical_artifact_record: {
        args: CmdClinicalArtifactRecordArgs;
        result: CmdClinicalArtifactRecordResult;
    };
    clinical_command_event_get: {
        args: CmdClinicalCommandEventGetArgs;
        result: CmdClinicalCommandEventGetResult;
    };
    clinical_command_record: {
        args: CmdClinicalCommandRecordArgs;
        result: CmdClinicalCommandRecordResult;
    };
    clinical_command_redo: { args: CmdClinicalCommandRedoArgs; result: CmdClinicalCommandRedoResult };
    clinical_command_undo: { args: CmdClinicalCommandUndoArgs; result: CmdClinicalCommandUndoResult };
    clinical_job_cancel: { args: CmdClinicalJobCancelArgs; result: CmdClinicalJobCancelResult };
    clinical_job_get: { args: CmdClinicalJobGetArgs; result: CmdClinicalJobGetResult };
    clinical_job_list: { args: CmdClinicalJobListArgs; result: CmdClinicalJobListResult };
    clinical_job_record: { args: CmdClinicalJobRecordArgs; result: CmdClinicalJobRecordResult };
    compress_text_lzma: { args: CmdCompressTextLzmaArgs; result: CmdCompressTextLzmaResult };
    decompress_text_lzma: { args: CmdDecompressTextLzmaArgs; result: CmdDecompressTextLzmaResult };
    decrypt_text_payload: { args: CmdDecryptTextPayloadArgs; result: CmdDecryptTextPayloadResult };
    dicom_import_prepare_from_path: {
        args: CmdDicomImportPrepareFromPathArgs;
        result: CmdDicomImportPrepareFromPathResult;
    };
    dicom_series_import_cancel: {
        args: CmdDicomSeriesImportCancelArgs;
        result: CmdDicomSeriesImportCancelResult;
    };
    dicom_series_import_start: {
        args: CmdDicomSeriesImportStartArgs;
        result: CmdDicomSeriesImportStartResult;
    };
    dicom_series_job_status: {
        args: CmdDicomSeriesJobStatusArgs;
        result: CmdDicomSeriesJobStatusResult;
    };
    dicom_volume_build_start: {
        args: CmdDicomVolumeBuildStartArgs;
        result: CmdDicomVolumeBuildStartResult;
    };
    dicom_volume_job_status: {
        args: CmdDicomVolumeJobStatusArgs;
        result: CmdDicomVolumeJobStatusResult;
    };
    dicom_segmentation_start: {
        args: CmdDicomSegmentationStartArgs;
        result: CmdDicomSegmentationStartResult;
    };
    dicom_segmentation_job_status: {
        args: CmdDicomSegmentationJobStatusArgs;
        result: CmdDicomSegmentationJobStatusResult;
    };
    dicom_segmentation_cancel: {
        args: CmdDicomSegmentationCancelArgs;
        result: CmdDicomSegmentationCancelResult;
    };
    dicom_segmentation_to_mesh_start: {
        args: CmdDicomSegmentationToMeshStartArgs;
        result: CmdDicomSegmentationToMeshStartResult;
    };
    encrypt_text_payload: { args: CmdEncryptTextPayloadArgs; result: CmdEncryptTextPayloadResult };
    get_backend_integration_catalog: {
        args: CmdGetBackendIntegrationCatalogArgs;
        result: CmdGetBackendIntegrationCatalogResult;
    };
    get_dental_model_seg_status: {
        args: CmdGetDentalModelSegStatusArgs;
        result: CmdGetDentalModelSegStatusResult;
    };
    get_public_asset_manifest: {
        args: CmdGetPublicAssetManifestArgs;
        result: CmdGetPublicAssetManifestResult;
    };
    get_python_bridge_status: {
        args: CmdGetPythonBridgeStatusArgs;
        result: CmdGetPythonBridgeStatusResult;
    };
    trame_slicer_sidecar_status: {
        args: CmdTrameSlicerSidecarStatusArgs;
        result: CmdTrameSlicerSidecarStatusResult;
    };
    trame_slicer_sidecar_start: {
        args: CmdTrameSlicerSidecarStartArgs;
        result: CmdTrameSlicerSidecarStartResult;
    };
    trame_slicer_sidecar_stop: {
        args: CmdTrameSlicerSidecarStopArgs;
        result: CmdTrameSlicerSidecarStopResult;
    };
    slicer_runtime_status: {
        args: CmdSlicerRuntimeStatusArgs;
        result: CmdSlicerRuntimeStatusResult;
    };
    slicer_runtime_download: {
        args: CmdSlicerRuntimeDownloadArgs;
        result: CmdSlicerRuntimeDownloadResult;
    };
    slicer_models_status: {
        args: CmdSlicerModelsStatusArgs;
        result: CmdSlicerModelsStatusResult;
    };
    slicer_fixtures_status: {
        args: CmdSlicerFixturesStatusArgs;
        result: CmdSlicerFixturesStatusResult;
    };
    slicer_fixtures_download: {
        args: CmdSlicerFixturesDownloadArgs;
        result: CmdSlicerFixturesDownloadResult;
    };
    slicer_models_download_all: {
        args: CmdSlicerModelsDownloadAllArgs;
        result: CmdSlicerModelsDownloadAllResult;
    };
    slicer_clinical_job_start: {
        args: CmdSlicerClinicalJobStartArgs;
        result: CmdSlicerClinicalJobStartResult;
    };
    slicer_clinical_job_status: {
        args: CmdSlicerClinicalJobStatusArgs;
        result: CmdSlicerClinicalJobStatusResult;
    };
    slicer_clinical_job_cancel: {
        args: CmdSlicerClinicalJobCancelArgs;
        result: CmdSlicerClinicalJobCancelResult;
    };
    get_runtime_info: { args: CmdGetRuntimeInfoArgs; result: CmdGetRuntimeInfoResult };
    get_system_runtime_report: {
        args: CmdGetSystemRuntimeReportArgs;
        result: CmdGetSystemRuntimeReportResult;
    };
    inspect_backend_workspace: {
        args: CmdInspectBackendWorkspaceArgs;
        result: CmdInspectBackendWorkspaceResult;
    };
    inspect_backend_topology: {
        args: CmdInspectBackendTopologyArgs;
        result: CmdInspectBackendTopologyResult;
    };
    inspect_dicom_metadata: { args: CmdInspectDicomMetadataArgs; result: CmdInspectDicomMetadataResult };
    inspect_public_asset: { args: CmdInspectPublicAssetArgs; result: CmdInspectPublicAssetResult };
    local_share_accept: { args: CmdLocalShareAcceptArgs; result: CmdLocalShareAcceptResult };
    local_share_advertise: { args: CmdLocalShareAdvertiseArgs; result: CmdLocalShareAdvertiseResult };
    local_share_browse_start: {
        args: CmdLocalShareBrowseStartArgs;
        result: CmdLocalShareBrowseStartResult;
    };
    local_share_browse_stop: {
        args: CmdLocalShareBrowseStopArgs;
        result: CmdLocalShareBrowseStopResult;
    };
    local_share_generate_pin: {
        args: CmdLocalShareGeneratePinArgs;
        result: CmdLocalShareGeneratePinResult;
    };
    local_share_listen: { args: CmdLocalShareListenArgs; result: CmdLocalShareListenResult };
    local_share_local_transports: {
        args: CmdLocalShareLocalTransportsArgs;
        result: CmdLocalShareLocalTransportsResult;
    };
    local_share_reject: { args: CmdLocalShareRejectArgs; result: CmdLocalShareRejectResult };
    local_share_send: { args: CmdLocalShareSendArgs; result: CmdLocalShareSendResult };
    local_share_stop_advertising: {
        args: CmdLocalShareStopAdvertisingArgs;
        result: CmdLocalShareStopAdvertisingResult;
    };
    local_share_stop_listening: {
        args: CmdLocalShareStopListeningArgs;
        result: CmdLocalShareStopListeningResult;
    };
    mesh_kernel_adapt_to_gingiva: {
        args: CmdMeshKernelAdaptToGingivaArgs;
        result: CmdMeshKernelAdaptToGingivaResult;
    };
    mesh_kernel_add_remove: { args: CmdMeshKernelAddRemoveArgs; result: CmdMeshKernelAddRemoveResult };
    mesh_kernel_compare: { args: CmdMeshKernelCompareArgs; result: CmdMeshKernelCompareResult };
    mesh_op: { args: CmdMeshOpArgs; result: CmdMeshOpResult };
    mesh_rasterize: { args: CmdMeshRasterizeArgs; result: CmdMeshRasterizeResult };
    mesh_vault_cancel: { args: CmdMeshVaultCancelArgs; result: CmdMeshVaultCancelResult };
    mesh_vault_find: { args: CmdMeshVaultFindArgs; result: CmdMeshVaultFindResult };
    mesh_vault_import_start: {
        args: CmdMeshVaultImportStartArgs;
        result: CmdMeshVaultImportStartResult;
    };
    mesh_vault_job_status: { args: CmdMeshVaultJobStatusArgs; result: CmdMeshVaultJobStatusResult };
    module_permissions_get: { args: CmdModulePermissionsGetArgs; result: CmdModulePermissionsGetResult };
    open_workspace_window: { args: CmdOpenWorkspaceWindowArgs; result: CmdOpenWorkspaceWindowResult };
    parameters_export_json: { args: CmdParametersExportJsonArgs; result: CmdParametersExportJsonResult };
    parameters_list_scopes: { args: CmdParametersListScopesArgs; result: CmdParametersListScopesResult };
    parameters_load: { args: CmdParametersLoadArgs; result: CmdParametersLoadResult };
    parameters_reset: { args: CmdParametersResetArgs; result: CmdParametersResetResult };
    parameters_save: { args: CmdParametersSaveArgs; result: CmdParametersSaveResult };
    resolve_paths: { args: CmdResolvePathsArgs; result: CmdResolvePathsResult };
    run_backend_geometry_probe: {
        args: CmdRunBackendGeometryProbeArgs;
        result: CmdRunBackendGeometryProbeResult;
    };
    run_backend_manifold_csg_probe: {
        args: CmdRunBackendManifoldCsgProbeArgs;
        result: CmdRunBackendManifoldCsgProbeResult;
    };
    run_dental_model_segmentation: {
        args: CmdRunDentalModelSegmentationArgs;
        result: CmdRunDentalModelSegmentationResult;
    };
    stop_dental_model_seg_sidecar: {
        args: CmdStopDentalModelSegSidecarArgs;
        result: CmdStopDentalModelSegSidecarResult;
    };
    tlanticad_data_root_get: {
        args: CmdTlanticadDataRootGetArgs;
        result: CmdTlanticadDataRootGetResult;
    };
    tool_registry_get: { args: CmdToolRegistryGetArgs; result: CmdToolRegistryGetResult };
    validate_case: { args: CmdValidateCaseArgs; result: CmdValidateCaseResult };
}

export type IpcCommandName = keyof IpcCatalog;

export const IPC_COMMANDS = {
    meshVaultCancel: "mesh_vault_cancel",
    meshVaultFind: "mesh_vault_find",
    meshVaultImportStart: "mesh_vault_import_start",
    meshVaultJobStatus: "mesh_vault_job_status",
    slicerFixturesStatus: "slicer_fixtures_status",
    slicerFixturesDownload: "slicer_fixtures_download",
} as const satisfies Record<string, IpcCommandName>;

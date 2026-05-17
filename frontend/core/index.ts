export type {
  ClinicalAsset,
  DentalCase,
  FileActionDefinition,
  FileActionId,
  ModuleWorkflowState,
  TlantiModuleDefinition,
  TlantiModuleId,
  TlantiModuleStage,
  ToothNumber,
} from './domain/entities'
export {
  FILE_ACTION_DEFINITIONS,
  TLANTI_MODULE_DEFINITIONS,
  TLANTI_MODULE_IDS,
  isFileActionId,
  isTlantiModuleId,
  listTlantiModules,
  resolveTlantiModuleDefinition,
} from './domain/module-registry'
export type {
  TlantiCadProductModuleDefinition,
  TlantiCadProductModuleId,
} from './domain/cad-product-module-registry'
export {
  CAD_PRODUCT_MODULE_DEFINITIONS,
  TLANTI_CAD_PRODUCT_MODULE_IDS,
  isTlantiCadProductModuleId,
  listCadProductModules,
  resolveCadProductModuleDefinition,
  resolveCadProductModuleForRoute,
} from './domain/cad-product-module-registry'
export type {
  TlantiCadCompetitorSignal,
  TlantiCadModuleRoadmapDefinition,
  TlantiCadModuleWorkflowPhase,
  TlantiCadWorkflowOwner,
} from './domain/cad-module-roadmap'
export {
  CAD_MODULE_ROADMAP_DEFINITIONS,
  TLANTI_CAD_WORKFLOW_OWNERS,
  listCadModuleRoadmaps,
  resolveCadModuleRoadmap,
} from './domain/cad-module-roadmap'
export type {
  ModuleSurfaceDefinition,
  ModuleSurfaceGroup,
  ModuleSurfaceId,
  ModuleSurfaceOwner,
  ModuleSurfaceRegistry,
} from './domain/module-surface-registry'
export {
  MODULE_SURFACE_DEFINITIONS,
  MODULE_SURFACE_IDS,
  MODULE_SURFACE_REGISTRY,
  isModuleSurfaceId,
  listModuleSurfaces,
  listModuleSurfacesForRoute,
  listModuleSurfacesForService,
  listModuleSurfacesForWorkflowStep,
  resolveModuleSurface,
} from './domain/module-surface-registry'
export type {
  WorkflowStepComputeMode,
  WorkflowStepDefinition,
  WorkflowStepId,
  WorkflowStepOwner,
  WorkflowStepRegistry,
} from './domain/workflow-step-registry'
export {
  WORKFLOW_STEP_DEFINITIONS,
  WORKFLOW_STEP_IDS,
  WORKFLOW_STEP_REGISTRY,
  isWorkflowStepId,
  listWorkflowSteps,
  listWorkflowStepsForSurface,
  resolveNextWorkflowStep,
  resolvePreviousWorkflowStep,
  resolveWorkflowStep,
} from './domain/workflow-step-registry'
export type {
  CadToolCapability,
  CadToolCategory,
  CadToolId,
  CadToolOwner,
  CadToolRuntimePlacement,
  CadToolDefinition as TlantiCadToolDefinition,
} from './domain/cad-tool-registry'
export {
  CAD_EXPERT_TOOL_MODE_IDS,
  CAD_TOOL_DEFINITIONS,
  CAD_TOOL_LEGACY_ALIASES,
  CAD_VIEWPORT_TOOL_MODE_IDS,
  listCadToolsForClinicalModule,
  listCadToolsForModule,
  listCadToolsForProductModule,
  normalizeCadToolId,
  resolveCadToolDefinition,
  validateCadToolPlatform,
} from './domain/cad-tool-registry'
export type {
  CadCommand,
  CadCommandEffect,
  CadCommandId,
  CadCommandOwner,
  CadCommandPayload,
  CadCommandRunResult,
  CadCommandValidationResult,
} from './domain/cad-command'
export {
  createCadCommand,
  createCadToolCommand,
  runCadCommandInMemory,
  validateCadCommand,
} from './domain/cad-command'
export type {
  CadContextMenuItem,
  CadContextMenuModuleId,
  CadContextMenuProvider,
  CadContextMenuRegistry,
  CadContextObjectKind,
  CadContextSource,
} from './domain/context-menu-registry'
export {
  createCadContextMenuRegistry,
} from './domain/context-menu-registry'
export {
  alignerContextMenuProvider,
  cadContextMenuProvider,
  cephContextMenuProvider,
  createDefaultCadContextMenuProviders,
  dicomContextMenuProvider,
  implantContextMenuProvider,
  splintContextMenuProvider,
} from './domain/context-menu-providers'
export type {
  CadCommandRunnerPort,
} from './use-cases/run-cad-command'
export {
  CadCommandUseCase,
  InMemoryCadCommandRunner,
  createCadCommandUseCase,
} from './use-cases/run-cad-command'
export type {
  AssetManifest,
  CadCoreAssetKind,
  CadCoreDependency,
  CadCoreLayer,
  CadCoreModuleId,
  CadCoreServiceContract,
  CadCoreToolCategory,
  CadJobArtifact,
  CadJobRequest,
  CadJobStatus,
  CadObject,
  CadToolContext,
  CadToolDefinition,
  CaseFolderLayout,
  CaseManifest,
  CaseRecord,
  ConstructionGraph,
  ConstructionNode,
  FeaturePermissionContext,
  FeaturePermissionExpression,
  Layer,
  ModuleManifest,
  PatientRecord,
  SceneGraph,
  SelectionSet,
  ToolRegistry,
  ToothIndication,
  WizardDefinition,
  WizardGuard,
  WizardGuardKind,
  WizardStep,
  WorkDefinition,
} from './domain/cad-core-platform'
export type {
  MeshVaultFormat,
  MeshVaultGpuHints,
  MeshVaultHandle,
  MeshVaultImportRequest,
  MeshVaultJobSnapshot,
  MeshVaultJobStatus,
  MeshVaultProgressEvent,
} from './domain/mesh-vault'
export {
  estimateMeshVaultMemoryCeiling,
  isMeshVaultComplete,
} from './domain/mesh-vault'
export type {
  MeshEngineBackend,
  MeshEngineBackendId,
  MeshEngineOperation,
  MeshProcessingPlan,
  MeshUriResolution,
} from './domain/mesh-engine'
export {
  resolveLocalMeshUri,
} from './adapters/local-mesh-uri-resolver'
export {
  TLANTI_MESH_ENGINE_BACKENDS,
  createMeshProcessingPlan,
  selectMeshBackend,
} from './adapters/meshlib-vtk-algebra-engine'
export type {
  MeshEngineWorkflowRequest,
  MeshEngineWorkflowResult,
} from './use-cases/mesh-engine-workflow-use-case'
export {
  MeshEngineWorkflowUseCase,
} from './use-cases/mesh-engine-workflow-use-case'
export type {
  LocalImagingEnginePort,
  LocalImagingSeriesManifest,
  LocalImagingSeriesRequest,
} from './adapters/volview-local-imaging-engine'
export {
  VolViewLocalImagingEngine,
} from './adapters/volview-local-imaging-engine'
export type {
  ImportDicomSeriesWorkflowRequest,
  ImportDicomSeriesWorkflowResult,
} from './use-cases/imaging-workflow-use-case'
export {
  ImagingWorkflowUseCase,
} from './use-cases/imaging-workflow-use-case'
export {
  CAD_CORE_CASE_DIRECTORIES,
  CAD_CORE_DEPENDENCIES,
  CAD_CORE_MODULE_MANIFESTS,
  CAD_CORE_SERVICE_CONTRACTS,
  CAD_CORE_TOOLS,
  CAD_CORE_WIZARDS,
  buildCaseFolderLayout,
  createCaseManifest,
  createWorkDefinition,
  evaluateFeaturePermissionExpression,
  listCadCoreToolsForModule,
  resolveCadCoreModuleManifest,
  validateCadCorePlatform,
} from './domain/cad-core-platform'
export type {
  AssetPlacementDefinition,
  AssetPlacementId,
  AssetLibraryRootDefinition,
  AssetLibraryRootId,
  AssetRuntimeOwner,
  ModuleAssetUsageDefinition,
} from './domain/asset-library-registry'
export {
  ASSET_PLACEMENTS,
  ASSET_LIBRARY_ROOTS,
  MODULE_ASSET_USAGE,
  listAssetPlacements,
  listAssetPlacementsForModule,
  listAssetLibraryRoots,
  listAssetUsageForModule,
} from './domain/asset-library-registry'
export type {
  ClinicalAiRuntimeOwner,
  ClinicalAiToolDefinition,
  ClinicalAiToolId,
} from './domain/clinical-ai-tool-registry'
export {
  CLINICAL_AI_TOOL_DEFINITIONS,
  listClinicalAiToolsForModule,
} from './domain/clinical-ai-tool-registry'
export { InMemoryCaseRepository } from './adapters/in-memory-case-repository'
export { PythonJawMotionRepository } from './adapters/python-jaw-motion-repository'
export { TauriAssetStorage } from './adapters/tauri-asset-storage'
export type { TauriStoredAssetRef } from './adapters/tauri-asset-storage'
export { TauriCadCoreOrchestrator } from './adapters/tauri-cad-core-orchestrator'
export { TauriCaseRepository } from './adapters/tauri-case-repository'
export { TauriClinicalCommandRepository } from './adapters/tauri-clinical-command-repository'
export { TauriClinicalJobRepository } from './adapters/tauri-clinical-job-repository'
export { TauriMeshVault } from './adapters/tauri-mesh-vault'
export { createTauriPersistibleClinicalToolUseCase } from './adapters/tauri-persistible-clinical-tool-runner'
export type { AssetStorage, AssetWriteRequest, StoredAssetRef } from './ports/asset-storage'
export type { MeshVault, MeshVaultProgressUnsubscribe } from './ports/mesh-vault'
export type {
  AssetImportPreparation,
  AssetImportPrepareRequest,
  CadCoreBootstrap,
  CadCoreBootstrapRequest,
  CadCoreOrchestrator,
  CaseOpenResponse,
  ModulePermissionDecision,
  ModulePermissionsRequest,
  ModulePermissionsResponse,
} from './ports/cad-core-orchestrator'
export type { CaseCreateInput, CaseRepository } from './ports/case-repository'
export type {
  ClinicalCommandEvent,
  ClinicalCommandEventType,
  ClinicalCommandRecordRequest,
  ClinicalCommandRepository,
} from './ports/clinical-command-repository'
export type {
  ClinicalArtifactRecord,
  ClinicalArtifactRecordRequest,
  ClinicalJobRecord,
  ClinicalJobRecordRequest,
  ClinicalJobRepository,
  ClinicalJobStatus,
} from './ports/clinical-job-repository'
export type {
  ClinicalJob,
  JawMotionGenerateRequest,
  JawMotionRepository,
  JawMotionResult,
} from './ports/jaw-motion-repository'
export type { ModuleWorkflowRepository } from './ports/module-workflow-repository'
export {
  completeCurrentWorkflowStep,
  createModuleWorkflowState,
  getModuleWorkflowProgress,
  validateTlantiModuleRegistry,
} from './use-cases/module-workflow'
export type {
  ClinicalToolRuntime,
  PersistibleClinicalToolDefinition,
  PersistibleClinicalToolId,
  PersistibleClinicalToolJobParams,
  RunPersistibleClinicalToolRequest,
} from './use-cases/cad-clinical-tool-use-case'
export {
  PERSISTIBLE_CLINICAL_TOOL_DEFINITIONS,
  PersistibleClinicalToolUseCase,
  buildPersistibleClinicalToolJobRequest,
  listPersistibleClinicalToolsForModule,
  resolvePersistibleClinicalToolDefinition,
} from './use-cases/cad-clinical-tool-use-case'
export type {
  ImportPathBackedAssetRequest,
  ImportPathBackedAssetResult,
} from './use-cases/mesh-vault-import-use-case'
export { MeshVaultImportUseCase } from './use-cases/mesh-vault-import-use-case'
export type { ImportClinicalArtifactRequest } from './use-cases/import-clinical-artifact-use-case'
export { ImportClinicalArtifactUseCase } from './use-cases/import-clinical-artifact-use-case'
export type { RecordClinicalCommandRequest } from './use-cases/clinical-command-use-case'
export { ClinicalCommandUseCase } from './use-cases/clinical-command-use-case'

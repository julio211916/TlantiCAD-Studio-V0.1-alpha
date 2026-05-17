import type {
  DentalImplantMode,
  DentalMaterialType,
  DentalProductionMethod,
  DentalRestorationType,
} from '@/lib/dental-workflow';
import { parsePersistedTlantiDbState } from '@/components/tlantidb/domain/tlantidb-schemas';

export type TlantiCaseAssetCategory = string;
export type TlantiCaseAssetRole = string;

export type TlantiDbRenderQuality = 'low' | 'medium' | 'high' | 'ultra';
export type TlantiDbThumbnailQuality = 'low' | 'medium' | 'high';
export type TlantiDbPerformanceMode = 'auto' | 'manual';
export type TlantiCadKernelPreference = 'mesh-first' | 'hybrid' | 'b-rep';
export type TlantiCadKernel = 'auto' | 'meshlib' | 'occt' | 'hybrid';
export type TlantiCaseRemoteJobStatus = string;
export type TlantiCaseRemoteJobTarget = string;
export type TlantiCaseCollaboratorRole = string;
export type TlantiCaseApprovalStatus = string;
export type TlantiCaseAutoPlanStatus = string;

export interface TlantiDbPerformanceProfile {
  mode: TlantiDbPerformanceMode;
  source: 'default' | 'auto' | 'manual' | string;
  renderQuality: TlantiDbRenderQuality;
  dicomCacheMb: number;
  enableGpuInference: boolean;
  enable3dViewer: boolean;
  enableCbctViewer: boolean;
  maxConcurrentImports: number;
  thumbnailQuality: TlantiDbThumbnailQuality;
  overallScore: number;
  machineSignature: string | null;
  lastAutoAppliedAt: string | null;
  [key: string]: any;
}

export interface TlantiToothState {
  selected: boolean;
  antagonist: boolean;
  restorationType?: DentalRestorationType;
  material?: DentalMaterialType;
  productionMethod?: DentalProductionMethod;
  implantMode?: DentalImplantMode;
  shade?: string;
  workTypeId?: string;
  legacyType?: string;
  workTimeMinutes?: number;
  minimalThicknessMm?: number;
  cementGapMm?: number;
  connectorWidthMm?: number;
  additionalScans?: {
    preOpModel?: boolean;
    extraGingiva?: boolean;
    substructureScan?: boolean;
    waxup?: boolean;
    [key: string]: any;
  };
  notes?: string;
  [key: string]: any;
}

export interface TlantiCaseAsset {
  id: string;
  name: string;
  category: TlantiCaseAssetCategory;
  role: TlantiCaseAssetRole;
  path?: string;
  sourcePath?: string;
  storagePath?: string;
  relativePath?: string;
  sizeBytes?: number | null;
  importedAt: string;
  tags?: string[];
  metadata?: Record<string, any>;
  [key: string]: any;
}

export interface TlantiCasePipeline {
  scan: boolean;
  design: boolean;
  model: boolean;
  manufacture: boolean;
  export: boolean;
  [key: string]: any;
}

export interface TlantiCaseComment {
  id: string;
  author: string;
  body?: string;
  message?: string;
  role?: TlantiCaseCollaboratorRole;
  createdAt: string;
  [key: string]: any;
}

export interface TlantiCaseDecision {
  id: string;
  title?: string;
  label?: string;
  status: string;
  rationale?: string;
  actor?: string;
  createdAt: string;
  [key: string]: any;
}

export interface TlantiCaseApproval {
  id: string;
  role?: TlantiCaseCollaboratorRole;
  approved?: boolean;
  status?: TlantiCaseApprovalStatus;
  label?: string;
  note?: string;
  signer?: string;
  createdAt?: string;
  signedAt?: string;
  [key: string]: any;
}

export interface TlantiCaseNotification {
  id: string;
  title: string;
  detail?: string;
  read: boolean;
  createdAt: string;
  [key: string]: any;
}

export interface TlantiCaseCollaborationState {
  reviewLock: { owner?: string; holder?: string; lockedAt?: string; acquiredAt?: string; [key: string]: any } | null;
  comments: TlantiCaseComment[];
  decisions: TlantiCaseDecision[];
  approvals: TlantiCaseApproval[];
  notifications: TlantiCaseNotification[];
  autoPlanningFeedback: Record<string, { status?: TlantiCaseAutoPlanStatus; note?: string; [key: string]: any }>;
  [key: string]: any;
}

export interface TlantiCaseRemoteJob {
  id: string;
  label: string;
  target: TlantiCaseRemoteJobTarget;
  status: TlantiCaseRemoteJobStatus;
  quotaUnits: number;
  estimatedCostUsd: number;
  createdAt: string;
  updatedAt: string;
  lastError?: string | null;
  encryptedTransport?: boolean;
  syncAssets?: boolean;
  retryCount: number;
  maxRetries?: number;
  [key: string]: any;
}

export interface TlantiCaseOperationsState {
  remoteJobs: TlantiCaseRemoteJob[];
  kernelTransition: {
    preference: TlantiCadKernelPreference;
    preferredKernel: TlantiCadKernel;
    offlineFallback: boolean;
    geometryBenchmarkScore: number | null;
    lastPolicyUpdateAt: string | null;
    [key: string]: any;
  };
  [key: string]: any;
}

export interface TlantiCase {
  id: string;
  name: string;
  caseNumber: string;
  status: string;
  clientName: string;
  clientId: string;
  patientName?: string;
  patientDateOfBirth?: string;
  orderNumber: string;
  laboratoryName?: string;
  technicianName: string;
  technicianId: string;
  notes: string;
  activeJaw: 'upper' | 'lower' | 'both';
  toothMap: Record<string, TlantiToothState>;
  connectorOverrides: Record<string, any>;
  assets: TlantiCaseAsset[];
  pipeline: TlantiCasePipeline;
  collaboration: TlantiCaseCollaborationState;
  operations: TlantiCaseOperationsState;
  storagePath?: string | null;
  lastInteropXmlPath?: string | null;
  lastExportedAt?: string | null;
  createdAt: string;
  updatedAt: string;
  workloadId?: string;
  workloadLabel?: string;
  moduleTarget?: string;
  requiredAssetRoles?: TlantiCaseAssetRole[];
  optionalAssetRoles?: TlantiCaseAssetRole[];
  workloadStatus?: string;
  lastOpenedModule?: string;
  moduleId?: string;
  sourceType?: string;
  lastOpenedAt?: string;
  [key: string]: any;
}

export interface TlantiDbPreferences {
  timeZone: string;
  numberingSystem: 'FDI' | 'UNIVERSAL';
  assetProfile: 'clinical' | 'lab' | 'demo';
  operatorAlias: string;
  navigationSensitivity: {
    zoom: number;
    pan: number;
    rotation: number;
  };
  performanceProfile: TlantiDbPerformanceProfile;
  [key: string]: any;
}

export interface TlantiDbState {
  activeCaseId: string;
  cases: TlantiCase[];
  preferences: TlantiDbPreferences;
  [key: string]: any;
}

const STORAGE_KEY = 'tlanticad:tlantidb-state:v1';
const subscribers = new Set<(state: TlantiDbState) => void>();

function nowIso() {
  return new Date().toISOString();
}

function createDefaultToothMap(): Record<string, TlantiToothState> {
  const map: Record<string, TlantiToothState> = {};
  for (const tooth of ['18','17','16','15','14','13','12','11','21','22','23','24','25','26','27','28','48','47','46','45','44','43','42','41','31','32','33','34','35','36','37','38']) {
    map[`tooth-${tooth}`] = {
      selected: false,
      antagonist: false,
      restorationType: 'anatomic-crown',
      material: 'zirconia',
      productionMethod: 'inhouse-milling',
      implantMode: 'none',
    };
  }
  return map;
}

export function createDefaultCase(patch: Partial<TlantiCase> = {}): TlantiCase {
  const timestamp = nowIso();
  const id = patch.id ?? `case-${Date.now().toString(36)}`;
  return {
    id,
    name: patch.name ?? 'New restorative case',
    caseNumber: patch.caseNumber ?? id.slice(-8).toUpperCase(),
    status: patch.status ?? 'new',
    clientName: patch.clientName ?? 'Nuevo paciente',
    clientId: patch.clientId ?? '',
    patientName: patch.patientName ?? '',
    patientDateOfBirth: patch.patientDateOfBirth ?? '',
    orderNumber: patch.orderNumber ?? '',
    laboratoryName: patch.laboratoryName ?? 'Tlanti Lab',
    technicianName: patch.technicianName ?? '',
    technicianId: patch.technicianId ?? '',
    notes: patch.notes ?? '',
    activeJaw: patch.activeJaw ?? 'upper',
    toothMap: patch.toothMap ?? createDefaultToothMap(),
    connectorOverrides: patch.connectorOverrides ?? {},
    assets: patch.assets ?? [],
    pipeline: patch.pipeline ?? { scan: false, design: false, model: false, manufacture: false, export: false },
    collaboration: patch.collaboration ?? {
      reviewLock: null,
      comments: [],
      decisions: [],
      approvals: [],
      notifications: [],
      autoPlanningFeedback: {},
    },
    operations: patch.operations ?? {
      remoteJobs: [],
      kernelTransition: {
        preference: 'mesh-first',
        preferredKernel: 'auto',
        offlineFallback: true,
        geometryBenchmarkScore: null,
        lastPolicyUpdateAt: null,
      },
    },
    storagePath: patch.storagePath ?? null,
    lastInteropXmlPath: patch.lastInteropXmlPath ?? null,
    lastExportedAt: patch.lastExportedAt ?? null,
    createdAt: patch.createdAt ?? timestamp,
    updatedAt: patch.updatedAt ?? timestamp,
    ...patch,
  };
}

export function createDefaultTlantiDbState(): TlantiDbState {
  const initialCase = createDefaultCase();
  return {
    activeCaseId: initialCase.id,
    cases: [initialCase],
    preferences: {
      timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'America/Mexico_City',
      numberingSystem: 'FDI',
      assetProfile: 'clinical',
      operatorAlias: 'TlantiCAD',
      navigationSensitivity: { zoom: 1, pan: 1, rotation: 1 },
      performanceProfile: {
        mode: 'auto',
        source: 'default',
        renderQuality: 'medium',
        dicomCacheMb: 512,
        enableGpuInference: false,
        enable3dViewer: true,
        enableCbctViewer: true,
        maxConcurrentImports: 2,
        thumbnailQuality: 'medium',
        overallScore: 50,
        machineSignature: null,
        lastAutoAppliedAt: null,
      },
      useInteractiveOdontogram: true,
    },
  };
}

export function saveTlantiDbState(state: TlantiDbState): void {
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  }
  subscribers.forEach((listener) => listener(state));
}

export function loadTlantiDbState(): TlantiDbState {
  if (typeof window === 'undefined') return createDefaultTlantiDbState();
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return createDefaultTlantiDbState();
    const parsed = parsePersistedTlantiDbState(JSON.parse(raw));
    if (!parsed) return createDefaultTlantiDbState();
    const fallback = createDefaultTlantiDbState();
    return {
      ...fallback,
      ...parsed,
      cases: parsed.cases?.length ? parsed.cases.map((item) => createDefaultCase(item)) : fallback.cases,
      preferences: { ...fallback.preferences, ...parsed.preferences },
    };
  } catch {
    return createDefaultTlantiDbState();
  }
}

export async function hydrateTlantiDbState(options: { freshStartup?: boolean } = {}): Promise<TlantiDbState> {
  return options.freshStartup ? createDefaultTlantiDbState() : loadTlantiDbState();
}

export function shouldCreateFreshStartupCase(caseId?: string): boolean {
  return Boolean(caseId);
}

export function subscribeTlantiDbState(listener: (state: TlantiDbState) => void): () => void {
  subscribers.add(listener);
  return () => subscribers.delete(listener);
}

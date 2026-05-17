/**
 * AI runtime domain — Block AS (V320).
 *
 * Mirrors the backend `/ai-runtime/status` payload. Pure types + helpers,
 * no React. The panel imports the typed model and an adapter pulls JSON.
 */

export type DeviceKind = 'cuda' | 'mps' | 'cpu';

export interface AIDevice {
    kind: DeviceKind;
    name: string;
    torchDevice: string;
    available: boolean;
    relativeLatency: number;
    detail: string;
}

export interface AIMemory {
    deviceKind: string;
    totalBytes: number;
    availableBytes: number;
    usedBytes: number;
    safeForVolumeVoxels: number;
}

export interface AIModelRecord {
    modelId: string;
    relativePath: string;
    sourceUrl: string;
    expectedBytes: number;
    sizeBytes: number;
    installed: boolean;
    integrityOk: boolean;
    sha256Expected: string | null;
    sha256Measured: string;
    lastVerifiedAt: number | null;
    lastError: string;
}

export interface AIRuntimeStatus {
    device: AIDevice;
    memory: AIMemory;
    models: AIModelRecord[];
    schemaVersion: number;
    updatedAt: number;
}

export interface AICacheEntry {
    key: string;
    modelId: string;
    bytesOnDisk: number;
    createdAt: number;
    accessedAt: number;
}

export interface AICacheState {
    root: string;
    budgetBytes: number;
    totalBytes: number;
    entryCount: number;
    entries: AICacheEntry[];
}

export type LocalRuntimeCapabilityState = 'ready' | 'degraded' | 'unavailable';

export interface LocalRuntimeCapability {
    id: string;
    label: string;
    state: LocalRuntimeCapabilityState;
    offlineOnly: true;
    detail: string;
    requiredFor: string[];
}

export interface LocalAIRuntimeHealthCheck {
    scope: 'ai-runtime';
    schemaVersion: 1;
    checkedAt: number;
    offlineOnly: true;
    overall: LocalRuntimeCapabilityState;
    capabilities: LocalRuntimeCapability[];
    blockers: string[];
    performanceBudget: {
        maxModelBytesInMemory: number | null;
        safeVolumeVoxels: number | null;
        expectedBackend: DeviceKind | 'unavailable';
    };
}

function aggregateCapabilityState(capabilities: LocalRuntimeCapability[]): LocalRuntimeCapabilityState {
    if (capabilities.some((capability) => capability.state === 'unavailable')) {
        return 'unavailable';
    }
    if (capabilities.some((capability) => capability.state === 'degraded')) {
        return 'degraded';
    }
    return 'ready';
}

export function createLocalAIRuntimeHealthCheck(
    status: AIRuntimeStatus | null,
): LocalAIRuntimeHealthCheck {
    const installedModels = status?.models.filter((model) => model.installed) ?? [];
    const verifiedModels = installedModels.filter(
        (model) => !model.sha256Expected || model.integrityOk,
    );
    const deviceReady = Boolean(status?.device.available);
    const memoryReady = Boolean(status?.memory.availableBytes && status.memory.availableBytes > 0);

    const capabilities: LocalRuntimeCapability[] = [
        {
            id: 'local-device',
            label: 'Local inference device',
            state: deviceReady ? 'ready' : 'unavailable',
            offlineOnly: true,
            detail: status
                ? `${status.device.name} (${status.device.kind})`
                : 'No local AI runtime status has been provided by Tauri/Python.',
            requiredFor: ['DICOM segmentation', 'mesh labeling', 'panoramic segmentation'],
        },
        {
            id: 'model-store',
            label: 'Offline model store',
            state:
                installedModels.length === 0
                    ? 'unavailable'
                    : verifiedModels.length === installedModels.length
                      ? 'ready'
                      : 'degraded',
            offlineOnly: true,
            detail:
                installedModels.length === 0
                    ? 'No installed local models found.'
                    : `${verifiedModels.length}/${installedModels.length} installed models verified.`,
            requiredFor: ['ONNX inference', 'TorchScript fallback'],
        },
        {
            id: 'memory-budget',
            label: 'Volume memory budget',
            state: memoryReady ? 'ready' : 'unavailable',
            offlineOnly: true,
            detail: status
                ? `${formatBytes(status.memory.availableBytes)} available for local inference.`
                : 'No local memory budget available.',
            requiredFor: ['CBCT volume preprocessing', 'sliding-window inference'],
        },
    ];
    const blockers = capabilities
        .filter((capability) => capability.state === 'unavailable')
        .map((capability) => capability.detail);

    return {
        scope: 'ai-runtime',
        schemaVersion: 1,
        checkedAt: Date.now(),
        offlineOnly: true,
        overall: aggregateCapabilityState(capabilities),
        capabilities,
        blockers,
        performanceBudget: {
            maxModelBytesInMemory: status?.memory.availableBytes ?? null,
            safeVolumeVoxels: status?.memory.safeForVolumeVoxels ?? null,
            expectedBackend: status?.device.available ? status.device.kind : 'unavailable',
        },
    };
}

export function formatBytes(bytes: number): string {
    if (bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.min(Math.floor(Math.log10(bytes) / 3), units.length - 1);
    return `${(bytes / 10 ** (i * 3)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function deviceLatencyLabel(device: AIDevice): string {
    if (device.kind === 'cuda') return 'CUDA — fastest';
    if (device.kind === 'mps') return 'MPS — ~3× CUDA';
    return 'CPU — ~12× CUDA, large jobs may take minutes';
}

export function modelHealthLabel(model: AIModelRecord): 'missing' | 'corrupt' | 'unverified' | 'ok' {
    if (!model.installed) return 'missing';
    if (model.sha256Expected && !model.integrityOk) return 'corrupt';
    if (model.sha256Expected && !model.lastVerifiedAt) return 'unverified';
    return 'ok';
}

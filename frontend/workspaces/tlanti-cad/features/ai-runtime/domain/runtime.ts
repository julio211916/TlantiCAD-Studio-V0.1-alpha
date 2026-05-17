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

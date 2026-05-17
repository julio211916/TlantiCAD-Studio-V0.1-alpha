import type { AICacheState, AIRuntimeStatus } from '../domain/runtime';

export interface AIRuntimePort {
    status(): Promise<AIRuntimeStatus>;
    verify(modelId: string): Promise<{ integrityOk: boolean; lastError: string }>;
    cache(): Promise<AICacheState>;
    prune(): Promise<{ freedBytes: number; totalBytesAfter: number }>;
    clearCache(): Promise<void>;
}

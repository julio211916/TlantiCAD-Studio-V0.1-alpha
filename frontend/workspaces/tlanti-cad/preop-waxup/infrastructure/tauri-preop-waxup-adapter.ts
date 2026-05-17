import type {
    PreopAlignInput,
    PreopWaxupPort,
} from '../application/preop-waxup-port';
import type { Mat4, PreopAlignment } from '../domain/preop-waxup';
import { createBackendPreopWaxupAdapter } from './backend-preop-waxup-adapter';

type InvokeArgs = Record<string, unknown>;

interface AlignmentRegisterResponse {
    result: {
        matrix: Mat4;
        rmsMm: number;
        iterations: number;
        converged: boolean;
        correspondenceCount: number;
        warnings: string[];
    };
    transformPath: string | null;
    alignedMeshPath: string | null;
    backend: string;
}

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args);
}

export function createTauriPreopWaxupAdapter(
    fallback: PreopWaxupPort = createBackendPreopWaxupAdapter(),
): PreopWaxupPort {
    return {
        async alignPreop(input: PreopAlignInput): Promise<PreopAlignment> {
            const response = await invokeCommand<AlignmentRegisterResponse>('alignment_register_meshes', {
                request: {
                    movingMeshPath: input.preopPath,
                    fixedMeshPath: input.modelPath,
                    mode: 'IterativeClosestPoint',
                    maxIterations: 32,
                    toleranceMm: 0.0001,
                    sampleLimit: 2500,
                    writeAlignedMesh: false,
                },
            });
            return {
                transformMatrix: response.result.matrix,
                rmsMm: response.result.rmsMm,
                backend: response.backend,
            };
        },

        adaptToPreop(input) {
            return fallback.adaptToPreop(input);
        },

        prepareWaxup(input) {
            return fallback.prepareWaxup(input);
        },
    };
}

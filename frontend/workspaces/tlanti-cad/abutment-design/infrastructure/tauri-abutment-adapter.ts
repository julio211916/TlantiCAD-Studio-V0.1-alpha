import type {
    AbutmentGenerateMeshRequest,
    AbutmentGenerateMeshResponse,
    AbutmentPort,
    AbutmentScrewChannelPlan,
    AbutmentScrewChannelRequest,
    AbutmentValidationRequest,
    AbutmentValidationResponse,
} from '../application/abutment-port';
import { createBackendAbutmentAdapter } from './backend-abutment-adapter';

type InvokeArgs = Record<string, unknown>;

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args);
}

export function createTauriAbutmentAdapter(
    validationPort: Pick<AbutmentPort, 'validate'> = createBackendAbutmentAdapter(),
): AbutmentPort {
    return {
        validate(input: AbutmentValidationRequest): Promise<AbutmentValidationResponse> {
            return validationPort.validate(input);
        },

        generateMesh(input: AbutmentGenerateMeshRequest): Promise<AbutmentGenerateMeshResponse> {
            return invokeCommand<AbutmentGenerateMeshResponse>('abutment_generate_mesh', {
                request: input,
            });
        },

        planScrewChannel(input: AbutmentScrewChannelRequest): Promise<AbutmentScrewChannelPlan> {
            return invokeCommand<AbutmentScrewChannelPlan>('abutment_plan_screw_channel', {
                request: input,
            });
        },
    };
}

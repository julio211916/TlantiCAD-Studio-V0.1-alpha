import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type {
    AbutmentGenerateMeshRequest,
    AbutmentGenerateMeshResponse,
    AbutmentPort,
    AbutmentScrewChannelPlan,
    AbutmentScrewChannelRequest,
    AbutmentValidationRequest,
    AbutmentValidationResponse,
} from '../application/abutment-port';

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

export function createBackendAbutmentAdapter(baseUrl: string = BACKEND_ORIGIN): AbutmentPort {
    return {
        async validate(input: AbutmentValidationRequest): Promise<AbutmentValidationResponse> {
            return callJson<AbutmentValidationResponse>(`${baseUrl}/cad/abutment/validate`, {
                method: 'POST',
                body: JSON.stringify({
                    outputPath: input.outputPath,
                    minThicknessMm: input.minThicknessMm,
                    angulatedScrewChannelDeg: input.angulatedScrewChannelDeg,
                    implantLibraryMaxDeg: input.implantLibraryMaxDeg ?? 25,
                }),
            });
        },
        async generateMesh(_input: AbutmentGenerateMeshRequest): Promise<AbutmentGenerateMeshResponse> {
            throw new Error('Abutment mesh generation is native Rust/Tauri only. Use createTauriAbutmentAdapter().');
        },
        async planScrewChannel(_input: AbutmentScrewChannelRequest): Promise<AbutmentScrewChannelPlan> {
            throw new Error('Abutment screw-channel planning is native Rust/Tauri only. Use createTauriAbutmentAdapter().');
        },
    };
}

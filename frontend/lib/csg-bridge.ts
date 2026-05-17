/**
 * V201 — CSG bridge.
 *
 * Wraps the Tauri command `cad_csg::mesh_op` so call-sites stay agnostic to
 * the runtime: in Tauri the call goes through IPC; in the browser preview the
 * helper rejects with a typed error so callers can fall back to the Python
 * mock.
 */

import { isTauriRuntime } from '../platform/desktop-system';
import { logger } from './logger';

export type CsgOp = 'union' | 'subtract' | 'intersect';

export interface MeshOpRequest {
    op: CsgOp;
    /** Absolute paths to STL files. First is the base; rest are operands. */
    inputs: string[];
    /** Where the kernel writes the output STL. */
    output: string;
    repair?: boolean;
}

export interface MeshOpResponse {
    output: string;
    triangles: number;
    watertight: boolean;
    volumeMm3: number;
    genus: number;
    backend: string;
}

export interface MeshOpError {
    kind: string;
    message?: string;
}

/**
 * Call the Tauri CSG kernel. Throws `{ kind: 'csg-bridge-not-available' }`
 * when not running inside Tauri so the caller can branch into a fallback.
 */
export async function invokeMeshOp(request: MeshOpRequest): Promise<MeshOpResponse> {
    if (!isTauriRuntime()) {
        throw { kind: 'csg-bridge-not-available' } as MeshOpError;
    }
    // Lazy import — `@tauri-apps/api/core` is unavailable in the browser preview.
    const { invoke } = await import('@tauri-apps/api/core');
    try {
        return await invoke<MeshOpResponse>('mesh_op', { request });
    } catch (err) {
        logger.warn('mesh_op invoke failed', err);
        throw err as MeshOpError;
    }
}

/**
 * Fire-and-forget wrapper that swallows `csg-bridge-not-available`. Useful
 * when CSG is an optional improvement step (Merge & Save). Returns null
 * when the bridge is not available so callers can downgrade gracefully.
 */
export async function invokeMeshOpOrNull(
    request: MeshOpRequest,
): Promise<MeshOpResponse | null> {
    try {
        return await invokeMeshOp(request);
    } catch (err) {
        const e = err as MeshOpError;
        if (e?.kind === 'csg-bridge-not-available') return null;
        // Real kernel error — let it bubble.
        throw err;
    }
}

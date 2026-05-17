/**
 * Cliente IPC central: envuelve `@tauri-apps/api/core` con tipado fuerte,
 * timeouts, logging y error-mapping.
 *
 * Uso recomendado:
 * ```ts
 * import { ipc, type CmdGetRuntimeInfoResult } from "@/lib/ipc"
 * const info = await ipc<void, CmdGetRuntimeInfoResult>("get_runtime_info")
 * ```
 *
 * Para wrappers tipados por feature, los `services/*` deben envolver
 * `ipc()` con la firma exacta de `contracts.ts` y exponer una API de
 * dominio en lugar de strings sueltos.
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

import {
    compose,
    errorMappingMiddleware,
    loggingMiddleware,
    timeoutMiddleware,
    type IpcContext,
    type IpcMiddleware,
} from "./middleware";

/** Timeout por defecto cuando el comando no especifica uno propio. */
const DEFAULT_TIMEOUT_MS = 30_000;

export interface IpcCallOptions {
    /** Vence la llamada tras `timeoutMs`. `0` o `undefined` desactiva. */
    timeoutMs?: number;
    /** Permite cancelar la llamada externamente. */
    signal?: AbortSignal;
    /** Sustituye la cadena de middleware por defecto (avanzado). */
    middleware?: IpcMiddleware;
}

const DEFAULT_PIPELINE: IpcMiddleware = compose(
    loggingMiddleware,
    timeoutMiddleware(DEFAULT_TIMEOUT_MS),
    errorMappingMiddleware,
);

/**
 * Invoca un comando Tauri pasando por la pipeline de middleware.
 *
 * `TArgs` puede ser `void` para comandos sin parámetros — Tauri acepta
 * `undefined` y omite el payload.
 */
export async function ipc<TArgs extends Record<string, unknown> | void, TResult>(
    command: string,
    args?: TArgs,
    opts?: IpcCallOptions,
): Promise<TResult> {
    const ctx: IpcContext = {
        command,
        args: (args ?? undefined) as Record<string, unknown> | undefined,
        startedAt: performance.now(),
        timeoutMs: opts?.timeoutMs,
        signal: opts?.signal,
    };

    const pipeline = opts?.middleware ?? DEFAULT_PIPELINE;

    const result = await pipeline(ctx, () => {
        if (ctx.args === undefined) {
            return tauriInvoke(command);
        }
        return tauriInvoke(command, ctx.args);
    });

    return result as TResult;
}

/**
 * Helper para construir un caller fuertemente tipado a partir de un
 * contrato. Útil dentro de un `service` cuando quieres reutilizar la
 * misma firma muchas veces sin repetir `<TArgs, TResult>`.
 */
export function bindCommand<TArgs extends Record<string, unknown> | void, TResult>(
    command: string,
    defaultOpts?: IpcCallOptions,
) {
    return (args?: TArgs, opts?: IpcCallOptions) =>
        ipc<TArgs, TResult>(command, args, { ...defaultOpts, ...opts });
}

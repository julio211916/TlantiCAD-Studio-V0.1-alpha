/**
 * Middleware composables para la capa IPC.
 *
 * Modelo: cada middleware recibe un `IpcContext` mutable + `next()`. Puede
 * envolver la promesa, instrumentarla, o cortocircuitar (lanzar / devolver
 * un valor sin llamar a `next`).
 *
 * `compose(...mws)` produce un único `IpcMiddleware` que ejecuta la cadena
 * en el orden dado. El `client` aplica:
 *   loggingMiddleware → timeoutMiddleware → errorMappingMiddleware → call
 */

import { logger } from "../logger";
import { classifyIpcError, IpcError } from "./errors";

export interface IpcContext {
    readonly command: string;
    readonly args: Record<string, unknown> | undefined;
    readonly startedAt: number;
    readonly timeoutMs?: number;
    readonly signal?: AbortSignal;
}

export type IpcNext = () => Promise<unknown>;
export type IpcMiddleware = (ctx: IpcContext, next: IpcNext) => Promise<unknown>;

/**
 * Compone middlewares en una sola función. El primero del arreglo es el
 * más externo — se ejecuta primero al entrar y último al salir.
 */
export function compose(...middlewares: IpcMiddleware[]): IpcMiddleware {
    return async (ctx, next) => {
        let index = -1;
        const dispatch = async (i: number): Promise<unknown> => {
            if (i <= index) {
                throw new Error("ipc middleware: next() called multiple times");
            }
            index = i;
            const mw = middlewares[i];
            if (!mw) return next();
            return mw(ctx, () => dispatch(i + 1));
        };
        return dispatch(0);
    };
}

/**
 * Logging estructurado: registra entrada, salida y duración. En caso de
 * error registra el código clasificado para que aparezca en la consola
 * dev sin volcar el stack completo dos veces.
 */
export const loggingMiddleware: IpcMiddleware = async (ctx, next) => {
    const tag = `[ipc] ${ctx.command}`;
    logger.debug(tag, "→", ctx.args ?? {});
    try {
        const result = await next();
        const elapsed = Math.round(performance.now() - ctx.startedAt);
        logger.debug(tag, "✓", `${elapsed}ms`);
        return result;
    } catch (error) {
        const elapsed = Math.round(performance.now() - ctx.startedAt);
        const code = error instanceof IpcError ? error.code : "UNKNOWN";
        logger.warn(tag, "✗", `${elapsed}ms`, code, error);
        throw error;
    }
};

/**
 * Aplica timeout al `next()` y respeta `AbortSignal`. Si vence el plazo
 * o se aborta, lanza `IpcError` con el código apropiado.
 */
export function timeoutMiddleware(defaultMs: number): IpcMiddleware {
    return async (ctx, next) => {
        const ms = ctx.timeoutMs ?? defaultMs;
        const signal = ctx.signal;

        if (signal?.aborted) {
            throw new IpcError(ctx.command, "ABORTED", undefined, `IPC ${ctx.command} aborted`);
        }

        if (!ms || ms <= 0) {
            return next();
        }

        return new Promise<unknown>((resolve, reject) => {
            let settled = false;
            const timer = setTimeout(() => {
                if (settled) return;
                settled = true;
                reject(
                    new IpcError(
                        ctx.command,
                        "TIMEOUT",
                        undefined,
                        `IPC ${ctx.command} timed out after ${ms}ms`,
                    ),
                );
            }, ms);

            const onAbort = () => {
                if (settled) return;
                settled = true;
                clearTimeout(timer);
                reject(
                    new IpcError(
                        ctx.command,
                        "ABORTED",
                        undefined,
                        `IPC ${ctx.command} aborted`,
                    ),
                );
            };
            signal?.addEventListener("abort", onAbort, { once: true });

            next()
                .then((value) => {
                    if (settled) return;
                    settled = true;
                    clearTimeout(timer);
                    signal?.removeEventListener("abort", onAbort);
                    resolve(value);
                })
                .catch((reason: unknown) => {
                    if (settled) return;
                    settled = true;
                    clearTimeout(timer);
                    signal?.removeEventListener("abort", onAbort);
                    reject(reason);
                });
        });
    };
}

/**
 * Convierte cualquier error que escape de `next()` en una `IpcError`
 * tipada. Si ya es una `IpcError` la deja pasar sin re-empaquetar.
 */
export const errorMappingMiddleware: IpcMiddleware = async (ctx, next) => {
    try {
        return await next();
    } catch (error) {
        if (error instanceof IpcError) throw error;
        const code = classifyIpcError(error);
        throw new IpcError(ctx.command, code, error);
    }
};

/**
 * Reintenta `next()` en errores transitorios. Solo para comandos
 * idempotentes — el llamador es responsable de no usarlo en mutaciones.
 */
export function retryMiddleware(options: {
    attempts: number;
    backoffMs?: number;
    retryOn?: ReadonlyArray<"TIMEOUT" | "BACKEND_DOWN" | "INTERNAL" | "UNKNOWN">;
}): IpcMiddleware {
    const max = Math.max(1, options.attempts);
    const backoff = options.backoffMs ?? 0;
    const retryOn = options.retryOn ?? ["TIMEOUT", "BACKEND_DOWN"];
    return async (_ctx, next) => {
        let lastError: unknown;
        for (let attempt = 1; attempt <= max; attempt++) {
            try {
                return await next();
            } catch (error) {
                lastError = error;
                const code = error instanceof IpcError ? error.code : classifyIpcError(error);
                const retriable = (retryOn as readonly string[]).includes(code);
                if (!retriable || attempt === max) throw error;
                if (backoff > 0) {
                    await new Promise<void>((resolve) => setTimeout(resolve, backoff * attempt));
                }
            }
        }
        throw lastError;
    };
}

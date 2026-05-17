/**
 * Barrel del módulo IPC central.
 *
 * Punto único de importación para el resto del frontend:
 *
 * ```ts
 * import {
 *     ipc,
 *     IpcError,
 *     type CmdGetRuntimeInfoResult,
 * } from "@/lib/ipc";
 * ```
 *
 * Re-exporta el cliente, los tipos de error, los middleware composables
 * y todo el catálogo de contratos. Cualquier nuevo comando registrado en
 * `apps/desktop/src-tauri/src/lib.rs` debe añadirse a `contracts.ts` —
 * nunca exponer strings sueltos a `tauriInvoke()` desde fuera de esta
 * carpeta.
 */

export { bindCommand, ipc, type IpcCallOptions } from "./client";
export * from "./contracts";
export { IpcError, type IpcErrorCode, classifyIpcError } from "./errors";
export {
    compose,
    errorMappingMiddleware,
    loggingMiddleware,
    retryMiddleware,
    timeoutMiddleware,
    type IpcContext,
    type IpcMiddleware,
    type IpcNext,
} from "./middleware";

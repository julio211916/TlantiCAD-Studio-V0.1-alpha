/**
 * Tipos de error tipados para la capa IPC central.
 *
 * Cada llamada a `ipc()` en `client.ts` lanza una `IpcError` cuando algo
 * falla — ya sea timeout, abort, error del runtime Tauri, o respuesta de
 * error del backend Rust. El consumidor puede `instanceof IpcError` y
 * ramificar por `error.code` sin parsear strings.
 */

export type IpcErrorCode =
    | "TIMEOUT"
    | "ABORTED"
    | "NOT_FOUND"
    | "INVALID_ARGS"
    | "BACKEND_DOWN"
    | "INTERNAL"
    | "UNKNOWN";

/**
 * Error tipado producido por la capa IPC. `command` y `code` son
 * obligatorios; `cause` retiene el error original (útil para crash
 * reporters) sin filtrar `any` al consumidor.
 */
export class IpcError extends Error {
    public readonly command: string;
    public readonly code: IpcErrorCode;
    public readonly cause?: unknown;

    constructor(command: string, code: IpcErrorCode, cause?: unknown, message?: string) {
        super(message ?? `IPC ${command} failed: ${code}`);
        this.name = "IpcError";
        this.command = command;
        this.code = code;
        this.cause = cause;
    }
}

/**
 * Heurística para clasificar un error desconocido del runtime Tauri o del
 * backend Rust en uno de los `IpcErrorCode`. Mantenemos esto en un solo
 * lugar para que tanto `errorMappingMiddleware` como tests futuros usen
 * la misma tabla.
 */
export function classifyIpcError(raw: unknown): IpcErrorCode {
    const message = extractMessage(raw).toLowerCase();
    if (!message) return "UNKNOWN";

    if (message.includes("timeout") || message.includes("timed out")) return "TIMEOUT";
    if (message.includes("aborted") || message.includes("cancel")) return "ABORTED";
    if (
        message.includes("not found") ||
        message.includes("no such") ||
        message.includes("does not exist")
    ) {
        return "NOT_FOUND";
    }
    if (
        message.includes("invalid") ||
        message.includes("missing field") ||
        message.includes("expected") ||
        message.includes("deserialization")
    ) {
        return "INVALID_ARGS";
    }
    if (
        message.includes("backend") ||
        message.includes("sidecar") ||
        message.includes("connection refused") ||
        message.includes("not running")
    ) {
        return "BACKEND_DOWN";
    }
    if (message.includes("internal") || message.includes("panic")) return "INTERNAL";
    return "UNKNOWN";
}

function extractMessage(raw: unknown): string {
    if (typeof raw === "string") return raw;
    if (raw instanceof Error) return raw.message;
    if (raw && typeof raw === "object") {
        const candidate = raw as { message?: unknown; toString?: () => string };
        if (typeof candidate.message === "string") return candidate.message;
        try {
            return JSON.stringify(raw);
        } catch {
            return candidate.toString?.() ?? "";
        }
    }
    return String(raw ?? "");
}

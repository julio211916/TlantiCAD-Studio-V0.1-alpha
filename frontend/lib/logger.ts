/**
 * Dev-aware logger wrapper.
 *
 * Bug-fix HIGH#6 — `console.error` / `console.warn` were leaking to
 * production builds in 14+ sites. Route through this thin wrapper so the
 * dev-only output is gated by `import.meta.env.DEV` and a single switch
 * controls verbosity.
 *
 * Production calls become no-ops; intentional production telemetry should
 * call `logger.report()` (currently no-op, future hook for crash reporters).
 */

interface ViteImportMetaEnv {
    readonly DEV?: boolean;
    readonly MODE?: string;
}

interface ViteImportMeta {
    readonly env?: ViteImportMetaEnv;
}

function isDev(): boolean {
    const env = (import.meta as unknown as ViteImportMeta).env;
    return Boolean(env?.DEV);
}

const DEV = isDev();

export const logger = {
    debug(...args: unknown[]): void {
        if (DEV) console.debug(...args);
    },
    info(...args: unknown[]): void {
        if (DEV) console.info(...args);
    },
    warn(...args: unknown[]): void {
        if (DEV) console.warn(...args);
    },
    error(...args: unknown[]): void {
        // Errors stay visible in dev; in prod they become a single-line stub
        // unless a reporter is wired.
        if (DEV) console.error(...args);
    },
    /**
     * Forward a notable error to a future crash-reporter integration.
     * Currently a no-op — wire to Sentry / Posthog in a later sprint.
     */
    report(_error: unknown, _context?: Record<string, unknown>): void {
        // intentionally empty in V172
    },
};

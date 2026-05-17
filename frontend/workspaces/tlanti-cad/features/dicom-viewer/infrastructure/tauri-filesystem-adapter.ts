/**
 * Tauri-backed filesystem adapter for sibling detection and binary reads.
 *
 * On web (no Tauri), both methods degrade gracefully to empty / throwing
 * implementations — the use case still works via `fromFiles` and `fromZip`.
 */

import type { FilesystemSiblingsPort } from '../application/ports';

async function isTauri(): Promise<boolean> {
    if (typeof window === 'undefined') return false;
    // Tauri v2 exposes `__TAURI_INTERNALS__`; v1 exposed `__TAURI__`.
    return Boolean(
        (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ ??
            (window as unknown as { __TAURI__?: unknown }).__TAURI__,
    );
}

export function createTauriFilesystemAdapter(): FilesystemSiblingsPort {
    return {
        async listSiblings(seedAbsolutePath) {
            if (!(await isTauri())) return [];
            const { readDir } = await import('@tauri-apps/plugin-fs');
            const directory = seedAbsolutePath.replace(/[\\/][^\\/]+$/, '');
            if (!directory) return [];
            try {
                const entries = await readDir(directory);
                return entries
                    .filter((entry) => entry.isFile)
                    .map((entry) => `${directory}/${entry.name}`);
            } catch {
                return [];
            }
        },

        async readFileAsArrayBuffer(absolutePath) {
            if (!(await isTauri())) {
                throw new Error('Filesystem access requires the Tauri desktop runtime.');
            }
            const { readFile } = await import('@tauri-apps/plugin-fs');
            const data = await readFile(absolutePath);
            // Tauri returns Uint8Array; convert to plain ArrayBuffer
            const buffer = new ArrayBuffer(data.byteLength);
            new Uint8Array(buffer).set(data);
            return buffer;
        },
    };
}

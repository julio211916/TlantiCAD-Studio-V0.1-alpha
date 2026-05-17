/**
 * TlantiShare Tauri adapter — V276/V277/V278 wired to real Rust commands.
 *
 * Wire protocol: see `src-tauri/src/local_share.rs`. Discovery is mDNS-SD over
 * `_tlantishare._tcp.local.`. File transfer is AES-256-GCM over TCP, key
 * derived via HKDF-SHA256 from a 6-digit pairing PIN displayed on the sender
 * and entered by the receiver. No central server, no presigned URLs.
 *
 * Frontend contract:
 *   - `start(onUpdate)` advertises this station + browses other peers.
 *     Maintains an in-memory peer set and pushes the snapshot on every
 *     `tlantishare://peer-found` / `tlantishare://peer-removed` event.
 *   - `send(peer, bundle, transport, onProgress)` invokes `local_share_send`
 *     with the bundle's `keyB64` interpreted as the 6-digit PIN. Progress is
 *     forwarded from `tlantishare://send-progress`.
 *   - `bluetooth` and `airdrop` raise `transport-not-enabled` until V279
 *     ships the Swift sidecar for AirDrop / btleplug for BLE.
 */

import { isTauriRuntime } from '../../../platform/desktop-system';
import { logger } from '../../../lib/logger';
import type {
    DiscoveryHandle,
    PeerDiscoveryPort,
    SharePort,
} from '../application/peer-port';
import type { Peer, ShareTransport } from '../domain/peer';

export interface LocalShareError {
    kind: 'tauri-runtime-required' | 'transport-not-enabled' | 'rust-error';
    message: string;
}

export interface TlantiShareAdvertiseInfo {
    deviceId: string;
    port: number;
    alias: string;
    fingerprint: string;
}

export interface TlantiShareIncomingRequest {
    transferId: string;
    fromAlias: string;
    fromAddress: string;
    files: Array<{ name: string; size: number; sha256: string }>;
}

const DEFAULT_PORT = 53318;

export function createTauriDiscoveryAdapter(port: number = DEFAULT_PORT): PeerDiscoveryPort {
    return {
        start(onUpdate) {
            if (!isTauriRuntime()) {
                onUpdate([]);
                return { stop: () => undefined };
            }

            const peers = new Map<string, Peer>();
            let cancelled = false;
            let unlistenFound: (() => void) | null = null;
            let unlistenRemoved: (() => void) | null = null;

            const handle: DiscoveryHandle = {
                stop: () => {
                    cancelled = true;
                    unlistenFound?.();
                    unlistenRemoved?.();
                    void (async () => {
                        try {
                            const { invoke } = await import('@tauri-apps/api/core');
                            await invoke('local_share_browse_stop');
                            await invoke('local_share_stop_advertising');
                        } catch (err) {
                            logger.warn('tlantishare stop failed', err);
                        }
                    })();
                },
            };

            (async () => {
                try {
                    const { invoke } = await import('@tauri-apps/api/core');
                    const { listen } = await import('@tauri-apps/api/event');

                    unlistenFound = await listen<Peer>('tlantishare://peer-found', (evt) => {
                        if (cancelled) return;
                        peers.set(evt.payload.id, evt.payload);
                        onUpdate(Array.from(peers.values()));
                    });
                    unlistenRemoved = await listen<string>(
                        'tlantishare://peer-removed',
                        (evt) => {
                            if (cancelled) return;
                            // payload is the mDNS fullname (alias._tlantishare._tcp.local.).
                            // Match by name prefix since we keyed by device_id.
                            const prefix = evt.payload.split('.')[0];
                            for (const [id, peer] of peers) {
                                if (peer.name === prefix) {
                                    peers.delete(id);
                                }
                            }
                            onUpdate(Array.from(peers.values()));
                        },
                    );

                    await invoke('local_share_advertise', { port, alias: null });
                    await invoke('local_share_browse_start');
                } catch (err) {
                    logger.warn('tlantishare discovery unavailable', err);
                    if (!cancelled) onUpdate([]);
                }
            })();

            return handle;
        },
        localTransports(): ShareTransport[] {
            const platform =
                typeof navigator !== 'undefined' ? navigator.platform.toLowerCase() : '';
            const isMac = platform.includes('mac');
            return isMac ? ['airdrop', 'bluetooth', 'lan'] : ['bluetooth', 'lan'];
        },
    };
}

export function createTauriShareAdapter(): SharePort {
    return {
        async send(peer, bundle, transport, onProgress) {
            if (!isTauriRuntime()) {
                const err: LocalShareError = {
                    kind: 'tauri-runtime-required',
                    message:
                        'Local sharing only works in the desktop app. Open TlantiCAD Studio outside the browser preview.',
                };
                throw err;
            }
            if (transport === 'bluetooth' || transport === 'airdrop') {
                const err: LocalShareError = {
                    kind: 'transport-not-enabled',
                    message: `${transport} transport pending — V279 sidecar not yet bundled. Use 'lan' on Wi-Fi instead.`,
                };
                throw err;
            }
            if (!peer.address) {
                throw {
                    kind: 'rust-error',
                    message: 'peer has no address — discovery did not resolve it',
                } as LocalShareError;
            }
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const { listen } = await import('@tauri-apps/api/event');

                const filePaths = bundle.includeAssets?.length
                    ? bundle.includeAssets.map((name) => `${bundle.caseFolderPath}/${name}`)
                    : [bundle.caseFolderPath];

                let transferId: string | null = null;
                const unlisten = await listen<{
                    transferId: string;
                    bytesSent: number;
                    totalBytes: number;
                    currentFile: string;
                }>('tlantishare://send-progress', (evt) => {
                    if (transferId && evt.payload.transferId !== transferId) return;
                    onProgress?.({
                        transport,
                        bytesSent: evt.payload.bytesSent,
                        totalBytes: evt.payload.totalBytes,
                        percent: evt.payload.totalBytes
                            ? Math.round((evt.payload.bytesSent * 100) / evt.payload.totalBytes)
                            : 0,
                    });
                });

                try {
                    const result = await invoke<{ transferId: string; totalBytes: number }>(
                        'local_share_send',
                        {
                            input: {
                                peerAddress: peer.address,
                                filePaths,
                                pin: bundle.keyB64,
                                alias: null,
                            },
                        },
                    );
                    transferId = result.transferId;
                    onProgress?.({
                        transport,
                        bytesSent: result.totalBytes,
                        totalBytes: result.totalBytes,
                        percent: 100,
                    });
                } finally {
                    unlisten();
                }
            } catch (err) {
                if (
                    typeof err === 'object' &&
                    err !== null &&
                    'kind' in err &&
                    'message' in err
                ) {
                    throw err;
                }
                const message = err instanceof Error ? err.message : String(err);
                logger.warn('local_share_send failed', err);
                const wrapped: LocalShareError = {
                    kind: message.includes('not implemented')
                        ? 'transport-not-enabled'
                        : 'rust-error',
                    message,
                };
                throw wrapped;
            }
        },
    };
}

/**
 * Receiver side — start a TCP listener. The UI calls this on launch (or when
 * the user opens the Share panel). When a peer dials in, a `tlantishare://incoming-request`
 * event fires and the UI shows the accept dialog.
 */
export async function startTlantiShareListener(
    stagingDir: string,
    port: number = DEFAULT_PORT,
): Promise<void> {
    if (!isTauriRuntime()) return;
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('local_share_listen', { port, stagingDir });
}

export async function stopTlantiShareListener(): Promise<void> {
    if (!isTauriRuntime()) return;
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('local_share_stop_listening');
}

export async function acceptTlantiShareTransfer(
    transferId: string,
    pin: string,
): Promise<void> {
    if (!isTauriRuntime()) return;
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('local_share_accept', { transferId, pin });
}

export async function rejectTlantiShareTransfer(transferId: string): Promise<void> {
    if (!isTauriRuntime()) return;
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('local_share_reject', { transferId });
}

export async function generateTlantiSharePin(): Promise<string> {
    if (!isTauriRuntime()) {
        // Web preview fallback — uses crypto.getRandomValues().
        const buf = new Uint32Array(1);
        crypto.getRandomValues(buf);
        return String(buf[0] % 1_000_000).padStart(6, '0');
    }
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<string>('local_share_generate_pin');
}

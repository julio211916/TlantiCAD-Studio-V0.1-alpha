/**
 * Local-share domain — V46.
 *
 * Device-to-device case sharing — no cloud servers, no presigned URLs, no
 * external auth. Three transports:
 *   - 'airdrop'   → macOS only, hands the bundle to NSItemProvider via the
 *                   Tauri sidecar (`local_share::airdrop_share` command).
 *   - 'bluetooth' → cross-platform, uses BLE/RFCOMM file transfer.
 *   - 'lan'       → Bonjour / mDNS peer discovery + direct TCP transfer
 *                   on the same LAN; AES-GCM encrypted with a QR-code key.
 *
 * Each peer carries its capability flags so the UI can grey out unsupported
 * transports per peer.
 */

export type ShareTransport = 'airdrop' | 'bluetooth' | 'lan';

export type PeerStatus = 'idle' | 'sending' | 'success' | 'error';

export interface Peer {
    /** Stable per-session id (uuid). */
    id: string;
    /** Display name (hostname / device alias / user-chosen handle). */
    name: string;
    /** Human-readable platform label ("macOS · Mac Studio", "Windows 11 PC"). */
    platform: string;
    /** Transports this peer reports support for. */
    transports: ShareTransport[];
    /**
     * `lan` peers only — IP + port; `bluetooth` may carry a MAC; `airdrop`
     * carries a system-handle hint. Opaque to the UI.
     */
    address?: string;
    /** Last-seen timestamp (ms). Used to gray-out stale peers. */
    lastSeenAt: number;
    status: PeerStatus;
    /** Filled when status === 'error'. */
    errorMessage?: string;
}

export interface ShareBundle {
    /** Source case folder (`<case-id>.tlanticase/` path). */
    caseFolderPath: string;
    /** Optional subset of asset filenames to include. Empty = full case. */
    includeAssets?: string[];
    /**
     * AES-GCM key (base64) shared out-of-band — usually shown as a QR
     * the receiver scans. NEVER transmitted over the wire alongside the
     * payload.
     */
    keyB64: string;
}

export interface ShareProgressEvent {
    transport: ShareTransport;
    bytesSent: number;
    totalBytes: number;
    percent: number;
}

export interface LocalShareState {
    peers: Peer[];
    isDiscovering: boolean;
    activeTransport: ShareTransport;
    error: string | null;
    /** Last successfully shared peer.id — so the UI can show "Sent ✓" badges. */
    recentSuccesses: string[];
}

export function defaultLocalShareState(): LocalShareState {
    return {
        peers: [],
        isDiscovering: false,
        activeTransport: 'lan',
        error: null,
        recentSuccesses: [],
    };
}

export function freshPeer(name: string, platform: string, transports: ShareTransport[]): Peer {
    return {
        id: crypto.randomUUID(),
        name,
        platform,
        transports,
        lastSeenAt: Date.now(),
        status: 'idle',
    };
}

export function isStale(peer: Peer, ttlMs = 30_000, now = Date.now()): boolean {
    return now - peer.lastSeenAt > ttlMs;
}

/**
 * Returns the transports both `peer` AND the active station support.
 * `lan` works between any two stations on the same Wi-Fi; `airdrop` only
 * between macOS pairs; `bluetooth` only when both have BT enabled.
 */
export function intersectTransports(
    peer: Peer,
    localTransports: readonly ShareTransport[],
): ShareTransport[] {
    const local = new Set(localTransports);
    return peer.transports.filter((t) => local.has(t));
}

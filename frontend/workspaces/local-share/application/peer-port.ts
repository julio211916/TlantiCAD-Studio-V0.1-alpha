/**
 * Peer discovery + share port (V46).
 *
 * Three transports MUST be implementable behind this single interface so the
 * UI never branches per transport. Each transport's adapter lives in
 * `infrastructure/<transport>-transport.ts`.
 */

import type {
    Peer,
    ShareBundle,
    ShareProgressEvent,
    ShareTransport,
} from '../domain/peer';

export interface DiscoveryHandle {
    /** Stop discovery and release any sockets / watchers. */
    stop(): void;
}

export interface PeerDiscoveryPort {
    /** Begin advertising + scanning for peers. Calls `onUpdate` on every change. */
    start(onUpdate: (peers: Peer[]) => void): DiscoveryHandle;
    /** Transports this station can act as a sender on. */
    localTransports(): ShareTransport[];
}

export interface SharePort {
    send(
        peer: Peer,
        bundle: ShareBundle,
        transport: ShareTransport,
        onProgress?: (event: ShareProgressEvent) => void,
    ): Promise<void>;
}

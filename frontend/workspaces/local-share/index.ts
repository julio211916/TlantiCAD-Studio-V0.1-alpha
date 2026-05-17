export type {
    LocalShareState,
    Peer,
    PeerStatus,
    ShareBundle,
    ShareProgressEvent,
    ShareTransport,
} from './domain/peer';
export {
    defaultLocalShareState,
    freshPeer,
    intersectTransports,
    isStale,
} from './domain/peer';

export type {
    DiscoveryHandle,
    PeerDiscoveryPort,
    SharePort,
} from './application/peer-port';
export {
    acceptTlantiShareTransfer,
    createTauriDiscoveryAdapter,
    createTauriShareAdapter,
    generateTlantiSharePin,
    rejectTlantiShareTransfer,
    startTlantiShareListener,
    stopTlantiShareListener,
} from './infrastructure/tauri-share-adapter';
export type {
    LocalShareError,
    TlantiShareAdvertiseInfo,
    TlantiShareIncomingRequest,
} from './infrastructure/tauri-share-adapter';

export { LocalSharePanel } from './ui/LocalSharePanel';
export type { LocalSharePanelProps } from './ui/LocalSharePanel';

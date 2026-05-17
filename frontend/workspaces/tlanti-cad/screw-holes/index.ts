export type {
    ScrewChannelMode,
    ScrewHolesParams,
    ScrewHoleToothState,
} from './domain/screw-holes';
export {
    applyGlobalMode,
    clampHeightMm,
    clampMinDiameterMm,
    clampOffsetMm,
    clampThicknessMm,
    cycleScrewMode,
    defaultScrewHolesParams,
    defaultToothState,
    flattenChannelTop,
    setChannelThickness,
    summarizeScrewHoles,
    toggleLocked,
} from './domain/screw-holes';

export type {
    ScrewHolesApplyInput,
    ScrewHolesApplyOutput,
    ScrewHolesPort,
} from './application/screw-holes-port';
export { createBackendScrewHolesAdapter } from './infrastructure/backend-screw-holes-adapter';

export { ScrewHolesPanel } from './ui/ScrewHolesPanel';
export type { ScrewHolesPanelProps } from './ui/ScrewHolesPanel';
export { ScrewChannelOverlay } from './ui/ScrewChannelOverlay';
export type { ChannelAnchor, ScrewChannelOverlayProps } from './ui/ScrewChannelOverlay';

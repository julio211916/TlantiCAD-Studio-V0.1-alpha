export type {
    CopyStrategy,
    Mat4,
    PreopAdaptResult,
    PreopAlignment,
    PreopWaxupState,
    ToothCopyDecision,
    WaxupPreparation,
} from './domain/preop-waxup';
export {
    initialPreopWaxupState,
    validateWaxupInput,
} from './domain/preop-waxup';

export type {
    PreopAdaptInput,
    PreopAlignInput,
    PreopWaxupPort,
    WaxupPrepareInput,
} from './application/preop-waxup-port';
export { createBackendPreopWaxupAdapter } from './infrastructure/backend-preop-waxup-adapter';
export { createTauriPreopWaxupAdapter } from './infrastructure/tauri-preop-waxup-adapter';

export { PreopWaxupPanel } from './ui/PreopWaxupPanel';
export type { PreopWaxupPanelProps } from './ui/PreopWaxupPanel';

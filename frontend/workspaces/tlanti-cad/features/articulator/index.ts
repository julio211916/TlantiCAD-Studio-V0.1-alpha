export type {
    ArticulatorConfig,
    JawFrame,
    JawMovement,
} from './domain/articulator-config';
export { defaultArticulatorConfig, frameDistanceMm } from './domain/articulator-config';

export type {
    ArticulatorPort,
    ArticulatorSimulateInput,
    ArticulatorSimulateOutput,
} from './application/articulator-port';
export { createBackendArticulatorAdapter } from './infrastructure/backend-articulator-adapter';

export { ArticulatorPanel } from './ui/ArticulatorPanel';
export type { ArticulatorPanelProps } from './ui/ArticulatorPanel';
export { ArticulatorContainer } from './ui/ArticulatorContainer';
export type { ArticulatorContainerProps } from './ui/ArticulatorContainer';
export { JawMotionOverlay } from './ui/JawMotionOverlay';
export type { JawMotionOverlayProps } from './ui/JawMotionOverlay';
export { InfluencingTeethDialog } from './ui/InfluencingTeethDialog';
export type { InfluencingTeethDialogProps } from './ui/InfluencingTeethDialog';

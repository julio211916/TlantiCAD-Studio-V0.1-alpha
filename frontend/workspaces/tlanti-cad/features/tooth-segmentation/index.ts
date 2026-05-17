/**
 * Public surface of the tooth-segmentation feature (Crown Segmentation
 * workflow — RealGUIDE reference).
 */

export type {
    JawKind,
    ToothCategory,
    ToothDefinition,
    ToothState,
    ToothStatus,
} from './domain/fdi-chart';
export {
    DEFAULT_TOOTH_PALETTE,
    PERMANENT_TEETH,
    defaultColorFor,
    findTooth,
    teethOfJaw,
} from './domain/fdi-chart';

export type {
    CrownSegJob,
    CrownSegJobStatus,
    CrownSegmentationLaunchArgs,
    ToothSegmentationPort,
} from './application/tooth-segmentation-port';

export { createBackendToothSegmentationAdapter } from './infrastructure/backend-tooth-segmentation-adapter';

export { ToothChart } from './ui/ToothChart';
export { CrownSegmentationPanel } from './ui/CrownSegmentationPanel';
export { useCrownSegmentation } from './ui/useCrownSegmentation';

import type {
    PreopAdaptResult,
    PreopAlignment,
    WaxupPreparation,
} from '../domain/preop-waxup';

export interface PreopAlignInput {
    preopPath: string;
    modelPath: string;
    initialTranslationMm?: [number, number, number];
}

export interface PreopAdaptInput {
    preopPath: string;
    toothPaths: string[];
    iterations: number;
}

export interface WaxupPrepareInput {
    waxupPath: string;
    marginPolylinePerTooth: Record<string, [number, number, number][]>;
    cropAboveMargin: boolean;
    closeHoles: boolean;
}

export interface PreopWaxupPort {
    alignPreop(input: PreopAlignInput): Promise<PreopAlignment>;
    adaptToPreop(input: PreopAdaptInput): Promise<PreopAdaptResult>;
    prepareWaxup(input: WaxupPrepareInput): Promise<WaxupPreparation>;
}

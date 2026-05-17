/**
 * Public surface of the DICOM viewer feature.
 *
 * Only `ui/` components and the app shell should import from here; everything
 * deeper is internal implementation detail of the hexagonal module.
 */

export type {
    DicomImportResult,
    DicomImportWarning,
    DicomInstance,
    DicomPatientInfo,
    DicomSeries,
    DicomStudy,
    Modality,
} from './domain/dicom-study';

export type {
    CtPreset,
    MriPreset,
    PresetKind,
    VolumePreset,
} from './domain/volume-preset';

export {
    CT_PRESETS,
    MRI_PRESETS,
    findPresetById,
    getPresetsForModality,
} from './domain/volume-preset';

export type { ViewerMode, ViewerModeCapability } from './domain/viewer-mode';
export {
    VIEWER_MODE_CAPABILITIES,
    resolveAvailableModes,
} from './domain/viewer-mode';

export type {
    ClinicalModuleId,
    ContextMenuItem,
    ContextMenuProvider,
    ContextMenuRegistry,
    ContextObjectKind,
    ContextSource,
} from './domain/contextual-menu-contract';
export { createContextMenuRegistry } from './domain/contextual-menu-contract';

export type { ImportSeriesUseCase } from './application/import-series.use-case';
export { createImportSeriesUseCase } from './application/import-series.use-case';

export type {
    SegmentationBackendPort,
    SegmentationJob,
    SegmentationJobStatus,
    SegmentationLabel,
    SegmentStudyUseCase,
} from './application/segment-study.use-case';
export { createSegmentStudyUseCase } from './application/segment-study.use-case';

export { DicomSegmentationOverlay } from './ui/DicomSegmentationOverlay';
export { DicomAiSegmentationButton } from './ui/DicomAiSegmentationButton';
export { useSegmentationRunner } from './ui/useSegmentationRunner';
export { DicomClinicalWorkspace } from './ui/DicomClinicalWorkspace';

/**
 * Composition root: wire the default adapters into a ready-to-use
 * ImportSeriesUseCase. Tests should compose their own with stub ports.
 */
export function createDefaultImportSeriesUseCase() {
    // Adapters are imported lazily to keep the bundle graph small for callers
    // that don't need the use case (e.g. the smile-design module).
    return Promise.all([
        import('./infrastructure/dicom-parser-adapter'),
        import('./infrastructure/cornerstone-image-id-adapter'),
        import('./infrastructure/tauri-filesystem-adapter'),
        import('./infrastructure/jszip-adapter'),
        import('./application/import-series.use-case'),
    ]).then(([parserMod, imageIdMod, fsMod, zipMod, useCaseMod]) =>
        useCaseMod.createImportSeriesUseCase({
            parser: parserMod.createDicomHeaderParserAdapter(),
            imageIdFactory: imageIdMod.createCornerstoneImageIdFactory(),
            filesystem: fsMod.createTauriFilesystemAdapter(),
            zip: zipMod.createJszipAdapter(),
        }),
    );
}

import { lazy } from 'react';

/**
 * Lazy-loaded panels used by the TlantiDB shell. Centralised so the main
 * component module stays compact and so every panel comes through a single
 * `React.lazy` boundary that the shell knows how to wrap with suspense.
 */
export const InteractiveOdontogram = lazy(() =>
    import('@/components/tlantidb/InteractiveOdontogram').then((m) => ({ default: m.InteractiveOdontogram })),
);

export const PublicAssetLibraryBrowser = lazy(() =>
    import('@/components/asset-library/PublicAssetLibraryBrowser').then((m) => ({ default: m.PublicAssetLibraryBrowser })),
);

export const TlantiDbActionPanel = lazy(() =>
    import('@/components/tlantidb/TlantiDbActionPanel').then((m) => ({ default: m.TlantiDbActionPanel })),
);

export const TlantiDbCaseBrowserSheet = lazy(() =>
    import('@/components/tlantidb/TlantiDbCaseBrowserSheet').then((m) => ({ default: m.TlantiDbCaseBrowserSheet })),
);

export const TlantiDbWorkloadWizard = lazy(() =>
    import('@/components/tlantidb/TlantiDbWorkloadWizard').then((m) => ({ default: m.TlantiDbWorkloadWizard })),
);

export const TlantiDbClinicalAssetsPanel = lazy(() =>
    import('@/components/tlantidb/TlantiDbClinicalAssetsPanel').then((m) => ({ default: m.TlantiDbClinicalAssetsPanel })),
);

export const TlantiDbSidebar = lazy(() =>
    import('@/components/tlantidb/TlantiDbSidebar').then((m) => ({ default: m.TlantiDbSidebar })),
);

export const SystemRuntimeSettingsPanel = lazy(() =>
    import('@/components/tlantidb/SystemRuntimeSettingsPanel').then((m) => ({ default: m.SystemRuntimeSettingsPanel })),
);

export const BackendIntegrationPanel = lazy(() =>
    import('@/components/tlantidb/BackendIntegrationPanel').then((m) => ({ default: m.BackendIntegrationPanel })),
);

export const BackendWorkspacePanel = lazy(() =>
    import('@/components/tlantidb/BackendWorkspacePanel').then((m) => ({ default: m.BackendWorkspacePanel })),
);

export const DicomDentalRoadmapPanel = lazy(() =>
    import('@/components/tlantidb/DicomDentalRoadmapPanel').then((m) => ({ default: m.DicomDentalRoadmapPanel })),
);

export const DicomDentalExecutionPanel = lazy(() =>
    import('@/components/tlantidb/DicomDentalExecutionPanel').then((m) => ({ default: m.DicomDentalExecutionPanel })),
);

export const PlatformToolkitPanel = lazy(() =>
    import('@/components/tlantidb/PlatformToolkitPanel').then((m) => ({ default: m.PlatformToolkitPanel })),
);

export const ClinicalQualityInteropPanel = lazy(() =>
    import('@/components/tlantidb/ClinicalQualityInteropPanel').then((m) => ({ default: m.ClinicalQualityInteropPanel })),
);

export const CollaborationAutoplanningPanel = lazy(() =>
    import('@/components/tlantidb/CollaborationAutoplanningPanel').then((m) => ({ default: m.CollaborationAutoplanningPanel })),
);

export const HybridOpsPrecisionPanel = lazy(() =>
    import('@/components/tlantidb/HybridOpsPrecisionPanel').then((m) => ({ default: m.HybridOpsPrecisionPanel })),
);

export const ToothWorkflowSheet = lazy(() =>
    import('@/components/tlantidb/ToothWorkflowSheet').then((m) => ({ default: m.ToothWorkflowSheet })),
);

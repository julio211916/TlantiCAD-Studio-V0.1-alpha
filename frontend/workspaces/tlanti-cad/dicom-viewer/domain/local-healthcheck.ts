import type { DicomImportResult, DicomStudy } from './dicom-study';

export type LocalDicomCapabilityState = 'ready' | 'degraded' | 'unavailable';

export interface LocalDicomCapability {
    id: string;
    label: string;
    state: LocalDicomCapabilityState;
    offlineOnly: true;
    detail: string;
    requiredFor: string[];
}

export interface LocalDicomHealthCheck {
    scope: 'dicom-local-pipeline';
    schemaVersion: 1;
    checkedAt: number;
    offlineOnly: true;
    overall: LocalDicomCapabilityState;
    capabilities: LocalDicomCapability[];
    blockers: string[];
    performanceBudget: {
        importedInstances: number;
        volumetricSeries: number;
        chunkedReadRequired: boolean;
        maxRecommendedBrowserBytes: number;
    };
}

function aggregateCapabilityState(capabilities: LocalDicomCapability[]): LocalDicomCapabilityState {
    if (capabilities.some((capability) => capability.state === 'unavailable')) {
        return 'unavailable';
    }
    if (capabilities.some((capability) => capability.state === 'degraded')) {
        return 'degraded';
    }
    return 'ready';
}

function countInstances(studies: DicomStudy[]): number {
    return studies.reduce(
        (total, study) =>
            total + study.series.reduce((seriesTotal, series) => seriesTotal + series.instanceCount, 0),
        0,
    );
}

function countVolumetricSeries(studies: DicomStudy[]): number {
    return studies.reduce(
        (total, study) =>
            total + study.series.filter((series) => series.isVolumetric && !series.hasSpacingGaps).length,
        0,
    );
}

export function createLocalDicomHealthCheck(
    result: DicomImportResult | null,
    options?: {
        hasTauriFilesystem?: boolean;
        hasZipSupport?: boolean;
        maxRecommendedBrowserBytes?: number;
    },
): LocalDicomHealthCheck {
    const importedInstances = result ? countInstances(result.studies) : 0;
    const volumetricSeries = result ? countVolumetricSeries(result.studies) : 0;
    const hasWarnings = Boolean(result?.warnings.length);
    const maxRecommendedBrowserBytes = options?.maxRecommendedBrowserBytes ?? 512 * 1024 * 1024;

    const capabilities: LocalDicomCapability[] = [
        {
            id: 'local-header-parser',
            label: 'Local DICOM header parser',
            state: result === null ? 'degraded' : importedInstances > 0 ? 'ready' : 'unavailable',
            offlineOnly: true,
            detail:
                result === null
                    ? 'Parser has not run yet.'
                    : `${importedInstances} local DICOM instances parsed without remote services.`,
            requiredFor: ['Import', 'Clean', 'Validate'],
        },
        {
            id: 'desktop-filesystem',
            label: 'Desktop filesystem import',
            state: options?.hasTauriFilesystem ? 'ready' : 'degraded',
            offlineOnly: true,
            detail: options?.hasTauriFilesystem
                ? 'Tauri filesystem adapter is available for folder/sibling detection.'
                : 'Browser file input is available; large CBCT folders should use Tauri chunked import.',
            requiredFor: ['DICOM folder import', 'large CBCT import'],
        },
        {
            id: 'zip-expansion',
            label: 'Offline ZIP expansion',
            state: options?.hasZipSupport === false ? 'unavailable' : 'ready',
            offlineOnly: true,
            detail:
                options?.hasZipSupport === false
                    ? 'ZIP expansion adapter is not wired.'
                    : 'ZIP archives are expanded in-process without network upload.',
            requiredFor: ['PACS export import', 'portable case transfer'],
        },
        {
            id: 'volumetric-series',
            label: 'Volumetric series readiness',
            state: volumetricSeries > 0 ? (hasWarnings ? 'degraded' : 'ready') : 'unavailable',
            offlineOnly: true,
            detail:
                volumetricSeries > 0
                    ? `${volumetricSeries} volumetric series ready for segmentation.`
                    : 'No consistent volumetric CT/CBCT series detected.',
            requiredFor: ['Segment', 'Design', 'Validate'],
        },
    ];
    const blockers = capabilities
        .filter((capability) => capability.state === 'unavailable')
        .map((capability) => capability.detail);

    return {
        scope: 'dicom-local-pipeline',
        schemaVersion: 1,
        checkedAt: Date.now(),
        offlineOnly: true,
        overall: aggregateCapabilityState(capabilities),
        capabilities,
        blockers,
        performanceBudget: {
            importedInstances,
            volumetricSeries,
            chunkedReadRequired: importedInstances > 512,
            maxRecommendedBrowserBytes,
        },
    };
}

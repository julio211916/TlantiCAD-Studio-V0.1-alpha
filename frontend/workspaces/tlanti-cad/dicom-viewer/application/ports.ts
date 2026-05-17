/**
 * Ports (interfaces) that application use-cases depend on.
 * Implementations live under `infrastructure/`.
 */

import type { DicomInstance, DicomStudy } from '../domain/dicom-study';

/** Parsed tag set from a single DICOM file, enough to group and sort. */
export interface ParsedDicomHeaders {
    sopInstanceUID: string;
    studyInstanceUID: string;
    seriesInstanceUID: string;
    studyId: string | null;
    studyDate: string | null;
    studyDescription: string | null;
    accessionNumber: string | null;
    seriesNumber: number | null;
    seriesDescription: string | null;
    modality: string;
    instanceNumber: number | null;
    sliceLocation: number | null;
    imagePositionPatient: [number, number, number] | null;
    pixelSpacing: [number, number] | null;
    sliceThickness: number | null;
    patient: {
        patientId: string;
        patientName: string;
        birthDate: string | null;
        sex: string | null;
    };
}

/**
 * Parses DICOM binary headers without decoding pixel data.
 * Implementation: `dicom-parser` (browser) or pydicom (backend).
 */
export interface DicomHeaderParserPort {
    parse(bytes: ArrayBuffer, sourceFilePath: string | null): Promise<ParsedDicomHeaders | null>;
}

/** Creates cornerstone `imageId`s from File/Blob objects. */
export interface ImageIdFactoryPort {
    fromFile(file: File): string;
    fromPath(absolutePath: string): string;
}

/**
 * Lists sibling files for a seed path. Tauri implementation uses
 * `@tauri-apps/plugin-fs readDir`. Web implementation returns an empty list.
 */
export interface FilesystemSiblingsPort {
    listSiblings(seedAbsolutePath: string): Promise<string[]>;
    readFileAsArrayBuffer(absolutePath: string): Promise<ArrayBuffer>;
}

/** Expands a ZIP archive to a flat list of File objects. */
export interface ZipArchivePort {
    expand(archive: File): Promise<File[]>;
}

/** Instance sort strategy used when grouping a series. */
export function compareInstances(a: DicomInstance, b: DicomInstance): number {
    // Prefer ImagePositionPatient Z when both have it
    const az = a.imagePositionPatient?.[2];
    const bz = b.imagePositionPatient?.[2];
    if (typeof az === 'number' && typeof bz === 'number' && az !== bz) return az - bz;
    // Fall back to sliceLocation
    if (typeof a.sliceLocation === 'number' && typeof b.sliceLocation === 'number' && a.sliceLocation !== b.sliceLocation) {
        return a.sliceLocation - b.sliceLocation;
    }
    // Final tiebreaker: instanceNumber
    const ai = a.instanceNumber ?? 0;
    const bi = b.instanceNumber ?? 0;
    return ai - bi;
}

/** Compute derived series-level flags after sorting. */
export function computeSeriesGeometry(sorted: DicomInstance[]): {
    sliceSpacingMm: number | null;
    isVolumetric: boolean;
    hasSpacingGaps: boolean;
} {
    if (sorted.length < 2) {
        return { sliceSpacingMm: null, isVolumetric: false, hasSpacingGaps: false };
    }
    const gaps: number[] = [];
    for (let i = 1; i < sorted.length; i += 1) {
        const prev = sorted[i - 1];
        const cur = sorted[i];
        const pz = prev.imagePositionPatient?.[2] ?? prev.sliceLocation;
        const cz = cur.imagePositionPatient?.[2] ?? cur.sliceLocation;
        if (typeof pz !== 'number' || typeof cz !== 'number') continue;
        gaps.push(Math.abs(cz - pz));
    }
    if (gaps.length === 0) {
        return { sliceSpacingMm: null, isVolumetric: false, hasSpacingGaps: false };
    }
    const mean = gaps.reduce((acc, g) => acc + g, 0) / gaps.length;
    const variance =
        gaps.reduce((acc, g) => acc + (g - mean) * (g - mean), 0) / gaps.length;
    const stddev = Math.sqrt(variance);
    // If stddev is more than 5% of the mean spacing we flag it.
    const hasGaps = mean > 0 && stddev / mean > 0.05;
    return {
        sliceSpacingMm: mean > 0 ? mean : null,
        isVolumetric: mean > 0 && !hasGaps,
        hasSpacingGaps: hasGaps,
    };
}

export function toStudyMap(headers: ParsedDicomHeaders[], imageIdFor: (h: ParsedDicomHeaders) => string): Map<string, DicomStudy> {
    const studies = new Map<string, DicomStudy>();
    const seriesMap = new Map<string, DicomInstance[]>();

    for (const h of headers) {
        if (!studies.has(h.studyInstanceUID)) {
            studies.set(h.studyInstanceUID, {
                studyInstanceUID: h.studyInstanceUID,
                studyId: h.studyId,
                studyDescription: h.studyDescription,
                studyDate: h.studyDate,
                accessionNumber: h.accessionNumber,
                patient: {
                    patientId: h.patient.patientId,
                    patientName: h.patient.patientName,
                    birthDate: h.patient.birthDate,
                    sex:
                        h.patient.sex === 'M' || h.patient.sex === 'F' || h.patient.sex === 'O'
                            ? (h.patient.sex as 'M' | 'F' | 'O')
                            : null,
                },
                series: [],
            });
        }
        const key = `${h.studyInstanceUID}::${h.seriesInstanceUID}`;
        const bucket = seriesMap.get(key) ?? [];
        bucket.push({
            sopInstanceUID: h.sopInstanceUID,
            instanceNumber: h.instanceNumber,
            sourceFilePath: null,
            imageId: imageIdFor(h),
            sliceLocation: h.sliceLocation,
            imagePositionPatient: h.imagePositionPatient,
            bytes: null,
        });
        seriesMap.set(key, bucket);
    }

    for (const [key, instances] of seriesMap.entries()) {
        const [studyUid, seriesUid] = key.split('::');
        const study = studies.get(studyUid);
        if (!study) continue;
        const sorted = instances.slice().sort(compareInstances);
        const geometry = computeSeriesGeometry(sorted);
        const firstHeader = headers.find((h) => h.seriesInstanceUID === seriesUid)!;
        const modality = normalizeModality(firstHeader.modality);
        study.series.push({
            seriesInstanceUID: seriesUid,
            seriesNumber: firstHeader.seriesNumber,
            seriesDescription: firstHeader.seriesDescription,
            modality,
            instanceCount: sorted.length,
            instances: sorted,
            sliceSpacingMm: geometry.sliceSpacingMm,
            pixelSpacingMm: firstHeader.pixelSpacing,
            isVolumetric: geometry.isVolumetric,
            hasSpacingGaps: geometry.hasSpacingGaps,
        });
    }

    // Sort series within each study by seriesNumber
    for (const study of studies.values()) {
        study.series.sort((a, b) => (a.seriesNumber ?? 0) - (b.seriesNumber ?? 0));
    }

    return studies;
}

function normalizeModality(raw: string): import('../domain/dicom-study').Modality {
    const upper = (raw ?? '').toUpperCase().trim();
    switch (upper) {
        case 'CT':
            return 'CT';
        case 'MR':
            return 'MR';
        case 'CR':
            return 'CR';
        case 'DX':
            return 'DX';
        case 'PX':
        case 'PAN':
            return 'PX';
        case 'IO':
            return 'IO';
        case 'SEG':
            return 'SEG';
        case 'RTSTRUCT':
        case 'RTDOSE':
        case 'RTPLAN':
            return 'RT';
        default:
            return 'OTHER';
    }
}

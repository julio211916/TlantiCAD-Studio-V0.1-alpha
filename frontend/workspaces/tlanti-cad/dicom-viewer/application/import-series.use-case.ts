/**
 * ImportSeriesUseCase — unified entry point to turn arbitrary clinical input
 * (individual .dcm files, folders, zip archives) into grouped `DicomStudy[]`.
 *
 * Pure TypeScript, no rendering. Depends only on ports — swap adapters for
 * tests (fake parser, fake fs, in-memory zip).
 */

import type {
    DicomImportResult,
    DicomImportWarning,
    DicomStudy,
} from '../domain/dicom-study';
import {
    computeSeriesGeometry,
    toStudyMap,
    type DicomHeaderParserPort,
    type FilesystemSiblingsPort,
    type ImageIdFactoryPort,
    type ParsedDicomHeaders,
    type ZipArchivePort,
} from './ports';

export interface ImportSeriesDependencies {
    parser: DicomHeaderParserPort;
    imageIdFactory: ImageIdFactoryPort;
    filesystem: FilesystemSiblingsPort;
    zip: ZipArchivePort;
}

export interface ImportSeriesUseCase {
    fromFiles(files: File[]): Promise<DicomImportResult>;
    fromZip(archive: File): Promise<DicomImportResult>;
    /**
     * Given a single `.dcm` absolute path, inspect its folder and load every
     * sibling that shares the same `SeriesInstanceUID`. Tauri-only.
     */
    detectSiblings(seedAbsolutePath: string): Promise<DicomImportResult>;
}

function isDicomLike(filename: string): boolean {
    const lower = filename.toLowerCase();
    return (
        lower.endsWith('.dcm') ||
        lower.endsWith('.dicom') ||
        lower.endsWith('.ima') ||
        // Some PACS/OEMs drop the extension entirely. Parser will confirm.
        !/\.[a-z0-9]{2,5}$/i.test(lower)
    );
}

export function createImportSeriesUseCase(
    deps: ImportSeriesDependencies,
): ImportSeriesUseCase {
    const { parser, imageIdFactory, filesystem, zip } = deps;

    async function parseBatch(
        items: Array<{ file: File; absolutePath: string | null }>,
    ): Promise<{
        headers: ParsedDicomHeaders[];
        imageIdByUid: Map<string, string>;
        warnings: DicomImportWarning[];
        skippedCount: number;
    }> {
        const headers: ParsedDicomHeaders[] = [];
        const imageIdByUid = new Map<string, string>();
        const warnings: DicomImportWarning[] = [];
        let skippedCount = 0;

        for (const { file, absolutePath } of items) {
            if (!isDicomLike(file.name)) {
                skippedCount += 1;
                continue;
            }
            try {
                const bytes = await file.arrayBuffer();
                const parsed = await parser.parse(bytes, absolutePath);
                if (!parsed) {
                    warnings.push({ kind: 'not-a-dicom-file', filename: file.name });
                    skippedCount += 1;
                    continue;
                }
                const imageId = absolutePath
                    ? imageIdFactory.fromPath(absolutePath)
                    : imageIdFactory.fromFile(file);
                imageIdByUid.set(parsed.sopInstanceUID, imageId);
                headers.push(parsed);
            } catch (err) {
                warnings.push({
                    kind: 'corrupt-instance',
                    filename: file.name,
                    reason: err instanceof Error ? err.message : String(err),
                });
                skippedCount += 1;
            }
        }

        return { headers, imageIdByUid, warnings, skippedCount };
    }

    function finaliseWarnings(
        studies: Map<string, DicomStudy>,
        warnings: DicomImportWarning[],
    ): DicomImportWarning[] {
        const extra: DicomImportWarning[] = [];
        for (const study of studies.values()) {
            for (const series of study.series) {
                if (series.instances.length === 1) {
                    extra.push({
                        kind: 'single-slice-series',
                        seriesInstanceUID: series.seriesInstanceUID,
                    });
                }
                if (!series.pixelSpacingMm) {
                    extra.push({
                        kind: 'missing-pixel-spacing',
                        seriesInstanceUID: series.seriesInstanceUID,
                    });
                }
                if (series.hasSpacingGaps) {
                    extra.push({
                        kind: 'non-monotonic-slices',
                        seriesInstanceUID: series.seriesInstanceUID,
                    });
                }
                if (series.modality === 'OTHER') {
                    extra.push({
                        kind: 'unsupported-modality',
                        seriesInstanceUID: series.seriesInstanceUID,
                        modality: 'OTHER',
                    });
                }
            }
        }
        return [...warnings, ...extra];
    }

    async function fromItems(
        items: Array<{ file: File; absolutePath: string | null }>,
    ): Promise<DicomImportResult> {
        const { headers, imageIdByUid, warnings, skippedCount } = await parseBatch(items);
        const studies = toStudyMap(
            headers,
            (h) => imageIdByUid.get(h.sopInstanceUID) ?? '',
        );
        return {
            studies: Array.from(studies.values()),
            warnings: finaliseWarnings(studies, warnings),
            skippedCount,
        };
    }

    return {
        async fromFiles(files) {
            return fromItems(files.map((file) => ({ file, absolutePath: null })));
        },

        async fromZip(archive) {
            const expanded = await zip.expand(archive);
            return fromItems(expanded.map((file) => ({ file, absolutePath: null })));
        },

        async detectSiblings(seedAbsolutePath) {
            const siblings = await filesystem.listSiblings(seedAbsolutePath);
            const items: Array<{ file: File; absolutePath: string }> = [];
            for (const path of siblings) {
                try {
                    const bytes = await filesystem.readFileAsArrayBuffer(path);
                    const name = path.split(/[\\/]/).pop() ?? 'unknown.dcm';
                    const file = new File([bytes], name);
                    items.push({ file, absolutePath: path });
                } catch {
                    // Skip unreadable files silently; warning generated below.
                }
            }
            return fromItems(items);
        },
    };
}

// Re-export for convenience
export { computeSeriesGeometry };

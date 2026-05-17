/**
 * DICOM header parser backed by `dicom-parser` (already installed).
 *
 * Only parses what we need to group, sort and describe a series. Pixel data
 * stays untouched — decoding is Cornerstone3D's job later.
 */

import dicomParser from 'dicom-parser';
import type {
    DicomHeaderParserPort,
    ParsedDicomHeaders,
} from '../application/ports';

/** DICOM VR-aware tag accessors. All return null when the tag is missing. */
function stringTag(dataset: dicomParser.DataSet, tag: string): string | null {
    const raw = dataset.string(tag);
    if (!raw) return null;
    const trimmed = raw.trim();
    return trimmed.length > 0 ? trimmed : null;
}

function intTag(dataset: dicomParser.DataSet, tag: string): number | null {
    const raw = stringTag(dataset, tag);
    if (raw === null) return null;
    const parsed = Number.parseInt(raw, 10);
    return Number.isFinite(parsed) ? parsed : null;
}

function floatTag(dataset: dicomParser.DataSet, tag: string): number | null {
    const raw = stringTag(dataset, tag);
    if (raw === null) return null;
    const parsed = Number.parseFloat(raw);
    return Number.isFinite(parsed) ? parsed : null;
}

function floatArrayTag(dataset: dicomParser.DataSet, tag: string): number[] | null {
    const raw = stringTag(dataset, tag);
    if (raw === null) return null;
    const parts = raw
        .split(/[\\\s]+/)
        .map((p) => Number.parseFloat(p))
        .filter((v) => Number.isFinite(v));
    return parts.length > 0 ? parts : null;
}

function parseImagePosition(
    values: number[] | null,
): [number, number, number] | null {
    if (!values || values.length < 3) return null;
    return [values[0], values[1], values[2]];
}

function parsePixelSpacing(values: number[] | null): [number, number] | null {
    if (!values || values.length < 2) return null;
    return [values[0], values[1]];
}

function normalisePatientName(raw: string | null): string {
    if (!raw) return '';
    // DICOM name is `Family^Given^Middle^Prefix^Suffix` — flatten.
    return raw.replace(/\^/g, ' ').replace(/\s+/g, ' ').trim();
}

export function createDicomHeaderParserAdapter(): DicomHeaderParserPort {
    return {
        async parse(bytes, sourceFilePath) {
            try {
                const byteArray = new Uint8Array(bytes);
                const dataset = dicomParser.parseDicom(byteArray);

                const sopInstanceUID = stringTag(dataset, 'x00080018');
                const studyInstanceUID = stringTag(dataset, 'x0020000d');
                const seriesInstanceUID = stringTag(dataset, 'x0020000e');
                if (!sopInstanceUID || !studyInstanceUID || !seriesInstanceUID) {
                    return null;
                }

                const headers: ParsedDicomHeaders = {
                    sopInstanceUID,
                    studyInstanceUID,
                    seriesInstanceUID,
                    studyId: stringTag(dataset, 'x00200010'),
                    studyDate: stringTag(dataset, 'x00080020'),
                    studyDescription: stringTag(dataset, 'x00081030'),
                    accessionNumber: stringTag(dataset, 'x00080050'),
                    seriesNumber: intTag(dataset, 'x00200011'),
                    seriesDescription: stringTag(dataset, 'x0008103e'),
                    modality: stringTag(dataset, 'x00080060') ?? 'OTHER',
                    instanceNumber: intTag(dataset, 'x00200013'),
                    sliceLocation: floatTag(dataset, 'x00201041'),
                    imagePositionPatient: parseImagePosition(
                        floatArrayTag(dataset, 'x00200032'),
                    ),
                    pixelSpacing: parsePixelSpacing(floatArrayTag(dataset, 'x00280030')),
                    sliceThickness: floatTag(dataset, 'x00180050'),
                    patient: {
                        patientId: stringTag(dataset, 'x00100020') ?? '',
                        patientName: normalisePatientName(stringTag(dataset, 'x00100010')),
                        birthDate: stringTag(dataset, 'x00100030'),
                        sex: stringTag(dataset, 'x00100040'),
                    },
                };

                // Touch sourceFilePath reference so linters don't complain if unused downstream.
                void sourceFilePath;

                return headers;
            } catch {
                return null;
            }
        },
    };
}

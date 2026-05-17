/**
 * Canonical DICOM domain types for the TlantiCAD viewer.
 *
 * These are the ONLY types `application/` use cases import. Adapters
 * (infrastructure/) must map vendor-specific shapes (pydicom, Cornerstone3D,
 * dicom-parser) into these before crossing the port boundary.
 *
 * No rendering, no IO, no cornerstone imports here.
 */

/** DICOM modality codes we care about clinically. */
export type Modality =
    | 'CT'   // Computed tomography
    | 'CBCT' // Cone-beam CT (derived, reported as CT in DICOM tag + extra hints)
    | 'MR'   // Magnetic resonance
    | 'CR'   // Computed radiography
    | 'DX'   // Digital radiography
    | 'PX'   // Panoramic X-Ray
    | 'IO'   // Intra-oral (surface scan wrapped as DICOM)
    | 'SEG'  // Segmentation
    | 'RT'   // Radiotherapy structure set
    | 'OTHER';

/** Minimal DICOM patient demographics we surface in the UI. */
export interface DicomPatientInfo {
    patientId: string;
    patientName: string;
    birthDate: string | null;
    sex: 'M' | 'F' | 'O' | null;
}

/** Single slice / frame within a series. */
export interface DicomInstance {
    sopInstanceUID: string;
    instanceNumber: number | null;
    sourceFilePath: string | null; // absolute path when available (desktop)
    imageId: string;               // cornerstone imageId (wadouri:file:...)
    /** Physical position along the axial axis — used for sorting. */
    sliceLocation: number | null;
    /** Full `ImagePositionPatient` [x,y,z] if present. */
    imagePositionPatient: [number, number, number] | null;
    /** Uncompressed size in bytes (approximate, for UI hints). */
    bytes: number | null;
}

/** One acquisition — slices that share geometry and belong together. */
export interface DicomSeries {
    seriesInstanceUID: string;
    seriesNumber: number | null;
    seriesDescription: string | null;
    modality: Modality;
    instanceCount: number;
    /** Instances sorted by sliceLocation / imagePositionPatient Z / instanceNumber. */
    instances: DicomInstance[];
    /** Z spacing in mm, when computable from the first two instances. */
    sliceSpacingMm: number | null;
    /** X/Y pixel spacing in mm. */
    pixelSpacingMm: [number, number] | null;
    /** True when `instances.length > 1` and spacing is consistent. */
    isVolumetric: boolean;
    /** True when monotonic gaps were detected between consecutive slices. */
    hasSpacingGaps: boolean;
}

/** One clinical study — may contain multiple series. */
export interface DicomStudy {
    studyInstanceUID: string;
    studyId: string | null;
    studyDescription: string | null;
    studyDate: string | null;
    accessionNumber: string | null;
    patient: DicomPatientInfo;
    series: DicomSeries[];
}

/** Warnings surfaced to the UI without blocking import. */
export type DicomImportWarning =
    | { kind: 'non-monotonic-slices'; seriesInstanceUID: string }
    | { kind: 'single-slice-series'; seriesInstanceUID: string }
    | { kind: 'missing-pixel-spacing'; seriesInstanceUID: string }
    | { kind: 'unsupported-modality'; seriesInstanceUID: string; modality: string }
    | { kind: 'corrupt-instance'; filename: string; reason: string }
    | { kind: 'not-a-dicom-file'; filename: string };

/** Result of an import operation. */
export interface DicomImportResult {
    studies: DicomStudy[];
    warnings: DicomImportWarning[];
    /** Files we tried to parse but weren't DICOM. */
    skippedCount: number;
}

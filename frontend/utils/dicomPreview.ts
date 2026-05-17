import * as dicomParser from 'dicom-parser';

import type { DicomAdjustments } from '@/types';

export interface DicomPreviewResult {
    dataUrl: string;
    height: number;
    source?: 'local' | 'backend';
    width: number;
}

interface DicomPreviewOptions {
    adjustments?: DicomAdjustments;
    invert?: boolean;
    windowCenter?: number;
    windowWidth?: number;
}

function getString(dataSet: dicomParser.DataSet, tag: string) {
    try {
        return dataSet.string(tag) || undefined;
    } catch {
        return undefined;
    }
}

function getNumber(dataSet: dicomParser.DataSet, tag: string) {
    const value = getString(dataSet, tag);
    if (!value) {
        return undefined;
    }

    const parsed = Number.parseFloat(value.split('\\')[0]);
    return Number.isFinite(parsed) ? parsed : undefined;
}

function getPixelArray(dataSet: dicomParser.DataSet, buffer: ArrayBuffer) {
    const pixelDataElement = dataSet.elements.x7fe00010;
    if (!pixelDataElement || pixelDataElement.length <= 0) {
        return null;
    }

    const bitsAllocated = getNumber(dataSet, 'x00280100') ?? 16;
    const pixelRepresentation = getNumber(dataSet, 'x00280103') ?? 0;
    const offset = pixelDataElement.dataOffset;

    if (bitsAllocated <= 8) {
        return new Uint8Array(buffer, offset, pixelDataElement.length);
    }

    const length = Math.floor(pixelDataElement.length / 2);
    const view = new DataView(buffer, offset, pixelDataElement.length);

    if (pixelRepresentation === 1) {
        const values = new Int16Array(length);
        for (let index = 0; index < length; index += 1) {
            values[index] = view.getInt16(index * 2, true);
        }
        return values;
    }

    const values = new Uint16Array(length);
    for (let index = 0; index < length; index += 1) {
        values[index] = view.getUint16(index * 2, true);
    }
    return values;
}

export function buildDicomPreviewDataUrl(buffer: ArrayBuffer, options: DicomPreviewOptions = {}): DicomPreviewResult | null {
    if (typeof document === 'undefined') {
        return null;
    }

    try {
        const byteArray = new Uint8Array(buffer);
        const dataSet = dicomParser.parseDicom(byteArray);
        const width = getNumber(dataSet, 'x00280011');
        const height = getNumber(dataSet, 'x00280010');
        const samplesPerPixel = getNumber(dataSet, 'x00280002') ?? 1;
        const photoMetricInterpretation = getString(dataSet, 'x00280004') ?? 'MONOCHROME2';
        const rescaleSlope = getNumber(dataSet, 'x00281053') ?? 1;
        const rescaleIntercept = getNumber(dataSet, 'x00281052') ?? 0;

        if (!width || !height || samplesPerPixel !== 1) {
            return null;
        }

        const pixels = getPixelArray(dataSet, buffer);
        if (!pixels || !pixels.length) {
            return null;
        }

        let min = Number.POSITIVE_INFINITY;
        let max = Number.NEGATIVE_INFINITY;
        const scaledPixels = new Float32Array(pixels.length);

        for (let index = 0; index < pixels.length; index += 1) {
            const value = Number(pixels[index]) * rescaleSlope + rescaleIntercept;
            scaledPixels[index] = value;
            if (value < min) min = value;
            if (value > max) max = value;
        }

        const intrinsicCenter = getNumber(dataSet, 'x00281050') ?? ((min + max) / 2);
        const intrinsicWidth = getNumber(dataSet, 'x00281051') ?? (max - min);
        const windowCenter = options.windowCenter ?? intrinsicCenter;
        const windowWidth = Math.max(options.windowWidth ?? intrinsicWidth, 1);
        const lower = windowCenter - windowWidth / 2;
        const upper = windowCenter + windowWidth / 2;

        const imageData = new ImageData(width, height);
        const shouldInvert = Boolean(options.invert) !== (photoMetricInterpretation === 'MONOCHROME1');

        for (let index = 0; index < scaledPixels.length; index += 1) {
            const raw = scaledPixels[index];
            let normalized = ((raw - lower) / Math.max(upper - lower, 1)) * 255;
            normalized = Number.isFinite(normalized) ? Math.max(0, Math.min(255, normalized)) : 0;
            const gray = shouldInvert ? 255 - normalized : normalized;
            const pixelIndex = index * 4;
            imageData.data[pixelIndex] = gray;
            imageData.data[pixelIndex + 1] = gray;
            imageData.data[pixelIndex + 2] = gray;
            imageData.data[pixelIndex + 3] = 255;
        }

        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const context = canvas.getContext('2d');
        if (!context) {
            return null;
        }

        context.putImageData(imageData, 0, 0);

        const crop = options.adjustments?.crop;
        const hasCrop = crop && (crop.top || crop.right || crop.bottom || crop.left);

        if (hasCrop) {
            const cropX = Math.floor((width * crop.left) / 100);
            const cropY = Math.floor((height * crop.top) / 100);
            const cropWidth = Math.max(1, width - cropX - Math.floor((width * crop.right) / 100));
            const cropHeight = Math.max(1, height - cropY - Math.floor((height * crop.bottom) / 100));
            const croppedCanvas = document.createElement('canvas');
            croppedCanvas.width = cropWidth;
            croppedCanvas.height = cropHeight;
            const croppedContext = croppedCanvas.getContext('2d');
            if (!croppedContext) {
                return null;
            }
            croppedContext.drawImage(canvas, cropX, cropY, cropWidth, cropHeight, 0, 0, cropWidth, cropHeight);
            return {
                dataUrl: croppedCanvas.toDataURL('image/png'),
                width: cropWidth,
                height: cropHeight,
                source: 'local',
            };
        }

        return {
            dataUrl: canvas.toDataURL('image/png'),
            width,
            height,
            source: 'local',
        };
    } catch (error) {
        console.warn('Failed to build local DICOM preview', error);
        return null;
    }
}
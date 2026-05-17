import JSZip from 'jszip';
import { FileData } from '../types';
import { v4 as uuidv4 } from 'uuid';
import { parseDicomInfo } from './dicomMetadata';

const MAX_DICOM_SLICES = 240;

type DicomSeriesEntry = {
  name: string;
  buffer: ArrayBuffer;
  instanceNumber?: number;
  sliceLocation?: number;
  imagePositionZ?: number;
  seriesInstanceUid?: string;
  studyInstanceUid?: string;
  seriesDescription?: string;
};

export interface FileUploadEntry {
  file: File;
  sourcePath?: string;
  relativePath?: string;
}

const createDefaultDicomAdjustments = () => ({
  rotation: 0,
  crop: {
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  },
});

function analyzeDicomSeries(items: DicomSeriesEntry[]) {
  const sortedItems = [...items].sort((left, right) => {
    if (left.instanceNumber !== undefined && right.instanceNumber !== undefined) {
      return left.instanceNumber - right.instanceNumber;
    }

    if (left.imagePositionZ !== undefined && right.imagePositionZ !== undefined) {
      return left.imagePositionZ - right.imagePositionZ;
    }

    if (left.sliceLocation !== undefined && right.sliceLocation !== undefined) {
      return left.sliceLocation - right.sliceLocation;
    }

    return left.name.localeCompare(right.name, undefined, { numeric: true });
  });

  const warnings: string[] = [];
  const missingSlices: number[] = [];

  const instanceNumbers = sortedItems
    .map((item) => item.instanceNumber)
    .filter((value): value is number => value !== undefined)
    .sort((left, right) => left - right);

  if (instanceNumbers.length >= 2) {
    for (let index = 1; index < instanceNumbers.length; index += 1) {
      const previous = instanceNumbers[index - 1];
      const current = instanceNumbers[index];
      if (current - previous > 1) {
        for (let missing = previous + 1; missing < current; missing += 1) {
          missingSlices.push(missing);
        }
      }
    }
  }

  if (missingSlices.length) {
    warnings.push(`Missing slices detected: ${missingSlices.slice(0, 8).join(', ')}${missingSlices.length > 8 ? '…' : ''}`);
  }

  let subsampleStep = 1;
  if (sortedItems.length > MAX_DICOM_SLICES) {
    subsampleStep = Math.ceil(sortedItems.length / MAX_DICOM_SLICES);
    warnings.push(`Large DICOM series detected. Subsampled every ${subsampleStep} slice for smoother performance.`);
  }

  const subsampledItems = subsampleStep > 1
    ? sortedItems.filter((_, index) => index % subsampleStep === 0 || index === sortedItems.length - 1)
    : sortedItems;

  return {
    sortedItems,
    subsampledItems,
    warnings,
    missingSlices,
    subsampleStep,
    originalSliceCount: sortedItems.length,
  };
}

function normalizeImportPath(pathLike: string) {
  return pathLike.replace(/\\/g, '/');
}

function getImportPathSegments(pathLike: string) {
  return normalizeImportPath(pathLike)
    .split('/')
    .filter(Boolean);
}

function getImportBasename(pathLike: string) {
  const segments = getImportPathSegments(pathLike);
  return segments[segments.length - 1]?.toLowerCase() ?? '';
}

function shouldIgnoreImportPath(pathLike: string) {
  const segments = getImportPathSegments(pathLike);
  const basename = getImportBasename(pathLike);

  if (!segments.length && !basename) {
    return false;
  }

  return segments.some((segment) => segment === '__MACOSX')
    || basename === '.ds_store'
    || basename.startsWith('._');
}

function getImportDirectory(pathLike: string) {
  const normalized = normalizeImportPath(pathLike);
  const lastSlash = normalized.lastIndexOf('/');
  if (lastSlash === -1) {
    return '';
  }

  return normalized.slice(0, lastSlash);
}

function getSeriesLabel(entry: FileUploadEntry) {
  const relativePath = entry.relativePath || entry.file.webkitRelativePath || '';
  const relativeDirectory = relativePath ? getImportDirectory(relativePath) : '';
  if (relativeDirectory) {
    return relativeDirectory;
  }

  if (entry.sourcePath) {
    const sourceDirectory = getImportDirectory(entry.sourcePath);
    if (sourceDirectory) {
      const segments = getImportPathSegments(sourceDirectory);
      return segments[segments.length - 1] ?? 'root';
    }
  }

  return 'root';
}

function getSeriesSourcePath(entry: FileUploadEntry) {
  if (!entry.sourcePath) {
    return undefined;
  }

  return getImportDirectory(entry.sourcePath) || entry.sourcePath;
}

function getSeriesGroupingKey(entry: FileUploadEntry, parsed?: ReturnType<typeof parseDicomInfo> | null) {
  const seriesSourcePath = getSeriesSourcePath(entry) ?? entry.sourcePath ?? 'root';
  const seriesUid = parsed?.seriesInstanceUid ?? parsed?.metadata.seriesInstanceUid;
  return `${seriesSourcePath}::${seriesUid ?? getSeriesLabel(entry)}`;
}

function resolveDicomSeriesLabel(fallbackLabel: string, items: DicomSeriesEntry[]) {
  const descriptiveLabel = items.find((item) => item.seriesDescription?.trim())?.seriesDescription?.trim();
  return descriptiveLabel || fallbackLabel;
}

function createDicomSeriesFileData(
  label: string,
  items: DicomSeriesEntry[],
  sourcePath?: string,
) {
  const { subsampledItems, warnings, missingSlices, subsampleStep, originalSliceCount } = analyzeDicomSeries(items);
  const first = subsampledItems[0];
  const initialInfo = first ? parseDicomInfo(first.buffer, subsampledItems.length) : null;
  const seriesFileData = createFileData(`${label} (DICOM Series)`, 'DICOM', null, first?.buffer ?? items[0].buffer, {
    sourcePath,
    dicomMetadata: initialInfo
      ? {
        ...initialInfo.metadata,
        sliceCount: subsampledItems.length,
        originalSliceCount,
        subsampleStep,
        missingSlices,
        warnings,
      }
      : {
        sliceCount: subsampledItems.length,
        originalSliceCount,
        subsampleStep,
        missingSlices,
        warnings,
      },
    windowCenter: initialInfo?.windowCenter,
    windowWidth: initialInfo?.windowWidth,
  });

  seriesFileData.buffers = subsampledItems.map((item) => item.buffer);
  return seriesFileData;
}

export const handleDirectoryUpload = async (entries: FileUploadEntry[]): Promise<FileData[]> => {
  const files: FileData[] = [];
  const dicomSeries = new Map<string, { label: string; sourcePath?: string; items: DicomSeriesEntry[] }>();

  for (const entry of entries) {
    const importPath = entry.relativePath || entry.file.webkitRelativePath || entry.sourcePath || entry.file.name;
    if (shouldIgnoreImportPath(importPath)) {
      continue;
    }

    const type = getFileType(entry.file.name);
    if (!type) {
      continue;
    }

    const buffer = await entry.file.arrayBuffer();
    if (type === 'DICOM') {
      const seriesLabel = getSeriesLabel(entry);
      const parsed = parseDicomInfo(buffer, 1);
      const seriesKey = getSeriesGroupingKey(entry, parsed);
      const currentSeries = dicomSeries.get(seriesKey) ?? {
        label: seriesLabel,
        sourcePath: getSeriesSourcePath(entry),
        items: [],
      };

      currentSeries.items.push({
        name: importPath,
        buffer,
        instanceNumber: parsed?.instanceNumber,
        sliceLocation: parsed?.sliceLocation,
        imagePositionZ: parsed?.imagePositionZ,
        seriesInstanceUid: parsed?.seriesInstanceUid,
        studyInstanceUid: parsed?.studyInstanceUid,
        seriesDescription: parsed?.seriesDescription,
      });
      dicomSeries.set(seriesKey, currentSeries);
      continue;
    }

    files.push(createFileData(entry.file.name, type, entry.file, buffer, {
      sourcePath: entry.sourcePath,
    }));
  }

  for (const series of dicomSeries.values()) {
    if (!series.items.length) {
      continue;
    }

    files.push(createDicomSeriesFileData(resolveDicomSeriesLabel(series.label, series.items), series.items, series.sourcePath));
  }

  return files;
};

export const handleFileUpload = async (
  file: File,
  options?: {
    sourcePath?: string;
  },
): Promise<FileData[]> => {
  const files: FileData[] = [];
  const fileName = file.name.toLowerCase();

  if (fileName.endsWith('.zip')) {
    const zip = new JSZip();
    try {
      const contents = await zip.loadAsync(file);
      const entries = Object.keys(contents.files);

      let sceneMeta: any[] | null = null;
      if (contents.files['scene.json']) {
        try {
          const metaStr = await contents.files['scene.json'].async('text');
          sceneMeta = JSON.parse(metaStr);
        } catch (e) {
          console.error("Failed to parse scene.json", e);
        }
      }

      const dicomBuffers: { [folder: string]: DicomSeriesEntry[] } = {};

      for (const filename of entries) {
        if (filename === 'scene.json') continue;
        if (shouldIgnoreImportPath(filename)) continue;

        if (!contents.files[filename].dir) {
          const type = getFileType(filename);
          if (type === 'DICOM') {
            const blob = await contents.files[filename].async('blob');
            const buffer = await blob.arrayBuffer();
            const parsed = parseDicomInfo(buffer, 1);
            const groupingEntry: FileUploadEntry = {
              file: new File([blob], filename.split('/').pop() ?? filename, { type: 'application/dicom' }),
              relativePath: filename,
              sourcePath: options?.sourcePath,
            };
            const seriesKey = getSeriesGroupingKey(groupingEntry, parsed);
            if (!dicomBuffers[seriesKey]) dicomBuffers[seriesKey] = [];
            dicomBuffers[seriesKey].push({
              name: filename,
              buffer,
              instanceNumber: parsed?.instanceNumber,
              sliceLocation: parsed?.sliceLocation,
              imagePositionZ: parsed?.imagePositionZ,
              seriesInstanceUid: parsed?.seriesInstanceUid,
              studyInstanceUid: parsed?.studyInstanceUid,
              seriesDescription: parsed?.seriesDescription,
            });
          } else if (type) {
            const blob = await contents.files[filename].async('blob');
            const buffer = await blob.arrayBuffer();
            const fileData = createFileData(filename, type, blob, buffer, {
              sourcePath: options?.sourcePath,
            });

            if (sceneMeta) {
              const meta = sceneMeta.find(m => m.name === filename || `${m.name}.stl` === filename || `${m.name}.jpg` === filename || `${m.name}.dcm` === filename);
              if (meta) {
                if (meta.id) fileData.id = meta.id;
                fileData.name = meta.name;
                if (meta.position) fileData.position = meta.position;
                if (meta.rotation) fileData.rotation = meta.rotation;
                if (meta.scale) fileData.scale = meta.scale;
                if (meta.visible !== undefined) fileData.visible = meta.visible;
                if (meta.opacity !== undefined) fileData.opacity = meta.opacity;
                if (meta.wireframe !== undefined) fileData.wireframe = meta.wireframe;
                if (meta.dicomAdjustments !== undefined) fileData.dicomAdjustments = meta.dicomAdjustments;
                if (meta.windowCenter !== undefined) fileData.windowCenter = meta.windowCenter;
                if (meta.windowWidth !== undefined) fileData.windowWidth = meta.windowWidth;
                if (meta.sliceIndex !== undefined) fileData.sliceIndex = meta.sliceIndex;
              }
            }

            files.push(fileData);
          }
        }
      }

      for (const folder in dicomBuffers) {
        const items = dicomBuffers[folder];
        const fallbackLabel = items[0]?.seriesDescription || folder.split('::')[1] || 'DICOM Series';
        const seriesFileData = createDicomSeriesFileData(resolveDicomSeriesLabel(fallbackLabel, items), items, options?.sourcePath);

        if (sceneMeta) {
          const meta = sceneMeta.find(m => m.name === folder || m.name === `${folder} (DICOM Series)`);
          if (meta) {
            if (meta.id) seriesFileData.id = meta.id;
            seriesFileData.name = meta.name;
            if (meta.position) seriesFileData.position = meta.position;
            if (meta.rotation) seriesFileData.rotation = meta.rotation;
            if (meta.scale) seriesFileData.scale = meta.scale;
            if (meta.visible !== undefined) seriesFileData.visible = meta.visible;
            if (meta.opacity !== undefined) seriesFileData.opacity = meta.opacity;
            if (meta.wireframe !== undefined) seriesFileData.wireframe = meta.wireframe;
            if (meta.dicomAdjustments !== undefined) seriesFileData.dicomAdjustments = meta.dicomAdjustments;
            if (meta.windowCenter !== undefined) seriesFileData.windowCenter = meta.windowCenter;
            if (meta.windowWidth !== undefined) seriesFileData.windowWidth = meta.windowWidth;
            if (meta.sliceIndex !== undefined) seriesFileData.sliceIndex = meta.sliceIndex;
          }
        }

        files.push(seriesFileData);
      }

    } catch (e) {
      console.error("Error unzipping", e);
    }
  } else {
    const buffer = await file.arrayBuffer();
    const type = getFileType(fileName);
    if (type) {
      if (type === 'DICOM') {
        const parsed = parseDicomInfo(buffer, 1);
        files.push(createFileData(file.name, type, file, buffer, {
          sourcePath: options?.sourcePath,
          dicomMetadata: parsed?.metadata,
          windowCenter: parsed?.windowCenter,
          windowWidth: parsed?.windowWidth,
        }));
      } else {
        files.push(createFileData(file.name, type, file, buffer, {
          sourcePath: options?.sourcePath,
        }));
      }
    }
  }

  return files;
};

const createFileData = (
  name: string,
  type: 'DICOM' | 'MODEL' | 'IMAGE',
  blob: Blob | File | null,
  buffer: ArrayBuffer,
  options?: {
    sourcePath?: string;
    dicomMetadata?: FileData['dicomMetadata'];
    windowCenter?: number;
    windowWidth?: number;
  },
): FileData => {
  return {
    id: uuidv4(),
    name: name,
    type: type,
    url: blob ? URL.createObjectURL(blob) : undefined,
    sourcePath: options?.sourcePath,
    buffer: buffer,
    visible: true,
    opacity: 1.0,
    wireframe: false,
    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
    windowCenter: options?.windowCenter ?? 40,
    windowWidth: options?.windowWidth ?? 400,
    sliceIndex: 0,
    dicomMetadata: options?.dicomMetadata,
    dicomAdjustments: type === 'DICOM' ? createDefaultDicomAdjustments() : undefined,
    dicomWorkspaceView: type === 'DICOM' ? 'review' : undefined,
  };
};

const getFileType = (filename: string): 'DICOM' | 'MODEL' | 'IMAGE' | null => {
  const ext = filename.split('.').pop()?.toLowerCase();
  if (['dcm', 'dicom', 'ima'].includes(ext || '')) return 'DICOM';
  if (['obj', 'stl', 'glb', 'gltf', 'ply'].includes(ext || '')) return 'MODEL';
  if (['png', 'jpg', 'jpeg', 'webp', 'avif', 'svg'].includes(ext || '')) return 'IMAGE';
  return null;
};
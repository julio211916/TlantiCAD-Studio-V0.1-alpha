import type { FileData } from '@/types';
import { isDentistSotaBetaEnabled } from '@/lib/feature-flags';

export interface TlantiCadToolCard {
  id: 'slicer-dentalsegmentator' | 'slicer-automated-dental-tools' | 'tooth-group-network' | 'hip-dicom-stl' | 'dentist-sota-pxi' | 'dentist-sota-cbct' | 'dentist-sota-dicom-safe-upload';
  label: string;
  badge: string;
  summary: string;
  inputs: string[];
  action: string;
}

const TOOL_CARDS: TlantiCadToolCard[] = [
  {
    id: 'hip-dicom-stl',
    label: 'Hip DICOM → STL',
    badge: 'FastAPI',
    summary: 'HU conversion, bone segmentation, marching cubes and STL export from DICOM series.',
    inputs: ['.dcm', '.dicom', '.ima', '.zip'],
    action: 'POST /dicom/reconstruct-hip-stl',
  },
  {
    id: 'slicer-dentalsegmentator',
    label: 'SlicerDentalSegmentator',
    badge: 'CBCT',
    summary: 'Automatic segmentation for CT/CBCT dental volumes.',
    inputs: ['.dcm', '.dicom', '.ima', '.zip'],
    action: 'vendor pipeline',
  },
  {
    id: 'slicer-automated-dental-tools',
    label: 'SlicerAutomatedDentalTools',
    badge: 'Landmarks',
    summary: 'Orientation, landmarks, batch segmentation and registration workflows.',
    inputs: ['.dcm', '.dicom', '.ima', '.vtk', '.stl', '.obj'],
    action: 'vendor pipeline',
  },
  {
    id: 'tooth-group-network',
    label: 'ToothGroupNetwork',
    badge: 'OBJ',
    summary: 'Tooth grouping and labeling for OBJ mesh workflows.',
    inputs: ['.obj'],
    action: 'vendor pipeline',
  },
  {
    id: 'dentist-sota-pxi',
    label: 'Dentist-SOTA PXI R2U-Net',
    badge: 'PXI',
    summary: 'Boundary-aware R2U-Net + MONAI route for panoramic X-ray tooth segmentation.',
    inputs: ['.png', '.jpg', '.jpeg', '.webp'],
    action: 'POST /dentist-sota/pxi/segment',
  },
  {
    id: 'dentist-sota-cbct',
    label: 'Dentist-SOTA CBCT R2U-Net',
    badge: 'CBCT',
    summary: 'STS-Tooth aligned CBCT segmentation guidance with sliding-window inference planning.',
    inputs: ['.dcm', '.dicom', '.ima', '.nii.gz'],
    action: 'POST /dentist-sota/cbct/segment',
  },
  {
    id: 'dentist-sota-dicom-safe-upload',
    label: 'Dentist-SOTA DICOM Safe Upload',
    badge: 'PII',
    summary: 'FastAPI DICOM PII stripping and safe metadata extraction before downstream processing.',
    inputs: ['.dcm', '.dicom', '.ima'],
    action: 'POST /dentist-sota/dicom/sanitize',
  },
];

function visibleToolCards() {
  if (isDentistSotaBetaEnabled()) {
    return TOOL_CARDS;
  }

  return TOOL_CARDS.filter((card) => !card.id.startsWith('dentist-sota-'));
}

function extensionFromFile(file: Pick<FileData, 'name' | 'sourcePath'> | null) {
  const target = file?.sourcePath ?? file?.name ?? '';
  const ext = target.split('.').pop()?.toLowerCase();
  return ext ? `.${ext}` : null;
}

export function getTlantiCadToolsForFile(file: FileData | null): TlantiCadToolCard[] {
  if (!file) {
    return visibleToolCards().slice(0, 3);
  }

  const cards = visibleToolCards();
  const ext = extensionFromFile(file);

  if (file.type === 'DICOM') {
    return cards.filter((card) => ['hip-dicom-stl', 'slicer-dentalsegmentator', 'slicer-automated-dental-tools', 'dentist-sota-cbct', 'dentist-sota-dicom-safe-upload'].includes(card.id));
  }

  if (ext && ['.png', '.jpg', '.jpeg', '.webp'].includes(ext)) {
    return cards.filter((card) => ['dentist-sota-pxi'].includes(card.id));
  }

  if (ext === '.obj') {
    return cards.filter((card) => ['tooth-group-network', 'slicer-automated-dental-tools'].includes(card.id));
  }

  if (ext === '.stl' || ext === '.vtk') {
    return cards.filter((card) => ['slicer-automated-dental-tools'].includes(card.id));
  }

  return cards.filter((card) => card.id !== 'hip-dicom-stl');
}

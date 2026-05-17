export type ToolMode = 'SELECT' | 'MOVE' | 'ROTATE' | 'SCALE' | 'CLIP' | 'MEASURE' | 'SEGMENT' | 'SCULPT' | 'VOXELIZE' | 'POINT_CLOUD' | 'BOOLEAN_CUT' | 'CROP';
export type Language = 'en' | 'es' | 'ru' | 'it' | 'fr' | 'pt';
export type ThemeMode = 'dark' | 'light';
export type DentalMaterialKey = 'enamel' | 'dentin' | 'gingiva' | 'zirconia' | 'metal' | 'pmma' | 'guide-resin' | 'scan-default';
export type DentalMaterialOverride = 'auto' | DentalMaterialKey;

export interface MeshMetadata {
  vertices: number;
  triangles: number;
  volume: number;
  area: number;
}

export interface DicomMetadata {
  patientName?: string;
  studyDate?: string;
  modality?: string;
  dimensions?: string;
  seriesDescription?: string;
  studyInstanceUid?: string;
  seriesInstanceUid?: string;
  sliceCount: number;
  originalSliceCount?: number;
  subsampleStep?: number;
  missingSlices?: number[];
  warnings?: string[];
}

export interface DicomAdjustments {
  rotation: number;
  crop: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
}

export type DicomWorkspaceView = 'review' | 'volume' | 'ai' | 'report';

export interface FileAiSegmentationState {
  status: 'idle' | 'running' | 'completed' | 'error';
  sessionId?: string;
  workflowId?: string;
  recommendedTool?: string;
  outputPath?: string;
  logs?: string;
  error?: string;
  numberingSystem?: 'FDI' | 'UNIVERSAL';
  lastRunAt?: string;
}

export interface FileData {
  id: string;
  name: string;
  type: 'DICOM' | 'MODEL' | 'IMAGE' | 'GROUP';
  parentId?: string;
  isExpanded?: boolean;
  url?: string;
  sourcePath?: string;
  buffer?: ArrayBuffer;
  buffers?: ArrayBuffer[];
  visible: boolean;
  opacity: number;
  wireframe: boolean;
  metadata?: MeshMetadata;
  dicomMetadata?: DicomMetadata;
  position: [number, number, number];
  rotation: [number, number, number];
  scale: [number, number, number];

  // DICOM display state
  windowCenter: number;
  windowWidth: number;
  sliceIndex: number;
  dicomAdjustments?: DicomAdjustments;
  dicomWorkspaceView?: DicomWorkspaceView;

  // Advanced Mesh States
  isPointCloud?: boolean;
  isVoxelized?: boolean;
  isSegmented?: boolean;
  meshVault?: {
    meshKey: string;
    checksumSha256: string;
    bytes: number;
    chunkCount: number;
    chunkSizeBytes: number;
    storagePath: string;
    gpuHints?: unknown;
  };
  aiSegmentation?: FileAiSegmentationState;

  // Semantic Library State
  semanticUsage?: 'overlay' | 'preset' | 'dental-library' | 'reference';
  semanticTags?: string[];
  sourceRoot?: string;
  sourceRelativePath?: string;
  dentalMaterialOverride?: DentalMaterialOverride;

  // Actions
  action?: { type: 'OPTIMIZE' | 'UNDO' | 'REDO', timestamp: number };
}

export interface ViewportState {
  tools: {
    active: ToolMode;
    gridVisible: boolean;
    gizmoVisible: boolean;
  };
  files: FileData[];
  selectedId: string | null;
}

export const THEME = {
  gradient: "bg-text-display text-black",
  gradientText: "text-text-display",
  glass: "bg-surface border border-border",
  glassLight: "bg-surface border border-border",
  glassDark: "bg-surface-raised border border-border",
  glassHover: "hover:bg-surface-raised transition-colors duration-200",
  glassBorder: "border border-border",
  glassCard: "bg-surface border border-border rounded-lg",
  glassInput: "bg-transparent border-b border-border-visible focus:border-text-primary transition-colors",
};

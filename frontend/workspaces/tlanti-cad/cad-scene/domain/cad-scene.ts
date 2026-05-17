export type CadSceneTool = 'select' | 'inspect' | 'section' | 'measure';

export type CadSceneEntityKind = 'mesh' | 'dicom-volume' | 'ai-segmentation' | 'export-target';
export type CadSceneClinicalRole = 'prep-scan' | 'antagonist' | 'margin' | 'crown-artifact' | 'dicom-reference';

export interface CadSceneEntity {
  id: string;
  label: string;
  kind: CadSceneEntityKind;
  clinicalRole: CadSceneClinicalRole;
  visible: boolean;
  locked: boolean;
  triangleBudget: number;
  renderMode: 'solid' | 'wireframe' | 'xray';
  transform: {
    position: [number, number, number];
    rotation: [number, number, number];
    scale: [number, number, number];
  };
}

export interface CadScenePerformanceProfile {
  renderQuality: 'balanced' | 'high-detail';
  targetFps: 60 | 90;
  dprMax: 1 | 1.25 | 1.5;
  shadowsEnabled: boolean;
}

export interface CadExtensionPoint {
  id: string;
  layer: 'rust' | 'python' | 'tauri' | 'three' | 'react';
  label: string;
  status: 'ready' | 'planned' | 'disabled';
  notes: string;
}

export interface CadShellBootstrap {
  route: string;
  offlineRequired: boolean;
  capabilities: {
    rustMeshOps: boolean;
    pythonAi: boolean;
    dicomPipeline: boolean;
    exportPipeline: boolean;
  };
  extensionPoints: CadExtensionPoint[];
}

export interface CadSceneStateSnapshot {
  activeTool: CadSceneTool;
  selectedEntityId: string | null;
  gridVisible: boolean;
  sceneRevision: number;
  entities: CadSceneEntity[];
  performance: CadScenePerformanceProfile;
  bootstrap: CadShellBootstrap | null;
  bootstrapStatus: 'idle' | 'loading' | 'ready' | 'error';
  bootstrapError: string | null;
}

export const CAD_SCENE_FIXTURE_ENTITIES: CadSceneEntity[] = [
  {
    id: 'fixture-upper-scan',
    label: '02 - UpperJaw - Single Full',
    kind: 'mesh',
    clinicalRole: 'prep-scan',
    visible: true,
    locked: false,
    triangleBudget: 150_154,
    renderMode: 'solid',
    transform: {
      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1.18, 1, 1],
    },
  },
  {
    id: 'fixture-antagonist-scan',
    label: 'Antagonist bite reference',
    kind: 'mesh',
    clinicalRole: 'antagonist',
    visible: true,
    locked: false,
    triangleBudget: 299_510,
    renderMode: 'solid',
    transform: {
      position: [0, 0.72, -0.04],
      rotation: [0.02, 0, 0],
      scale: [0.62, 0.46, 0.52],
    },
  },
  {
    id: 'fixture-margin-preview',
    label: 'Margin extension point',
    kind: 'ai-segmentation',
    clinicalRole: 'margin',
    visible: true,
    locked: true,
    triangleBudget: 1_200,
    renderMode: 'wireframe',
    transform: {
      position: [0, 0.18, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
    },
  },
];

export const DEFAULT_CAD_SCENE_PERFORMANCE: CadScenePerformanceProfile = {
  renderQuality: 'balanced',
  targetFps: 60,
  dprMax: 1.25,
  shadowsEnabled: false,
};

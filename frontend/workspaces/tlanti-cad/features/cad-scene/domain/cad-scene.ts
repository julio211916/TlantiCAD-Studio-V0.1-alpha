export type CadSceneTool = 'select' | 'inspect' | 'section' | 'measure';

export type CadSceneEntityKind = 'mesh' | 'dicom-volume' | 'ai-segmentation' | 'export-target';

export interface CadSceneEntity {
  id: string;
  label: string;
  kind: CadSceneEntityKind;
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
    label: 'Upper scan fixture',
    kind: 'mesh',
    visible: true,
    locked: false,
    triangleBudget: 12_000,
    renderMode: 'solid',
    transform: {
      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
    },
  },
  {
    id: 'fixture-margin-preview',
    label: 'Margin extension point',
    kind: 'ai-segmentation',
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

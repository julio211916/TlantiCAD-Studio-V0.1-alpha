import { invoke } from '@tauri-apps/api/core';

import { isTauriRuntime } from '@/platform/desktop-system';

export type CadRuntimeLayer = 'react' | 'three' | 'tauri' | 'rust' | 'python';
export type CadRuntimeStatus = 'ready' | 'planned' | 'disabled';

export interface CadRuntimeExtensionPoint {
  id: string;
  layer: CadRuntimeLayer;
  label: string;
  status: CadRuntimeStatus;
  notes: string;
}

export interface CadEditorRuntimeStatus {
  route: string;
  offlineRequired: boolean;
  capabilities: {
    rustMeshOps: boolean;
    pythonAi: boolean;
    dicomPipeline: boolean;
    exportPipeline: boolean;
  };
  extensionPoints: CadRuntimeExtensionPoint[];
}

const browserFallbackRuntime: CadEditorRuntimeStatus = {
  route: 'cad-shell/browser-fallback',
  offlineRequired: true,
  capabilities: {
    rustMeshOps: false,
    pythonAi: false,
    dicomPipeline: false,
    exportPipeline: false,
  },
  extensionPoints: [
    {
      id: 'react-cad-shell',
      layer: 'react',
      label: 'React CAD UI',
      status: 'ready',
      notes: 'Browser fallback owns editor controls only.',
    },
    {
      id: 'three-viewport',
      layer: 'three',
      label: 'Three.js viewport',
      status: 'ready',
      notes: 'Browser fallback renders the editor viewport without desktop compute.',
    },
    {
      id: 'tauri-orchestrator',
      layer: 'tauri',
      label: 'Tauri orchestration',
      status: 'disabled',
      notes: 'Desktop runtime was not detected; Rust crates and Python AI commands are unavailable.',
    },
  ],
};

export async function loadCadEditorRuntimeStatus(input: {
  caseId?: string | null;
  moduleId?: string | null;
} = {}): Promise<CadEditorRuntimeStatus> {
  if (!isTauriRuntime()) {
    return browserFallbackRuntime;
  }

  return invoke<CadEditorRuntimeStatus>('cad_shell_bootstrap', {
    request: {
      caseId: input.caseId ?? null,
      moduleId: input.moduleId ?? null,
    },
  });
}

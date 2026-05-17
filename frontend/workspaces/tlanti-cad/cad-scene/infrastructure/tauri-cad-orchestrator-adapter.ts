import { invoke } from '@tauri-apps/api/core';

import type { CadOrchestratorPort } from '../application/cad-orchestrator-port';
import type { CadShellBootstrap } from '../domain/cad-scene';
import { isTauriRuntime } from '@/lib/desktop-system';

const browserFallbackBootstrap: CadShellBootstrap = {
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
      id: 'react-shell',
      layer: 'react',
      label: 'React CAD shell',
      status: 'ready',
      notes: 'Browser fallback: UI shell is active without desktop orchestration.',
    },
    {
      id: 'three-viewport',
      layer: 'three',
      label: 'Three.js viewport',
      status: 'ready',
      notes: 'Lazy-loaded viewport renders fixture geometry locally.',
    },
    {
      id: 'tauri-orchestrator',
      layer: 'tauri',
      label: 'Tauri orchestrator',
      status: 'disabled',
      notes: 'Desktop runtime was not detected.',
    },
  ],
};

export function createTauriCadOrchestratorAdapter(): CadOrchestratorPort {
  return {
    async bootstrapShell(input) {
      if (!isTauriRuntime()) {
        return browserFallbackBootstrap;
      }

      return invoke<CadShellBootstrap>('cad_shell_bootstrap', {
        request: {
          caseId: input.caseId ?? null,
          moduleId: input.moduleId ?? null,
        },
      });
    },
  };
}

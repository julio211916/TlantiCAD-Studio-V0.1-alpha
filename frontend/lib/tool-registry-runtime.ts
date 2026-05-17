import { invoke } from '@tauri-apps/api/core';

import { isTauriRuntime } from '@/platform/desktop-system';

export interface RuntimeToolRegistryTool {
  id: string;
  label: string;
  category: string;
  owner: string;
  permissions: string[];
  requiredAssets: string[];
  jobKinds: string[];
  performanceRule: string;
}

export interface RuntimeToolRegistryModule {
  id: string;
  label: string;
  owner: string;
  purpose: string;
  tools: string[];
  permissions: string[];
  dependencies: string[];
  outputAssets: string[];
}

export interface RuntimeToolRegistrySnapshot {
  tools: RuntimeToolRegistryTool[];
  modules: RuntimeToolRegistryModule[];
}

const browserFallbackRegistry: RuntimeToolRegistrySnapshot = {
  tools: [],
  modules: [],
};

export async function loadRuntimeToolRegistry(): Promise<RuntimeToolRegistrySnapshot> {
  if (!isTauriRuntime()) {
    return browserFallbackRegistry;
  }

  return invoke<RuntimeToolRegistrySnapshot>('tool_registry_get');
}

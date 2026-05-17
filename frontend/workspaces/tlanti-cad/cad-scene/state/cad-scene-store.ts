import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import {
  CAD_SCENE_FIXTURE_ENTITIES,
  DEFAULT_CAD_SCENE_PERFORMANCE,
  type CadSceneEntity,
  type CadSceneStateSnapshot,
  type CadSceneTool,
  type CadShellBootstrap,
} from '../domain/cad-scene';

interface CadSceneActions {
  setActiveTool(tool: CadSceneTool): void;
  selectEntity(entityId: string | null): void;
  toggleGrid(): void;
  toggleEntityVisibility(entityId: string): void;
  setBootstrapLoading(): void;
  setBootstrapReady(bootstrap: CadShellBootstrap): void;
  setBootstrapError(error: string): void;
  resetFixtureScene(): void;
}

export type CadSceneStore = CadSceneStateSnapshot & CadSceneActions;

function cloneFixtureEntities(): CadSceneEntity[] {
  return CAD_SCENE_FIXTURE_ENTITIES.map((entity) => ({
    ...entity,
    transform: {
      position: [...entity.transform.position],
      rotation: [...entity.transform.rotation],
      scale: [...entity.transform.scale],
    },
  }));
}

export const cadSceneStore = createStore<CadSceneStore>((set) => ({
  activeTool: 'select',
  selectedEntityId: 'fixture-upper-scan',
  gridVisible: true,
  sceneRevision: 0,
  entities: cloneFixtureEntities(),
  performance: DEFAULT_CAD_SCENE_PERFORMANCE,
  bootstrap: null,
  bootstrapStatus: 'idle',
  bootstrapError: null,
  setActiveTool: (tool) => set((state) => ({ activeTool: tool, sceneRevision: state.sceneRevision + 1 })),
  selectEntity: (entityId) => set((state) => ({ selectedEntityId: entityId, sceneRevision: state.sceneRevision + 1 })),
  toggleGrid: () => set((state) => ({ gridVisible: !state.gridVisible, sceneRevision: state.sceneRevision + 1 })),
  toggleEntityVisibility: (entityId) => set((state) => ({
    entities: state.entities.map((entity) => (
      entity.id === entityId ? { ...entity, visible: !entity.visible } : entity
    )),
    sceneRevision: state.sceneRevision + 1,
  })),
  setBootstrapLoading: () => set({ bootstrapStatus: 'loading', bootstrapError: null }),
  setBootstrapReady: (bootstrap) => set({ bootstrap, bootstrapStatus: 'ready', bootstrapError: null }),
  setBootstrapError: (error) => set({ bootstrapStatus: 'error', bootstrapError: error }),
  resetFixtureScene: () => set((state) => ({
    activeTool: 'select',
    selectedEntityId: 'fixture-upper-scan',
    gridVisible: true,
    entities: cloneFixtureEntities(),
    sceneRevision: state.sceneRevision + 1,
  })),
}));

export function useCadSceneStore<T>(selector: (state: CadSceneStore) => T): T {
  return useStore(cadSceneStore, selector);
}

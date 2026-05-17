import { WORKSPACE_TITLES, type AppWorkspaceId } from '@/components/workspaces/workspace.config';
import type { WorkspaceLaunchContext } from '@/lib/workspace-routing';

export type WorkspacePreloaderPhase = 'boot' | 'sync' | 'transition' | 'lazy-load';

const MODULE_TITLES: Record<string, string> = {
  cad: 'CAD Design',
  dicom: 'DICOM Planning',
  implant: 'Implant Planning',
  guide: 'Surgical Guide',
  splint: 'Splint Workflow',
  smile: 'Smile Design',
  ceph: 'Cephalometry',
  fab: 'Manufacturing',
  aligners: 'Aligners',
  odontogram: 'Odontogram',
  'model-creator': 'Model Creator',
  partials: 'Partials',
};

export function getWorkspaceTitle(workspace: AppWorkspaceId): string {
  return workspace === 'tlanticad' ? WORKSPACE_TITLES.tlanticad : WORKSPACE_TITLES.tlantidb;
}

export function getWorkspaceModuleTitle(moduleId?: string): string {
  if (!moduleId) return WORKSPACE_TITLES.tlanticad;
  return MODULE_TITLES[moduleId] ?? moduleId.replace(/-/g, ' ');
}

export function describeWorkspacePreload(context: WorkspaceLaunchContext, phase: WorkspacePreloaderPhase): string {
  if (phase === 'boot') return WORKSPACE_TITLES.boot;
  if (phase === 'sync') return 'Syncing case workspace';
  if (context.workspace === 'tlanticad') return getWorkspaceModuleTitle(context.module);
  return getWorkspaceTitle(context.workspace);
}

export function buildWorkspacePreloaderState(context: WorkspaceLaunchContext, phase: WorkspacePreloaderPhase) {
  return {
    phase,
    workspace: context.workspace,
    caseId: context.caseId,
    moduleId: context.module,
    subtitle: describeWorkspacePreload(context, phase),
  };
}

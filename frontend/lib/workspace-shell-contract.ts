export type WorkspaceHeaderSection = 'case' | 'patient' | 'clinic' | 'assets';

export type CaseNavigationIntent =
  | 'activate'
  | 'create'
  | 'search-result'
  | 'file-action';

export interface WorkspaceContextSyncRequest {
  caseId?: string;
  moduleId?: string;
  intent: CaseNavigationIntent;
}

export type CadModuleSurface =
  | 'wizard'
  | 'odontogram'
  | 'dicom'
  | 'implant'
  | 'guide'
  | 'splint'
  | 'smile'
  | 'ceph'
  | 'fab'
  | 'aligners';

export type ModuleSurfaceRegistry = Record<CadModuleSurface, boolean>;

export function createEmptyModuleSurfaceRegistry(): ModuleSurfaceRegistry {
  return {
    wizard: false,
    odontogram: false,
    dicom: false,
    implant: false,
    guide: false,
    splint: false,
    smile: false,
    ceph: false,
    fab: false,
    aligners: false,
  };
}

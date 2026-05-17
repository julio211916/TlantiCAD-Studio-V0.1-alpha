export type TlantiModuleStage = 'intake' | 'imaging' | 'design' | 'review' | 'manufacture'

export type TlantiModuleId =
  | 'cad'
  | 'dicom'
  | 'model-creator'
  | 'partials'
  | 'implant'
  | 'guide'
  | 'splint'
  | 'ceph'
  | 'fab'
  | 'aligners'
  | 'orthocad'

export type FileActionId = 'export' | 'xml-interop' | 'folder-reveal' | 'snapshot' | 'print' | 'copy'

export type ToothNumber =
  | 11
  | 12
  | 13
  | 14
  | 15
  | 16
  | 17
  | 18
  | 21
  | 22
  | 23
  | 24
  | 25
  | 26
  | 27
  | 28
  | 31
  | 32
  | 33
  | 34
  | 35
  | 36
  | 37
  | 38
  | 41
  | 42
  | 43
  | 44
  | 45
  | 46
  | 47
  | 48

export interface TlantiModuleDefinition {
  id: TlantiModuleId
  label: string
  shortLabel: string
  description: string
  stage: TlantiModuleStage
  workflow: readonly string[]
  tools: readonly string[]
  aiCapabilities: readonly string[]
  isCore?: boolean
}

export interface FileActionDefinition {
  id: FileActionId
  label: string
  description: string
}

export interface ClinicalAsset {
  id: string
  name: string
  role: 'dicom' | 'scan' | 'mesh' | 'model' | 'report' | 'manufacturing' | 'photo' | 'other'
  localPath?: string | null
  toothNumbers?: readonly ToothNumber[]
  moduleId?: TlantiModuleId | null
  tags?: readonly string[]
  createdAt: string
}

export interface DentalCase {
  id: string
  caseNumber: string
  name: string
  activeModuleId: TlantiModuleId
  assets: readonly ClinicalAsset[]
  createdAt: string
  updatedAt: string
}

export interface ModuleWorkflowState {
  caseId: string
  moduleId: TlantiModuleId
  currentStepIndex: number
  completedSteps: readonly string[]
  updatedAt: string
}

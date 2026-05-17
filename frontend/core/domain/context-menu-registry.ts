import type { CadCommandId } from './cad-command'
import type { TlantiModuleId } from './entities'

export type CadContextMenuModuleId = TlantiModuleId | 'cad' | 'smile' | 'ortho'
export type CadContextObjectKind =
  | 'cad-object'
  | 'tooth'
  | 'implant'
  | 'abutment'
  | 'sleeve'
  | 'scan-body'
  | 'nerve'
  | 'sinus'
  | 'segmentation'
  | 'measurement'
  | 'annotation'
  | 'dicom-series'
  | 'landmark'
  | 'aligner-stage'

export type CadContextSource =
  | { kind: 'viewport'; moduleId: CadContextMenuModuleId }
  | { kind: 'object'; moduleId: CadContextMenuModuleId; objectKind: CadContextObjectKind; objectId: string }
  | { kind: 'series'; moduleId: 'dicom'; seriesId?: string }

export interface CadContextMenuItem {
  id: string
  label: string
  commandId: CadCommandId
  toolId?: string
  shortcut?: string
  disabledReason?: string | null
  tone?: 'default' | 'primary' | 'danger'
  separator?: boolean
  children?: readonly CadContextMenuItem[]
}

export interface CadContextMenuProvider {
  resolve(source: CadContextSource): readonly CadContextMenuItem[]
}

export interface CadContextMenuRegistry {
  register(moduleId: CadContextMenuModuleId, provider: CadContextMenuProvider): void
  providerFor(moduleId: CadContextMenuModuleId): CadContextMenuProvider | null
  resolve(source: CadContextSource): readonly CadContextMenuItem[]
}

export function createCadContextMenuRegistry(): CadContextMenuRegistry {
  const providers = new Map<CadContextMenuModuleId, CadContextMenuProvider>()

  return {
    register(moduleId, provider) {
      providers.set(moduleId, provider)
    },
    providerFor(moduleId) {
      return providers.get(moduleId) ?? null
    },
    resolve(source) {
      return providers.get(source.moduleId)?.resolve(source) ?? []
    },
  }
}

export function item(input: CadContextMenuItem): CadContextMenuItem {
  return input
}

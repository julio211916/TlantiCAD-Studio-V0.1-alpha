import type { TlantiModuleId } from './entities'
import type { CadToolId } from './cad-tool-registry'
import { resolveCadToolDefinition } from './cad-tool-registry'

export type CadCommandOwner = 'react-ui' | 'three-render' | 'tauri-command' | 'rust-core' | 'python-sidecar'
export type CadCommandEffect = 'ui-state' | 'scene-preview' | 'case-state' | 'job-start' | 'asset-export'

export type CadCommandId =
  | `cad.tool.${string}`
  | `cad.view.${string}`
  | `cad.selection.${string}`
  | `mesh.${string}`
  | `dental.${string}`
  | `dicom.${string}`
  | `implant.${string}`
  | `guide.${string}`
  | `splint.${string}`
  | `ceph.${string}`
  | `ortho.${string}`
  | `manufacturing.${string}`

export interface CadCommandPayload {
  caseId?: string
  moduleId?: TlantiModuleId | string
  toolId?: CadToolId
  objectId?: string
  objectIds?: readonly string[]
  sourceId?: string
  params?: Record<string, unknown>
}

export interface CadCommand {
  id: CadCommandId
  label: string
  owner: CadCommandOwner
  effect: CadCommandEffect
  payload: CadCommandPayload
  createdAt: string
}

export interface CadCommandValidationResult {
  ok: boolean
  issues: readonly string[]
}

export interface CadCommandRunResult {
  accepted: boolean
  command: CadCommand
  queuedJobKind?: string
  issues: readonly string[]
}

export function createCadCommand(input: {
  id: CadCommandId
  label?: string
  owner: CadCommandOwner
  effect: CadCommandEffect
  payload?: CadCommandPayload
  now?: Date
}): CadCommand {
  const tool = input.payload?.toolId ? resolveCadToolDefinition(input.payload.toolId) : null
  return {
    id: input.id,
    label: input.label ?? tool?.label ?? input.id,
    owner: input.owner,
    effect: input.effect,
    payload: input.payload ?? {},
    createdAt: (input.now ?? new Date()).toISOString(),
  }
}

export function createCadToolCommand(toolId: string, payload: Omit<CadCommandPayload, 'toolId'> = {}, now?: Date): CadCommand {
  const tool = resolveCadToolDefinition(toolId)
  if (!tool) {
    return createCadCommand({
      id: `cad.tool.${toolId}` as CadCommandId,
      label: toolId,
      owner: 'react-ui',
      effect: 'ui-state',
      payload: { ...payload, toolId },
      now,
    })
  }

  const effect: CadCommandEffect = tool.capabilities.includes('asset-export')
    ? 'asset-export'
    : tool.capabilities.includes('job')
      ? 'job-start'
      : tool.owner === 'three-render'
        ? 'scene-preview'
        : 'ui-state'

  return createCadCommand({
    id: tool.commandId as CadCommandId,
    label: tool.label,
    owner: tool.owner,
    effect,
    payload: { ...payload, toolId: tool.id },
    now,
  })
}

export function validateCadCommand(command: CadCommand): CadCommandValidationResult {
  const issues: string[] = []
  const payloadByteLength = new TextEncoder().encode(JSON.stringify(command.payload)).byteLength

  if (!command.id.trim()) issues.push('Command id is required')
  if (!command.label.trim()) issues.push(`${command.id} missing label`)
  if (payloadByteLength > 16 * 1024) issues.push(`${command.id} payload exceeds 16KB (${payloadByteLength} bytes)`)

  if (command.payload.toolId) {
    const tool = resolveCadToolDefinition(command.payload.toolId)
    if (!tool) {
      issues.push(`${command.id} references unknown tool ${command.payload.toolId}`)
    } else {
      if (tool.requiresActiveMesh && !command.payload.objectId && !command.payload.objectIds?.length && !command.payload.sourceId) {
        issues.push(`${command.id} requires an active mesh/object/source ref`)
      }
      if (tool.requiresActiveDicom && !command.payload.sourceId) {
        issues.push(`${command.id} requires an active DICOM source ref`)
      }
    }
  }

  return { ok: issues.length === 0, issues }
}

export function runCadCommandInMemory(command: CadCommand): CadCommandRunResult {
  const validation = validateCadCommand(command)
  if (!validation.ok) {
    return { accepted: false, command, issues: validation.issues }
  }

  return {
    accepted: true,
    command,
    queuedJobKind: command.effect === 'job-start' ? command.id : undefined,
    issues: [],
  }
}

import { parseUrl } from '../../utils/url'
import type { MeshUriResolution } from '../domain/mesh-engine'

const REMOTE_URL_PATTERN = /^https?:\/\//i
const LOOPBACK_HOSTS = new Set(['127.0.0.1', 'localhost'])
const ABSOLUTE_PATH_PATTERN = /^(\/|[a-zA-Z]:[\\/])/

export function resolveLocalMeshUri(input: string): MeshUriResolution {
  const trimmed = input.trim()

  if (/^(mesh-vault|asset):\/\//i.test(trimmed)) {
    const parsed = parseUrl(trimmed)
    return {
      input,
      scheme: parsed.protocol.startsWith('mesh-vault') ? 'mesh-vault' : 'asset',
      localOnly: true,
      normalized: parsed.toString(),
    }
  }

  if (/^file:\/\//i.test(trimmed)) {
    const parsed = parseUrl(trimmed)
    return {
      input,
      scheme: 'file',
      localOnly: true,
      normalized: decodeURI(parsed.pathname),
    }
  }

  if (REMOTE_URL_PATTERN.test(trimmed)) {
    const parsed = parseUrl(trimmed)
    const localOnly = LOOPBACK_HOSTS.has(parsed.hostname)
    return {
      input,
      scheme: localOnly ? 'loopback' : 'blocked-remote',
      localOnly,
      normalized: parsed.toString(),
    }
  }

  if (ABSOLUTE_PATH_PATTERN.test(trimmed)) {
    return { input, scheme: 'absolute-path', localOnly: true, normalized: trimmed }
  }

  return { input, scheme: 'relative-path', localOnly: true, normalized: trimmed }
}

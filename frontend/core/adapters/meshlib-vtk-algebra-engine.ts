import type { MeshEngineBackend, MeshEngineOperation, MeshProcessingPlan } from '../domain/mesh-engine'
import { resolveLocalMeshUri } from './local-mesh-uri-resolver'

export const TLANTI_MESH_ENGINE_BACKENDS: MeshEngineBackend[] = [
  {
    id: 'meshlib-wasm',
    title: 'MeshLib repair and topology engine',
    runtime: 'wasm-worker',
    sourceRoots: ['Tauri/meshlib', 'Tauri/meshlib-wasm/src/index.ts'],
    operations: ['boolean-union', 'boolean-difference', 'boolean-intersection', 'decimate', 'smooth', 'remesh', 'hole-fill', 'offset'],
    bufferPolicy: 'handle-backed',
    enabled: true,
  },
  {
    id: 'vtk-imaging',
    title: 'VTK imaging and volume bridge',
    runtime: 'vtk-worker',
    sourceRoots: ['frontend/io/vtk', 'frontend/workspaces/tlanti-cad/features/dicom-viewer', 'Tauri/backend/python/trame_slicer_sidecar.py'],
    operations: ['vtk-volume-preview', 'dicom-to-mesh'],
    bufferPolicy: 'preview-only',
    enabled: true,
  },
  {
    id: 'rust-algebra',
    title: 'Rust algebra and deterministic mesh math',
    runtime: 'rust-tauri',
    sourceRoots: ['Tauri/src/mesh', 'Tauri/src/tlanticad-mesh', 'Tauri/src/tlanticad-geometry'],
    operations: ['normals', 'bounds', 'transform'],
    bufferPolicy: 'path-backed',
    enabled: true,
  },
  {
    id: 'mesh-vault',
    title: 'Mesh Vault file ingress',
    runtime: 'asset-vault',
    sourceRoots: ['frontend/core/domain/mesh-vault.ts', 'frontend/core/adapters/tauri-mesh-vault.ts', 'Tauri/library'],
    operations: ['bounds', 'vtk-volume-preview'],
    bufferPolicy: 'path-backed',
    enabled: true,
  },
]

export function selectMeshBackend(operation: MeshEngineOperation): MeshEngineBackend {
  const backend = TLANTI_MESH_ENGINE_BACKENDS.find((candidate) => candidate.operations.includes(operation))
  if (!backend) {
    throw new Error(`No mesh backend registered for operation: ${operation}`)
  }
  return backend
}

export function createMeshProcessingPlan(sourceUri: string, operation: MeshEngineOperation): MeshProcessingPlan {
  const input = resolveLocalMeshUri(sourceUri)
  if (!input.localOnly) {
    throw new Error(`Remote mesh source rejected by offline contract: ${sourceUri}`)
  }

  const backend = selectMeshBackend(operation)
  return {
    input,
    operation,
    backend,
    expectedIo: backend.runtime === 'wasm-worker' ? 'worker-buffer' : backend.runtime === 'rust-tauri' ? 'path-stream' : 'manifest-only',
  }
}

export type MeshEngineBackendId = 'meshlib-wasm' | 'vtk-imaging' | 'rust-algebra' | 'mesh-vault'

export type MeshEngineOperation =
  | 'boolean-union'
  | 'boolean-difference'
  | 'boolean-intersection'
  | 'decimate'
  | 'smooth'
  | 'remesh'
  | 'hole-fill'
  | 'offset'
  | 'normals'
  | 'bounds'
  | 'transform'
  | 'vtk-volume-preview'
  | 'dicom-to-mesh'

export interface MeshEngineBackend {
  id: MeshEngineBackendId
  title: string
  runtime: 'wasm-worker' | 'vtk-worker' | 'rust-tauri' | 'asset-vault'
  sourceRoots: string[]
  operations: MeshEngineOperation[]
  bufferPolicy: 'path-backed' | 'handle-backed' | 'preview-only'
  enabled: boolean
}

export interface MeshUriResolution {
  input: string
  scheme: 'file' | 'absolute-path' | 'relative-path' | 'mesh-vault' | 'asset' | 'loopback' | 'blocked-remote'
  localOnly: boolean
  normalized: string
}

export interface MeshProcessingPlan {
  input: MeshUriResolution
  operation: MeshEngineOperation
  backend: MeshEngineBackend
  expectedIo: 'manifest-only' | 'path-stream' | 'worker-buffer'
}

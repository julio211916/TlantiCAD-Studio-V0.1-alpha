/**
 * MeshLib WebAssembly Bindings for TlantiCAD
 *
 * Provides TypeScript wrapper around MeshLib compiled to WASM.
 * MeshLib operations: boolean ops, remeshing, decimation, smoothing, hole fill, offset.
 */

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface MeshData {
  vertices: Float32Array;
  indices: Uint32Array;
  normals: Float32Array;
}

export interface BooleanResult {
  mesh: MeshData;
  elapsed_ms: number;
}

export interface DecimateOptions {
  targetRatio: number;
  maxError?: number;
}

export interface SmoothOptions {
  iterations: number;
  lambda: number;
}

export interface RemeshOptions {
  targetEdgeLength: number;
  iterations?: number;
}

export interface OffsetOptions {
  distance: number;
  resolution?: number;
}

/** MeshLib WASM Module interface (loaded via Emscripten) */
interface MeshLibModule {
  _ml_create_mesh(vertexPtr: number, vertexCount: number, indexPtr: number, indexCount: number): number;
  _ml_delete_mesh(meshId: number): void;
  _ml_boolean_union(meshA: number, meshB: number): number;
  _ml_boolean_difference(meshA: number, meshB: number): number;
  _ml_boolean_intersection(meshA: number, meshB: number): number;
  _ml_decimate(meshId: number, targetRatio: number): number;
  _ml_subdivide(meshId: number, iterations: number): number;
  _ml_smooth(meshId: number, iterations: number, lambda: number): number;
  _ml_remesh(meshId: number, targetEdgeLength: number): number;
  _ml_fill_holes(meshId: number): number;
  _ml_offset(meshId: number, distance: number): number;
  _ml_get_vertex_count(meshId: number): number;
  _ml_get_face_count(meshId: number): number;
  _ml_get_vertices(meshId: number, outPtr: number): void;
  _ml_get_indices(meshId: number, outPtr: number): void;
  _ml_get_normals(meshId: number, outPtr: number): void;
  _malloc(size: number): number;
  _free(ptr: number): void;
  HEAPF32: Float32Array;
  HEAPU32: Uint32Array;
}

let wasmModule: MeshLibModule | null = null;

/**
 * Initialize MeshLib WASM module.
 * Call this once before using any mesh operations.
 */
export async function initMeshLib(wasmUrl?: string): Promise<void> {
  if (wasmModule) return;

  const url = wasmUrl ?? '/meshlib.js';
  try {
    const script = document.createElement('script');
    script.src = url;
    await new Promise<void>((resolve, reject) => {
      script.onload = () => resolve();
      script.onerror = () => reject(new Error(`Failed to load MeshLib WASM from ${url}`));
      document.head.appendChild(script);
    });
    // MeshLib Emscripten module attaches to window.Module
    wasmModule = (globalThis as Record<string, unknown>).Module as MeshLibModule;
    console.log('[MeshLib] WASM module loaded');
  } catch (e) {
    console.warn('[MeshLib] WASM not available, using fallback Rust operations', e);
  }
}

/** Check if WASM module is loaded */
export function isMeshLibReady(): boolean {
  return wasmModule !== null;
}

/** Helper to allocate mesh in WASM memory */
function allocateMesh(data: MeshData): number {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const vertexBytes = data.vertices.byteLength;
  const indexBytes = data.indices.byteLength;

  const vertexPtr = wasmModule._malloc(vertexBytes);
  const indexPtr = wasmModule._malloc(indexBytes);

  wasmModule.HEAPF32.set(data.vertices, vertexPtr / 4);
  wasmModule.HEAPU32.set(data.indices, indexPtr / 4);

  const meshId = wasmModule._ml_create_mesh(vertexPtr, data.vertices.length / 3, indexPtr, data.indices.length);

  wasmModule._free(vertexPtr);
  wasmModule._free(indexPtr);

  return meshId;
}

/** Extract mesh data from WASM */
function extractMesh(meshId: number): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const vertexCount = wasmModule._ml_get_vertex_count(meshId);
  const faceCount = wasmModule._ml_get_face_count(meshId);

  const vertexSize = vertexCount * 3;
  const indexSize = faceCount * 3;
  const normalSize = vertexCount * 3;

  const vPtr = wasmModule._malloc(vertexSize * 4);
  const iPtr = wasmModule._malloc(indexSize * 4);
  const nPtr = wasmModule._malloc(normalSize * 4);

  wasmModule._ml_get_vertices(meshId, vPtr);
  wasmModule._ml_get_indices(meshId, iPtr);
  wasmModule._ml_get_normals(meshId, nPtr);

  const vertices = new Float32Array(wasmModule.HEAPF32.buffer, vPtr, vertexSize).slice();
  const indices = new Uint32Array(wasmModule.HEAPU32.buffer, iPtr, indexSize).slice();
  const normals = new Float32Array(wasmModule.HEAPF32.buffer, nPtr, normalSize).slice();

  wasmModule._free(vPtr);
  wasmModule._free(iPtr);
  wasmModule._free(nPtr);

  return { vertices, indices, normals };
}

/** Boolean union of two meshes */
export function booleanUnion(meshA: MeshData, meshB: MeshData): BooleanResult {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');
  const start = performance.now();

  const idA = allocateMesh(meshA);
  const idB = allocateMesh(meshB);
  const resultId = wasmModule._ml_boolean_union(idA, idB);
  const mesh = extractMesh(resultId);

  wasmModule._ml_delete_mesh(idA);
  wasmModule._ml_delete_mesh(idB);
  wasmModule._ml_delete_mesh(resultId);

  return { mesh, elapsed_ms: performance.now() - start };
}

/** Boolean difference (A - B) */
export function booleanDifference(meshA: MeshData, meshB: MeshData): BooleanResult {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');
  const start = performance.now();

  const idA = allocateMesh(meshA);
  const idB = allocateMesh(meshB);
  const resultId = wasmModule._ml_boolean_difference(idA, idB);
  const mesh = extractMesh(resultId);

  wasmModule._ml_delete_mesh(idA);
  wasmModule._ml_delete_mesh(idB);
  wasmModule._ml_delete_mesh(resultId);

  return { mesh, elapsed_ms: performance.now() - start };
}

/** Boolean intersection */
export function booleanIntersection(meshA: MeshData, meshB: MeshData): BooleanResult {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');
  const start = performance.now();

  const idA = allocateMesh(meshA);
  const idB = allocateMesh(meshB);
  const resultId = wasmModule._ml_boolean_intersection(idA, idB);
  const mesh = extractMesh(resultId);

  wasmModule._ml_delete_mesh(idA);
  wasmModule._ml_delete_mesh(idB);
  wasmModule._ml_delete_mesh(resultId);

  return { mesh, elapsed_ms: performance.now() - start };
}

/** Decimate mesh to target ratio */
export function decimate(mesh: MeshData, opts: DecimateOptions): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const id = allocateMesh(mesh);
  const resultId = wasmModule._ml_decimate(id, opts.targetRatio);
  const result = extractMesh(resultId);

  wasmModule._ml_delete_mesh(id);
  wasmModule._ml_delete_mesh(resultId);

  return result;
}

/** Smooth mesh */
export function smooth(mesh: MeshData, opts: SmoothOptions): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const id = allocateMesh(mesh);
  const resultId = wasmModule._ml_smooth(id, opts.iterations, opts.lambda);
  const result = extractMesh(resultId);

  wasmModule._ml_delete_mesh(id);
  wasmModule._ml_delete_mesh(resultId);

  return result;
}

/** Remesh with target edge length */
export function remesh(mesh: MeshData, opts: RemeshOptions): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const id = allocateMesh(mesh);
  const resultId = wasmModule._ml_remesh(id, opts.targetEdgeLength);
  const result = extractMesh(resultId);

  wasmModule._ml_delete_mesh(id);
  wasmModule._ml_delete_mesh(resultId);

  return result;
}

/** Fill holes in mesh */
export function fillHoles(mesh: MeshData): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const id = allocateMesh(mesh);
  const resultId = wasmModule._ml_fill_holes(id);
  const result = extractMesh(resultId);

  wasmModule._ml_delete_mesh(id);
  wasmModule._ml_delete_mesh(resultId);

  return result;
}

/** Offset mesh surface */
export function offset(mesh: MeshData, opts: OffsetOptions): MeshData {
  if (!wasmModule) throw new Error('MeshLib WASM not initialized');

  const id = allocateMesh(mesh);
  const resultId = wasmModule._ml_offset(id, opts.distance);
  const result = extractMesh(resultId);

  wasmModule._ml_delete_mesh(id);
  wasmModule._ml_delete_mesh(resultId);

  return result;
}

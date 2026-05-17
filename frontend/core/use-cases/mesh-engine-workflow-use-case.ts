import type { MeshEngineBackend, MeshEngineOperation, MeshProcessingPlan } from '../domain/mesh-engine'
import { TLANTI_MESH_ENGINE_BACKENDS, createMeshProcessingPlan } from '../adapters/meshlib-vtk-algebra-engine'

export interface MeshEngineWorkflowRequest {
  sourceUri: string
  operation: MeshEngineOperation
}

export interface MeshEngineWorkflowResult {
  plan: MeshProcessingPlan
  backends: MeshEngineBackend[]
}

export class MeshEngineWorkflowUseCase {
  listBackends(): MeshEngineBackend[] {
    return TLANTI_MESH_ENGINE_BACKENDS
  }

  plan(request: MeshEngineWorkflowRequest): MeshEngineWorkflowResult {
    return {
      plan: createMeshProcessingPlan(request.sourceUri, request.operation),
      backends: this.listBackends(),
    }
  }
}

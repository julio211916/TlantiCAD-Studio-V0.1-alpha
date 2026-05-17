export interface LocalImagingSeriesRequest {
  sourcePath: string
  caseId?: string
  modalityHint?: 'dicom' | 'vtk' | 'nifti' | 'unknown'
}

export interface LocalImagingSeriesManifest {
  sourcePath: string
  caseId?: string
  engine: 'trame-slicer-local'
  accepted: true
  capabilities: Array<'dicom-metadata' | 'mpr' | 'volume-rendering' | 'segmentation-preview' | 'mesh-handoff'>
  sourceRoots: string[]
}

export interface LocalImagingEnginePort {
  openSeries(request: LocalImagingSeriesRequest): Promise<LocalImagingSeriesManifest>
}

const REMOTE_ROUTE_PATTERN = /^(https?:)?\/\//i

export class VolViewLocalImagingEngine implements LocalImagingEnginePort {
  async openSeries(request: LocalImagingSeriesRequest): Promise<LocalImagingSeriesManifest> {
    if (REMOTE_ROUTE_PATTERN.test(request.sourcePath)) {
      throw new Error(`Clinical imaging is offline-only; remote source rejected: ${request.sourcePath}`)
    }

    return {
      sourcePath: request.sourcePath,
      caseId: request.caseId,
      engine: 'trame-slicer-local',
      accepted: true,
      capabilities: ['dicom-metadata', 'mpr', 'volume-rendering', 'segmentation-preview', 'mesh-handoff'],
      sourceRoots: [
        'frontend/io',
        'frontend/composables',
        'frontend/workspaces/tlanti-cad/features/dicom-viewer',
        'Tauri/backend/python/trame_slicer_sidecar.py',
      ],
    }
  }
}

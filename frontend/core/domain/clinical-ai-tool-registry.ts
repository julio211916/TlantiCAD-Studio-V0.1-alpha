import type { TlantiModuleId } from './entities'

export type ClinicalAiRuntimeOwner = 'python-sidecar' | 'rust-tauri' | 'three-renderer'

export type ClinicalAiToolId =
  | 'jawmotionai-contact-aware'
  | 'slicer-dentalsegmentator'
  | 'slicer-automated-dental-tools'
  | 'tooth-group-network'
  | 'dentist-sota-pxi-r2unet'
  | 'dentist-sota-cbct-r2unet'
  | 'dentist-sota-dicom-pii-stripper'
  | 'pydicom-metadata-preview'
  | 'pytorch-dental-inference'

export interface ClinicalAiToolDefinition {
  id: ClinicalAiToolId
  label: string
  owner: ClinicalAiRuntimeOwner
  vendorPath?: string
  modules: readonly TlantiModuleId[]
  inputs: readonly string[]
  outputs: readonly string[]
  workflow: readonly string[]
  uiPlacement: 'jobs-panel' | 'dicom-panel' | 'mesh-tools' | 'landmarks-panel'
  runtimeRule: string
}

export const CLINICAL_AI_TOOL_DEFINITIONS: readonly ClinicalAiToolDefinition[] = [
  {
    id: 'jawmotionai-contact-aware',
    label: 'JawMotionAI Contact-Aware Motion',
    owner: 'python-sidecar',
    vendorPath:
      'backend/.tlanticad/python/vendors/JawMotionAI',
    modules: ['cad', 'model-creator', 'splint', 'implant', 'guide'],
    inputs: ['maxilla mesh asset', 'mandible mesh asset', 'incisal and condyle marks'],
    outputs: ['jaw motion tracks', 'frame transforms', 'exocad XML artifact'],
    workflow: ['Load meshes', 'Validate marks', 'Generate motion', 'Resolve contact', 'Persist XML/result'],
    uiPlacement: 'landmarks-panel',
    runtimeRule:
      'Run only in the Python sidecar. React submits marks and asset ids; Three.js previews tracks without recomputing collisions.',
  },
  {
    id: 'slicer-dentalsegmentator',
    label: 'Slicer DentalSegmentator',
    owner: 'python-sidecar',
    vendorPath:
      'backend/.tlanticad/python/vendors/SlicerDentalSegmentator/DentalSegmentator',
    modules: ['dicom', 'implant', 'guide', 'ceph', 'aligners'],
    inputs: ['CBCT DICOM folder', 'DICOM series id'],
    outputs: ['segmentation mask', 'jaw/tooth label map', 'preview mesh'],
    workflow: ['Load DICOM', 'Preprocess volume', 'Infer labels', 'Postprocess mask', 'Export mesh handoff'],
    uiPlacement: 'dicom-panel',
    runtimeRule:
      'Run only through the Python sidecar job queue. React may request the job but must not parse DICOM or run inference.',
  },
  {
    id: 'slicer-automated-dental-tools',
    label: 'Slicer Automated Dental Tools',
    owner: 'python-sidecar',
    vendorPath:
      'backend/.tlanticad/python/vendors/SlicerAutomatedDentalTools',
    modules: ['dicom', 'cad', 'model-creator', 'orthocad', 'aligners'],
    inputs: ['STL/OBJ mesh', 'CBCT-derived mesh', 'tooth labels'],
    outputs: ['landmarks', 'oriented model', 'diagnostic annotations'],
    workflow: ['Load model', 'Detect landmarks', 'Validate orientation', 'Persist annotations'],
    uiPlacement: 'landmarks-panel',
    runtimeRule:
      'Run as an async Python workflow. Persist only job refs, landmarks, and assets through Tauri repositories.',
  },
  {
    id: 'tooth-group-network',
    label: 'Tooth Group Network',
    owner: 'python-sidecar',
    modules: ['cad', 'model-creator', 'partials', 'orthocad', 'aligners'],
    inputs: ['OBJ/STL dental arch mesh'],
    outputs: ['tooth groups', 'mesh labels', 'editable selection sets'],
    workflow: ['Load mesh', 'Infer tooth groups', 'Map labels to mesh ids', 'Review manually'],
    uiPlacement: 'mesh-tools',
    runtimeRule:
      'Use Python/Torch for inference; Three.js receives optimized buffers and labels for rendering only.',
  },
  {
    id: 'dentist-sota-pxi-r2unet',
    label: 'Dentist-SOTA PXI R2U-Net',
    owner: 'python-sidecar',
    vendorPath:
      '.tlanticad/python/vendors/Dentist-SOTA/ai',
    modules: ['dicom', 'orthocad', 'ceph'],
    inputs: ['panoramic X-ray image', 'STS-style PNG/JPG study'],
    outputs: ['binary tooth mask', 'boundary-aware segmentation', 'confidence metrics'],
    workflow: ['Load PXI image', 'Normalize to STS-Tooth convention', 'Infer with R2U-Net', 'Review mask', 'Persist result'],
    uiPlacement: 'dicom-panel',
    runtimeRule:
      'Prefer this route for 2D panoramic X-rays. Use MONAI/PyTorch in the Python sidecar and keep visualization-only overlays in React.',
  },
  {
    id: 'dentist-sota-cbct-r2unet',
    label: 'Dentist-SOTA CBCT R2U-Net',
    owner: 'python-sidecar',
    vendorPath:
      '.tlanticad/python/vendors/Dentist-SOTA/ai',
    modules: ['dicom', 'implant', 'guide', 'aligners', 'ceph'],
    inputs: ['CBCT DICOM folder', 'NIfTI volume', 'resampled study tensor'],
    outputs: ['segmentation volume', 'sliding-window mask', 'FDI candidate labels'],
    workflow: ['Sanitize DICOM', 'Resample to isotropic spacing', 'Run sliding-window inference', 'Postprocess mask', 'Queue review/export'],
    uiPlacement: 'jobs-panel',
    runtimeRule:
      'Treat as a research-grade volumetric route. Keep it behind async jobs until model packaging, calibration and benchmark validation are complete.',
  },
  {
    id: 'dentist-sota-dicom-pii-stripper',
    label: 'Dentist-SOTA DICOM PII Strip',
    owner: 'python-sidecar',
    vendorPath:
      '.tlanticad/python/vendors/Dentist-SOTA/backend',
    modules: ['dicom', 'implant', 'guide', 'ceph'],
    inputs: ['DICOM file upload', 'DICOM series'],
    outputs: ['sanitized metadata', 'safe upload payload', 'modality and image size summary'],
    workflow: ['Receive DICOM', 'Strip tags', 'Validate modality', 'Return safe metadata'],
    uiPlacement: 'jobs-panel',
    runtimeRule:
      'Use as a safe-ingestion helper before persistence or external sharing. This route complements, not replaces, TlantiCAD SQLite/Tauri ownership of clinical state.',
  },
  {
    id: 'pydicom-metadata-preview',
    label: 'PyDICOM Metadata and Preview',
    owner: 'python-sidecar',
    modules: ['dicom', 'implant', 'guide', 'ceph'],
    inputs: ['DICOM file', 'DICOM folder'],
    outputs: ['metadata JSON', 'preview slices', 'series graph'],
    workflow: ['Inspect headers', 'Group series', 'Generate thumbnails', 'Persist metadata'],
    uiPlacement: 'jobs-panel',
    runtimeRule:
      'Use PyDICOM for metadata and codec-sensitive previews; Rust can later add dicom-rs fast-path metadata.',
  },
  {
    id: 'pytorch-dental-inference',
    label: 'PyTorch Dental Inference',
    owner: 'python-sidecar',
    modules: ['dicom', 'cad', 'model-creator', 'implant', 'guide', 'aligners'],
    inputs: ['volume tensor', 'mesh tensor', 'preprocessed image stack'],
    outputs: ['segmentation result', 'landmarks', 'confidence map'],
    workflow: ['Load model', 'Preprocess', 'Infer', 'Postprocess', 'Queue review'],
    uiPlacement: 'jobs-panel',
    runtimeRule:
      'Prefer ONNX Runtime when models are exportable; TorchScript/PyTorch remains fallback for advanced models.',
  },
] as const

export function listClinicalAiToolsForModule(moduleId: TlantiModuleId): readonly ClinicalAiToolDefinition[] {
  return CLINICAL_AI_TOOL_DEFINITIONS.filter((tool) => tool.modules.includes(moduleId))
}

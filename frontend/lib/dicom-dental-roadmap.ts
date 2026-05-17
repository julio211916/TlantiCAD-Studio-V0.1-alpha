export interface DicomRoadmapSprint {
  id: string;
  title: string;
  focus: string;
  phases: number;
  tasksPerPhase: number;
}

export interface DicomCapabilitySnapshot {
  id: string;
  label: string;
  status: 'available' | 'partial' | 'planned';
  evidence: string;
}

export interface DicomActionableTask {
  id: string;
  stream: string;
  lane: string;
  title: string;
  outcome: string;
  sourceTaskRange: string;
}

export interface DicomActionablePhase {
  sprintId: string;
  phaseId: string;
  title: string;
  sourceTaskCount: number;
  laneCount: number;
  sourceDocPath: string;
  kickoffTasks: DicomActionableTask[];
}

export const DICOM_DENTAL_SPRINTS: DicomRoadmapSprint[] = [
  { id: 'S01', title: 'Fundación DICOM dental y baseline clínico', focus: 'baseline del viewer dental, contratos de estudio/serie/instancia y datasets CBCT', phases: 10, tasksPerPhase: 120 },
  { id: 'S02', title: 'Paridad 2D dental del viewer', focus: 'slice viewer, window/level, overlays y metadatos clínicos', phases: 10, tasksPerPhase: 120 },
  { id: 'S03', title: 'Gestión de estudio, series y metadata avanzada', focus: 'DICOMDIR, diccionario, multi-view y comparación', phases: 10, tasksPerPhase: 120 },
  { id: 'S04', title: 'MPR, reconstrucción 3D y volumen', focus: 'MPR axial/coronal/sagital, VR y reconstrucción dental', phases: 10, tasksPerPhase: 120 },
  { id: 'S05', title: 'Segmentación dental 2D/3D', focus: 'thresholds, watershed, brush, mandíbula, maxila y dientes', phases: 10, tasksPerPhase: 120 },
  { id: 'S06', title: 'Upper jaw motion, oclusión y cinemática', focus: 'seguimiento temporal maxilar superior y relación oclusal', phases: 10, tasksPerPhase: 120 },
  { id: 'S07', title: 'RTSTRUCT, SR, mediciones y reporte clínico', focus: 'contours, measurements, structured reports y trazabilidad', phases: 10, tasksPerPhase: 120 },
  { id: 'S08', title: 'Meshes, manufactura e interoperabilidad', focus: 'DicomToMesh, STL/OBJ/PLY, CAM y guías', phases: 10, tasksPerPhase: 120 },
  { id: 'S09', title: 'Red, DICOMweb, PACS y performance', focus: 'UL, DICOMweb, cache, transfer syntaxes y optimización', phases: 10, tasksPerPhase: 120 },
  { id: 'S10', title: 'Hardening regulatorio, QA y release', focus: 'de-identify, seguridad, QA visual y release multiplataforma', phases: 10, tasksPerPhase: 120 },
];

export const DICOM_CURRENT_CAPABILITIES: DicomCapabilitySnapshot[] = [
  { id: 'viewer-2d', label: 'Viewer 2D dental básico', status: 'available', evidence: 'components/ohif/TlantiOhifViewer.tsx + components/DicomControls.tsx' },
  { id: 'dicom-rust-parse', label: 'Parsing y pixel decode en Rust', status: 'available', evidence: 'src-tauri/crates/dental-imaging/src/dicom_viewer.rs' },
  { id: 'pydicom-sidecar', label: 'Operaciones avanzadas con PyDICOM sidecar', status: 'available', evidence: 'src-tauri/crates/python-bridge/src/pydicom.rs' },
  { id: 'metadata-lite', label: 'Lectura simplificada de DICOM en I/O core', status: 'partial', evidence: 'src-tauri/crates/tlanticad-io/src/dicom.rs' },
  { id: 'mpr-3d', label: 'MPR / 3D / volume rendering dental', status: 'planned', evidence: 'Roadmap S04' },
  { id: 'rtstruct-sr', label: 'RTSTRUCT / Structured Reports / KOS', status: 'planned', evidence: 'Roadmap S07' },
  { id: 'upper-jaw-motion', label: 'Upper jaw motion y cinemática oclusal', status: 'planned', evidence: 'Roadmap S06' },
  { id: 'segmentation', label: 'Segmentación dental clínica y automática', status: 'planned', evidence: 'Roadmap S05' },
];

export const DICOM_ROADMAP_DOC_ROOT = 'docs/roadmaps/dicom-dental';

export const DICOM_S01_P01_ACTIONABLE_PHASE: DicomActionablePhase = {
  sprintId: 'S01',
  phaseId: 'P01',
  title: 'Definición clínica y alcance',
  sourceTaskCount: 120,
  laneCount: 12,
  sourceDocPath: 'docs/roadmaps/dicom-dental/sprint-01.md',
  kickoffTasks: [
    {
      id: 'S01-P01-CLIN-K01',
      stream: 'CLIN',
      lane: 'Clinical workflow',
      title: 'Cerrar criterios clínicos de aceptación dental',
      outcome: 'Checklist clínico unificado para ortodoncia, implantología, cirugía y validación de upper jaw motion.',
      sourceTaskRange: 'S01-P01-CLIN-T01..T10',
    },
    {
      id: 'S01-P01-ING-K01',
      stream: 'ING',
      lane: 'Ingest & datasets',
      title: 'Congelar paquete base de datasets CBCT/panorámicos',
      outcome: 'Fixtures anonimizados con estructura estudio/serie/instancia, DICOMDIR y estudios problemáticos listos para test.',
      sourceTaskRange: 'S01-P01-ING-T01..T10',
    },
    {
      id: 'S01-P01-META-K01',
      stream: 'META',
      lane: 'Metadata & dictionary',
      title: 'Definir contrato de metadata enriquecida',
      outcome: 'Tags prioritarios, fallback rules y serialización JSON acordados entre Rust, sidecar y frontend.',
      sourceTaskRange: 'S01-P01-META-T01..T10',
    },
    {
      id: 'S01-P01-PIX-K01',
      stream: 'PIX',
      lane: 'Pixel & decoding',
      title: 'Validar baseline de decoding multiformato',
      outcome: 'Cobertura para MONOCHROME1/2, RGB y transfer syntaxes encapsulados con métricas de memoria/tiempo.',
      sourceTaskRange: 'S01-P01-PIX-T01..T10',
    },
    {
      id: 'S01-P01-UI2D-K01',
      stream: 'UI2D',
      lane: 'Viewer 2D UX',
      title: 'Cerrar baseline UX del viewer 2D dental',
      outcome: 'Slice nav, zoom/pan/rotation, WL, overlays y mediciones listos para evaluación clínica.',
      sourceTaskRange: 'S01-P01-UI2D-T01..T10',
    },
    {
      id: 'S01-P01-VOL3D-K01',
      stream: 'VOL3D',
      lane: 'MPR & 3D',
      title: 'Acordar entregable mínimo de MPR/3D',
      outcome: 'MPR axial/coronal/sagital sincronizado y criterio de reconstrucción/crop definidos para datasets dentales.',
      sourceTaskRange: 'S01-P01-VOL3D-T01..T10',
    },
    {
      id: 'S01-P01-SEGM-K01',
      stream: 'SEGM',
      lane: 'Segmentation & AI',
      title: 'Definir baseline de segmentación dental',
      outcome: 'Máscaras objetivo, presets threshold y ruta de revisión manual documentadas para mandíbula/maxila/dientes.',
      sourceTaskRange: 'S01-P01-SEGM-T01..T10',
    },
    {
      id: 'S01-P01-MOTN-K01',
      stream: 'MOTN',
      lane: 'Upper jaw motion',
      title: 'Modelar el primer contrato de movimiento maxilar',
      outcome: 'Landmarks, soporte temporal y relación con oclusión definidos para la primera iteración funcional.',
      sourceTaskRange: 'S01-P01-MOTN-T01..T10',
    },
    {
      id: 'S01-P01-XPRT-K01',
      stream: 'XPRT',
      lane: 'Interop & export',
      title: 'Cerrar outputs clínicos/manufactura del baseline',
      outcome: 'Contrato de export a STL/OBJ/PLY y nomenclatura de outputs alineados con CAD/CAM.',
      sourceTaskRange: 'S01-P01-XPRT-T01..T10',
    },
    {
      id: 'S01-P01-QASE-K01',
      stream: 'QASE',
      lane: 'QA, security & perf',
      title: 'Activar gate de QA/performance de la fase',
      outcome: 'Suite con datasets corruptos, de-identify, límites RAM/CPU/GPU y riesgos regulatorios mínimos registrados.',
      sourceTaskRange: 'S01-P01-QASE-T01..T10',
    },
    {
      id: 'S01-P01-DOCS-K01',
      stream: 'DOCS',
      lane: 'Docs & enablement',
      title: 'Preparar handoff clínico/QA de Sprint 01 Fase 01',
      outcome: 'Demo script, capturas esperadas, troubleshooting y acta de release gate listos para seguimiento.',
      sourceTaskRange: 'S01-P01-DOCS-T01..T10',
    },
    {
      id: 'S01-P01-AUTO-K01',
      stream: 'AUTO',
      lane: 'Automation & CI',
      title: 'Definir automatización mínima reproducible',
      outcome: 'Smoke de viewer, fixtures reproducibles y Puppeteer visual target conectados al primer gate CI.',
      sourceTaskRange: 'S01-P01-AUTO-T01..T10',
    },
  ],
};
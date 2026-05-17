import type { TlantiCadProductModuleId } from './cad-product-module-registry'

export type TlantiCadCompetitorSignal =
  | 'exocad'
  | 'nemostudio'
  | 'maestro3d'
  | 'audaxceph'
  | 'smilecloud'

export type TlantiCadWorkflowOwner =
  | 'react-ui'
  | 'three-render'
  | 'tauri-command'
  | 'rust-core'
  | 'python-sidecar'
  | 'wasm-preview'

export interface TlantiCadModuleWorkflowPhase {
  id: string
  label: string
  userGoal: string
  logicOwner: TlantiCadWorkflowOwner
  tools: readonly string[]
  jobs: readonly string[]
  outputs: readonly string[]
  guardrails: readonly string[]
}

export interface TlantiCadModuleRoadmapDefinition {
  moduleId: TlantiCadProductModuleId
  competitorSignals: readonly TlantiCadCompetitorSignal[]
  differentiators: readonly string[]
  belongsInModule: readonly string[]
  doesNotBelongInModule: readonly string[]
  workflow: readonly TlantiCadModuleWorkflowPhase[]
  nextImplementationTasks: readonly string[]
}

export const TLANTI_CAD_WORKFLOW_OWNERS = [
  'react-ui',
  'three-render',
  'tauri-command',
  'rust-core',
  'python-sidecar',
  'wasm-preview',
] as const satisfies readonly TlantiCadWorkflowOwner[]

export const CAD_MODULE_ROADMAP_DEFINITIONS = {
  'tlanticad-crown': {
    moduleId: 'tlanticad-crown',
    competitorSignals: ['exocad', 'smilecloud'],
    differentiators: [
      'Guided crown workflow with fewer visible tools per step than classic lab CAD',
      'Offline provenance for every margin, axis, contact and thickness decision',
      'Photo/smile context can inform anterior anatomy without turning Crown into a smile app',
    ],
    belongsInModule: [
      'Prep scan import and cleanup',
      'Margin marking and margin validation',
      'Insertion axis and blockout preview',
      'Anatomy proposal, morph, contacts and occlusion',
      'Thickness, cement gap and manufacturing export',
    ],
    doesNotBelongInModule: [
      'CBCT segmentation beyond referenced implant or root context',
      'Full ortho tooth movement staging',
      'Case management dashboards',
      'Large mesh repair UI that belongs in Freeform or Model',
    ],
    workflow: [
      {
        id: 'crown-import',
        label: 'Import and prep detection',
        userGoal: 'Load scan assets and identify the prepared tooth with minimal clicks.',
        logicOwner: 'tauri-command',
        tools: ['import-scan', 'tooth-detect', 'prep-select'],
        jobs: ['mesh-repair', 'margin-detection'],
        outputs: ['prep-asset-ref', 'tooth-selection'],
        guardrails: ['React only sends asset ids', 'All file reads stay inside app-data'],
      },
      {
        id: 'crown-margin-axis',
        label: 'Margin and insertion axis',
        userGoal: 'Produce a clinically editable margin and insertion path before anatomy is proposed.',
        logicOwner: 'rust-core',
        tools: ['margin', 'axis', 'blockout-preview'],
        jobs: ['margin-detection', 'mesh-offset'],
        outputs: ['margin-curve', 'insertion-axis'],
        guardrails: ['Manual correction must be command-event based', 'No geometry snapshots in React state'],
      },
      {
        id: 'crown-validate-export',
        label: 'Contacts, thickness and export',
        userGoal: 'Validate proximal/occlusal contacts, material thickness and manufacturing readiness.',
        logicOwner: 'python-sidecar',
        tools: ['contacts', 'thickness', 'manufacturing-export'],
        jobs: ['contact-analysis', 'thickness-map'],
        outputs: ['crown-stl', 'case-xml', 'manufacturing-report'],
        guardrails: ['Three.js displays maps only', 'Clinical reports keep tool_version and params'],
      },
    ],
    nextImplementationTasks: [
      'Promote margin and axis operations to persistible clinical commands',
      'Add crown-specific command palette ranking',
      'Add thickness map artifact to asset manifest',
    ],
  },
  'tlanticad-implant': {
    moduleId: 'tlanticad-implant',
    competitorSignals: ['exocad', 'nemostudio'],
    differentiators: [
      'CBCT/STL registration is a first-class offline job, not a hidden wizard side effect',
      'Nerve, sinus and sleeve validations are stored as reviewable clinical events',
      'DICOM fast metadata can move to Rust while Python remains advanced fallback',
    ],
    belongsInModule: [
      'CBCT import, DICOM sanitization and preview',
      'Surface scan registration',
      'Nerve, sinus and anatomical safety zones',
      'Implant library, prosthetic axis, sleeve and guide planning',
      'Post and Core work type, scan body branching and restorative wizard handoff',
      'Surgical report and guide export',
    ],
    doesNotBelongInModule: [
      'General crown morphology design beyond prosthetic reference',
      'Cloud-only planning or remote model inference',
      'Manual filesystem scanning from React',
      'Full orthognathic osteotomy planning',
    ],
    workflow: [
      {
        id: 'implant-ingest',
        label: 'DICOM/STL ingest',
        userGoal: 'Open CBCT and scan context safely with patient data protection.',
        logicOwner: 'python-sidecar',
        tools: ['dicom-sanitize', 'dicom-preview', 'scan-import'],
        jobs: ['dicom-sanitize', 'dicom-segmentation'],
        outputs: ['sanitized-dicom', 'dicom-preview', 'surface-scan-ref'],
        guardrails: ['No DICOM parsing in React', 'Derived previews carry checksum'],
      },
      {
        id: 'implant-registration',
        label: 'Registration and anatomy',
        userGoal: 'Align CBCT, IOS and anatomical structures for a prosthetically driven plan.',
        logicOwner: 'rust-core',
        tools: ['surface-registration', 'nerve-mark', 'sinus-mark', 'measure'],
        jobs: ['surface-registration', 'guide-preview'],
        outputs: ['registration-transform', 'anatomy-markers'],
        guardrails: ['Registration transforms are immutable artifacts', 'Canvas renders overlays only'],
      },
      {
        id: 'implant-guide',
        label: 'Implant, post/core, sleeve and guide',
        userGoal: 'Select implant system or post/core branch, plan axis, validate scan-body branch and export guide/report.',
        logicOwner: 'tauri-command',
        tools: ['implant-library', 'material-config', 'scan-body', 'implant-axis', 'sleeve-controls', 'guide-export'],
        jobs: ['guide-preview', 'mesh-offset', 'post-core-preview'],
        outputs: ['implant-plan', 'post-core-plan', 'guide-stl', 'planning-report'],
        guardrails: ['Implant library is app-data scoped', 'Guide generation is async and cancellable'],
      },
    ],
    nextImplementationTasks: [
      'Move DICOM sanitize command into Tauri asset storage',
      'Add registration smoke test with fixture transforms',
      'Add guide/sleeve validation artifact schema',
      'Add Post and Core wizard branch with scan body / no-scan-body validation',
    ],
  },
  'tlanticad-bridge': {
    moduleId: 'tlanticad-bridge',
    competitorSignals: ['exocad', 'maestro3d'],
    differentiators: [
      'Span-first bridge workflow separates pontic logic from single crown morphology',
      'Connector validation becomes a measurable job with thresholds per material',
      'Virtual pontic UX is simplified into a guided span editor',
    ],
    belongsInModule: [
      'Multiple prep import and span definition',
      'Pontic creation or clone-based pontic proposal',
      'Connector sizing and insertion path',
      'Bridge contacts, occlusion, thickness and export',
    ],
    doesNotBelongInModule: [
      'Implant surgical guide design',
      'Full partial framework design',
      'Cephalometric tracing',
      'Freeform mesh cleanup unrelated to bridge assets',
    ],
    workflow: [
      {
        id: 'bridge-span',
        label: 'Span definition',
        userGoal: 'Select abutments and missing units with clear material/manufacturing constraints.',
        logicOwner: 'react-ui',
        tools: ['odontogram', 'span-select', 'material-route'],
        jobs: ['connector-validation'],
        outputs: ['bridge-span', 'material-profile'],
        guardrails: ['UI stores only DTO choices', 'Material rules live in use-cases'],
      },
      {
        id: 'bridge-pontic-connectors',
        label: 'Pontics and connectors',
        userGoal: 'Generate pontic anatomy and connector dimensions that can be edited and validated.',
        logicOwner: 'rust-core',
        tools: ['pontic-design', 'connector-size', 'axis'],
        jobs: ['mesh-boolean', 'connector-validation'],
        outputs: ['pontic-mesh', 'connector-map'],
        guardrails: ['Boolean work is off main thread', 'Connector limits are not hardcoded in components'],
      },
      {
        id: 'bridge-export',
        label: 'Validation and export',
        userGoal: 'Confirm fit, contact, thickness and export a manufacturing-ready bridge.',
        logicOwner: 'python-sidecar',
        tools: ['contacts', 'thickness', 'manufacturing-export'],
        jobs: ['contact-analysis', 'thickness-map'],
        outputs: ['bridge-stl', 'manufacturing-report'],
        guardrails: ['Contact map is a derived artifact', 'Export keeps parent asset lineage'],
      },
    ],
    nextImplementationTasks: [
      'Add bridge span domain DTO and validation test',
      'Add connector threshold registry by material',
      'Add pontic preview command stub behind clinical jobs',
    ],
  },
  'tlanticad-waxup': {
    moduleId: 'tlanticad-waxup',
    competitorSignals: ['exocad', 'smilecloud', 'nemostudio'],
    differentiators: [
      'Smile design becomes a bridge into real CAD assets, not only visual presentation',
      'Offline-first photo-to-3D alignment with explicit confidence and manual correction',
      'Patient-facing previews are generated as artifacts with clinical provenance',
    ],
    belongsInModule: [
      'Photo, face, scan and tooth-library alignment',
      'Anterior/posterior anatomy mockup',
      'Smile and occlusion review',
      'Before/after preview and waxup export',
    ],
    doesNotBelongInModule: [
      'Final crown margin/cement gap decisions',
      'DICOM segmentation unless used as visual reference',
      'Cloud collaboration as a core dependency',
      'Implant sleeve or surgical guide generation',
    ],
    workflow: [
      {
        id: 'waxup-context',
        label: 'Photo and scan context',
        userGoal: 'Attach facial/photo context to the dental scan without blocking the CAD canvas.',
        logicOwner: 'python-sidecar',
        tools: ['smile-photos', 'face-align', 'scan-align'],
        jobs: ['smile-preview'],
        outputs: ['photo-alignment', 'face-reference'],
        guardrails: ['Photos stay local', 'AI preview is optional and checksummed'],
      },
      {
        id: 'waxup-anatomy',
        label: 'Tooth library and anatomy',
        userGoal: 'Place and edit tooth proposals using visual smile context and real 3D constraints.',
        logicOwner: 'wasm-preview',
        tools: ['tooth-library', 'anatomy-morph', 'smile-guides'],
        jobs: ['tooth-library-placement'],
        outputs: ['waxup-proposal', 'smile-guide-lines'],
        guardrails: ['WASM is preview-only', 'Final mesh generation uses Rust/Python jobs'],
      },
      {
        id: 'waxup-export',
        label: 'Mockup and export',
        userGoal: 'Generate mockup STL and presentation assets that can feed Crown or Model workflows.',
        logicOwner: 'tauri-command',
        tools: ['mockup-export', 'case-share-preview'],
        jobs: ['asset-derive', 'contact-analysis'],
        outputs: ['waxup-stl', 'before-after-preview', 'case-xml'],
        guardrails: ['No remote sharing dependency', 'Presentation output is not the source of truth'],
      },
    ],
    nextImplementationTasks: [
      'Add waxup photo alignment DTOs',
      'Add local before/after preview artifact type',
      'Rank Smilecloud-inspired actions only inside Waxup/Smile context',
    ],
  },
  'tlanticad-freeform': {
    moduleId: 'tlanticad-freeform',
    competitorSignals: ['maestro3d'],
    differentiators: [
      'Freeform is the explicit mesh surgery room, so other modules stay clean',
      'Sculpt, smooth, cut, repair and boolean tools are job-backed when destructive',
      'Preview tools can use WASM, but authoritative outputs come from Tauri/Rust/Python',
    ],
    belongsInModule: [
      'Generic mesh import and inspection',
      'Masking, sculpting, smoothing, clipping and repair',
      'Boolean/offset experiments before converting to module-specific outputs',
      'Derived asset creation and validation',
    ],
    doesNotBelongInModule: [
      'Clinical indication selection',
      'DICOM diagnosis',
      'Implant planning rules',
      'Final prosthetic reports without a destination module',
    ],
    workflow: [
      {
        id: 'freeform-import',
        label: 'Mesh import and inspect',
        userGoal: 'Bring any mesh into a stable canvas and inspect topology, units and scale.',
        logicOwner: 'tauri-command',
        tools: ['import-mesh', 'measure', 'layers-panel'],
        jobs: ['asset-derive'],
        outputs: ['mesh-manifest', 'scale-report'],
        guardrails: ['Do not parse large files in React', 'Keep mesh metadata separate from binary assets'],
      },
      {
        id: 'freeform-edit',
        label: 'Sculpt, cut and repair',
        userGoal: 'Edit mesh shape while keeping destructive operations undoable and auditable.',
        logicOwner: 'rust-core',
        tools: ['sculpt', 'smooth', 'cut', 'repair', 'boolean'],
        jobs: ['mesh-repair', 'mesh-boolean', 'mesh-smooth'],
        outputs: ['derived-mesh', 'repair-report'],
        guardrails: ['Persist commands, not mesh snapshots', 'Use LOD for active brush feedback'],
      },
      {
        id: 'freeform-handoff',
        label: 'Validate and hand off',
        userGoal: 'Validate mesh quality and send a clean derived asset to a dental module.',
        logicOwner: 'python-sidecar',
        tools: ['validate-mesh', 'handoff-module'],
        jobs: ['mesh-repair', 'asset-derive'],
        outputs: ['validated-mesh', 'handoff-event'],
        guardrails: ['Handoff requires target module', 'Validation output is a clinical artifact'],
      },
    ],
    nextImplementationTasks: [
      'Wire MeshLib repair/boolean/offset as real Tauri jobs',
      'Add brush preview LOD budget',
      'Add derived mesh manifest UI',
    ],
  },
  'tlanticad-abutment': {
    moduleId: 'tlanticad-abutment',
    competitorSignals: ['exocad', 'nemostudio', 'smilecloud'],
    differentiators: [
      'Abutment is separated from implant surgery so restorative users see fewer controls',
      'Emergence profile, screw channel and cement gap are explicit validated phases',
      'Smile/soft-tissue context can inform anterior emergence without owning smile design',
    ],
    belongsInModule: [
      'Implant platform selection',
      'Cross-section profile and emergence margin loop design',
      'Post bottom, core minimum angle and spacing controls',
      'Surface adaptation against gingiva/prep scan',
      'Screw channel and angulation validation',
      'Margin, cement gap and abutment export',
    ],
    doesNotBelongInModule: [
      'CBCT nerve tracing',
      'Surgical guide sleeve design',
      'Full crown anatomy finalization',
      'Orthodontic staging',
    ],
    workflow: [
      {
        id: 'abutment-platform',
        label: 'Platform and tissue context',
        userGoal: 'Select implant platform and gum context before designing emergence.',
        logicOwner: 'tauri-command',
        tools: ['implant-library', 'gingiva-context', 'axis'],
        jobs: ['implant-platform-resolve'],
        outputs: ['platform-ref', 'gingiva-profile'],
        guardrails: ['Implant libraries are scoped resources', 'No library binaries in frontend bundle'],
      },
      {
        id: 'abutment-profile-emergence',
        label: 'Profile, margin loop and emergence body',
        userGoal: 'Shape cross-section, margin loop, collar body and surface adaptation before destructive booleans.',
        logicOwner: 'rust-core',
        tools: ['abutment-cross-section', 'abutment-margin-loop', 'abutment-collar', 'abutment-shrinkwrap', 'measure'],
        jobs: ['abutment-cross-section', 'abutment-margin-loop', 'abutment-collar-body', 'abutment-shrinkwrap'],
        outputs: ['profile-curve', 'margin-loop', 'adapted-abutment-mesh', 'distance-map'],
        guardrails: ['Object-name conventions are converted to typed asset refs', 'Surface projection uses mesh hashes and cached acceleration structures'],
      },
      {
        id: 'abutment-channel-export',
        label: 'Screw channel, cleanup and export',
        userGoal: 'Cut the screw channel, validate mesh quality and output STL/construction/report artifacts.',
        logicOwner: 'rust-core',
        tools: ['abutment-screw-channel', 'abutment-cleanup', 'abutment-report', 'thickness'],
        jobs: ['abutment-boolean-cut', 'abutment-mesh-cleanup', 'abutment-export-package'],
        outputs: ['abutment-stl', 'construction-info-json', 'planning-pdf'],
        guardrails: ['Booleans are cancellable Rust jobs', 'Reports include implant system metadata and source hashes'],
      },
    ],
    nextImplementationTasks: [
      'Bind AbutmentDesignPanel actions to ABUTMENT_WORKFLOW_DEFINITION command ids',
      'Add implant platform manifest schema',
      'Promote screw channel validation to Rust mesh job with progress/cancel',
      'Add client-specific defaults for post/core angle and spacing',
    ],
  },
  'tlanticad-model': {
    moduleId: 'tlanticad-model',
    competitorSignals: ['exocad', 'maestro3d', 'audaxceph'],
    differentiators: [
      'Model creation is print-first with explicit hollow, drainage and label steps',
      'Study-model measurements can share assets with Ceph without merging workflows',
      'Antagonist prompts become workflow state, not random dialogs',
    ],
    belongsInModule: [
      'Scan import and alignment',
      'Base creation, hollow, drain and label',
      'Antagonist management',
      'Printable STL/PLY/OBJ export and report',
    ],
    doesNotBelongInModule: [
      'Final crown or bridge morphology',
      'Cephalometric landmark protocol editing',
      'Implant sleeve planning',
      'Cloud-only review workflows',
    ],
    workflow: [
      {
        id: 'model-align',
        label: 'Import and align',
        userGoal: 'Orient scans predictably and detect whether antagonist or bite assets are missing.',
        logicOwner: 'tauri-command',
        tools: ['import-scan', 'align', 'antagonist-prompt'],
        jobs: ['mesh-repair'],
        outputs: ['aligned-scan', 'missing-asset-prompt'],
        guardrails: ['Prompts are drawer state, not blocking alerts', 'Alignment transform is persisted'],
      },
      {
        id: 'model-base',
        label: 'Base, hollow and label',
        userGoal: 'Produce a printable diagnostic or working model with controlled material usage.',
        logicOwner: 'rust-core',
        tools: ['base', 'hollow', 'drainage-holes', 'label'],
        jobs: ['model-base', 'model-hollow'],
        outputs: ['model-base-mesh', 'label-metadata'],
        guardrails: ['Hollowing is async', 'Labels are vector/mesh artifacts with provenance'],
      },
      {
        id: 'model-export',
        label: 'Repair and printable export',
        userGoal: 'Validate watertightness, reduce file size and export to manufacturing.',
        logicOwner: 'python-sidecar',
        tools: ['repair', 'decimate', 'manufacturing-export'],
        jobs: ['mesh-repair', 'asset-derive'],
        outputs: ['model-stl', 'printable-model-report'],
        guardrails: ['Do not keep full mesh history in React', 'Output keeps checksum and parent asset id'],
      },
    ],
    nextImplementationTasks: [
      'Replace antagonist modal with workflow drawer',
      'Add model base/hollow job schema',
      'Add printable model QA screenshot state',
    ],
  },
  'tlanticad-bar': {
    moduleId: 'tlanticad-bar',
    competitorSignals: ['exocad', 'nemostudio'],
    differentiators: [
      'Bar/partial workflows stay separate from bridge to avoid prosthetic tool overload',
      'Attachment and relief checks become measurable async validations',
      'Edentulous workflows share implant positions without inheriting implant surgical UI',
    ],
    belongsInModule: [
      'Edentulous or partial scan import',
      'Bar path design and attachment placement',
      'Relief, undercut and manufacturing validation',
      'Framework/bar export',
    ],
    doesNotBelongInModule: [
      'Single crown margin design',
      'CBCT anatomy segmentation unless imported as implant-position reference',
      'Smile video presentation',
      'Ortho staging and aligners',
    ],
    workflow: [
      {
        id: 'bar-context',
        label: 'Edentulous context',
        userGoal: 'Load scan, implant positions and tissue context for a bar or partial framework.',
        logicOwner: 'tauri-command',
        tools: ['import-scan', 'implant-position-ref', 'undercut-map'],
        jobs: ['mesh-repair'],
        outputs: ['edentulous-context', 'implant-position-map'],
        guardrails: ['Implant positions are references, not surgical edits', 'Undercut map is derived'],
      },
      {
        id: 'bar-design',
        label: 'Bar path and attachments',
        userGoal: 'Draw bar path, place attachments and preview relief.',
        logicOwner: 'rust-core',
        tools: ['bar-design', 'attachment-library', 'relief'],
        jobs: ['bar-path-preview', 'attachment-validation'],
        outputs: ['bar-path', 'attachment-map'],
        guardrails: ['Attachment libraries are local resources', 'Validation thresholds are material-aware'],
      },
      {
        id: 'bar-export',
        label: 'Validate and export',
        userGoal: 'Validate relief and produce a manufacturing-ready bar or partial framework.',
        logicOwner: 'python-sidecar',
        tools: ['mesh-offset', 'repair', 'manufacturing-export'],
        jobs: ['mesh-offset', 'mesh-repair'],
        outputs: ['bar-stl', 'partial-framework-report'],
        guardrails: ['Export is cancellable', 'Report includes attachment ids and parameters'],
      },
    ],
    nextImplementationTasks: [
      'Add bar path command DTO',
      'Add attachment library manifest',
      'Create relief/undercut artifact viewer',
    ],
  },
  'tlanticad-telescope': {
    moduleId: 'tlanticad-telescope',
    competitorSignals: ['exocad'],
    differentiators: [
      'Telescope crowns get a dedicated friction/spacing workflow instead of hiding in Crown',
      'Primary and secondary crown assets are linked but independently validated',
      'Fit reports are first-class artifacts for lab communication',
    ],
    belongsInModule: [
      'Primary crown generation',
      'Insertion axis and path validation',
      'Secondary crown design',
      'Friction, spacing, thickness and fit report',
    ],
    doesNotBelongInModule: [
      'Partial framework bar path design',
      'Implant surgical planning',
      'Smile design video',
      'General mesh sculpting not related to telescope fit',
    ],
    workflow: [
      {
        id: 'telescope-primary',
        label: 'Primary crown',
        userGoal: 'Create the primary crown geometry and establish insertion path.',
        logicOwner: 'rust-core',
        tools: ['axis', 'primary-crown', 'blockout-preview'],
        jobs: ['thickness-map'],
        outputs: ['primary-crown-stl', 'insertion-axis'],
        guardrails: ['Primary crown is a separate asset', 'Axis edits are clinical command events'],
      },
      {
        id: 'telescope-secondary',
        label: 'Secondary crown and friction',
        userGoal: 'Design secondary crown spacing and friction behavior with validation.',
        logicOwner: 'python-sidecar',
        tools: ['telescope-fit', 'spacing-map', 'measure'],
        jobs: ['spacing-validation', 'friction-fit-preview'],
        outputs: ['secondary-crown-stl', 'fit-map'],
        guardrails: ['Spacing map is derived', 'Friction preview must not block UI'],
      },
      {
        id: 'telescope-export',
        label: 'Fit report and export',
        userGoal: 'Export both crowns and a fit report with reproducible parameters.',
        logicOwner: 'tauri-command',
        tools: ['manufacturing-export', 'fit-report'],
        jobs: ['asset-derive'],
        outputs: ['fit-report', 'case-xml'],
        guardrails: ['Report links primary and secondary assets', 'No hidden defaults in export'],
      },
    ],
    nextImplementationTasks: [
      'Add telescope-fit shell action',
      'Add primary/secondary asset relation schema',
      'Add spacing validation test fixtures',
    ],
  },
  'tlanticad-bite-splint': {
    moduleId: 'tlanticad-bite-splint',
    competitorSignals: ['exocad', 'maestro3d', 'nemostudio'],
    differentiators: [
      'JawMotionAI is native to the splint workflow instead of an external export step',
      'Contacts and excursions are calculated as backend jobs and rendered as overlays',
      'Michigan/nightguard workflows are simplified into phase-based controls',
    ],
    belongsInModule: [
      'Maxilla, mandible and bite registration',
      'Occlusal plane and offset definition',
      'Jaw motion, contact and excursion analysis',
      'Trim, smooth, validate and export',
    ],
    doesNotBelongInModule: [
      'General smile design',
      'Implant guide sleeve planning',
      'Ceph landmark tracing',
      'Large DICOM segmentation beyond jaw motion context',
    ],
    workflow: [
      {
        id: 'splint-registration',
        label: 'Scan and bite registration',
        userGoal: 'Load maxilla, mandible and bite relation, then verify registration before design.',
        logicOwner: 'tauri-command',
        tools: ['import-scan', 'bite-align', 'occlusal-plane'],
        jobs: ['surface-registration'],
        outputs: ['bite-transform', 'occlusal-plane'],
        guardrails: ['Bite transforms are persisted', 'Missing assets become workflow prompts'],
      },
      {
        id: 'splint-motion',
        label: 'Offset, motion and contacts',
        userGoal: 'Generate splint shell and evaluate protrusive/lateral contact behavior.',
        logicOwner: 'python-sidecar',
        tools: ['offset', 'jaw-motion', 'contacts'],
        jobs: ['jaw-motion', 'occlusion-map', 'contact-analysis'],
        outputs: ['motion-tracks', 'occlusion-map', 'splint-preview'],
        guardrails: ['JawMotion runs in Python sidecar', 'Three.js only displays tracks and heatmaps'],
      },
      {
        id: 'splint-export',
        label: 'Trim and export',
        userGoal: 'Trim, smooth, validate thickness and export a printable splint.',
        logicOwner: 'rust-core',
        tools: ['trim', 'smooth', 'thickness', 'splint-export'],
        jobs: ['mesh-offset', 'mesh-repair'],
        outputs: ['splint-stl', 'occlusion-report'],
        guardrails: ['Trim is undoable as command events', 'Output includes parent scan checksums'],
      },
    ],
    nextImplementationTasks: [
      'Wire JawMotion result preview to splint overlays',
      'Add occlusion-map artifact UI',
      'Add splint offset/trim command persistence',
    ],
  },
} as const satisfies Record<TlantiCadProductModuleId, TlantiCadModuleRoadmapDefinition>

export function listCadModuleRoadmaps(): readonly TlantiCadModuleRoadmapDefinition[] {
  return Object.values(CAD_MODULE_ROADMAP_DEFINITIONS)
}

export function resolveCadModuleRoadmap(moduleId: TlantiCadProductModuleId): TlantiCadModuleRoadmapDefinition {
  return CAD_MODULE_ROADMAP_DEFINITIONS[moduleId]
}

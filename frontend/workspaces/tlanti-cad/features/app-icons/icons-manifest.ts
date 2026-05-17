/**
 * icons-manifest — single source of truth for every semantic icon name the
 * TlantiCAD UI renders.
 *
 * Resolution order at render time (see <AppIcon />):
 *   1. `svg`     → a TlantiCAD SVG under `/public/icons/tlanticad/**` rendered
 *                  through a CSS mask so it inherits `currentColor`.
 *   2. `exocad`  → a reference raster/SVG file under `/public/icons/**` served
 *                  as <img> for legacy compatibility.
 *   3. `lucide`  → a Lucide React icon component (tree-shakable).
 *   4. `missing` → rendered as a dashed placeholder. Surface this in the dev
 *                  overlay so the user knows what to commission.
 *
 * Add new entries by module (see MODULE_ICONS). The manifest is the only
 * place that references public paths — UI components pass semantic names.
 */

import type { LucideIcon } from 'lucide-react';
import {
    Activity,
    AlertTriangle,
    ArrowLeft,
    ArrowRight,
    Box,
    Brain,
    Camera,
    ChevronDown,
    Circle,
    CircleDot,
    Crop,
    Download,
    Droplet,
    Eye,
    EyeOff,
    FileText,
    Folder,
    Grid3x3,
    Hand,
    Home,
    Image,
    Info,
    Layers,
    LayoutGrid,
    LogOut,
    Maximize,
    Minus,
    Move,
    MoveHorizontal,
    Pencil,
    PieChart,
    Play,
    Plus,
    Printer,
    Redo2,
    RefreshCw,
    RotateCcw,
    Ruler,
    Save,
    Scissors,
    Settings,
    Share2,
    Sliders,
    Smile,
    SplitSquareHorizontal,
    Square,
    Stethoscope,
    Sun,
    Target,
    Trash2,
    TriangleAlert,
    Undo2,
    Upload,
    Wand2,
    Wrench,
    X,
    ZoomIn,
    ZoomOut,
} from 'lucide-react';

export type IconSource =
    | { kind: 'svg'; path: string }
    | { kind: 'exocad'; path: string }
    | { kind: 'lucide'; icon: LucideIcon }
    | { kind: 'missing'; hint: string };

/**
 * Modules where each icon belongs. Used by the gap report in docs so that
 * missing icons can be commissioned module-by-module.
 */
export type IconModule =
    | 'shell'
    | 'viewer'
    | 'dentaldb'
    | 'crown-seg'
    | 'implant'
    | 'surgical-guide'
    | 'splint'
    | 'smile'
    | 'ortho'
    | 'post-core'
    | 'ai'
    | 'workflow'
    | 'freeforming'
    | 'telescope'
    | 'screw'
    | 'bite-splint'
    | 'palette'
    | 'abutment'
    | 'module'
    | 'tools'
    | 'actions'
    | 'common';

export interface IconEntry {
    module: IconModule;
    source: IconSource;
    description: string;
}

/**
 * Canonical manifest. Key = semantic name (kebab-case, namespaced by module).
 */
export const APP_ICONS: Record<string, IconEntry> = {
    // ── TlantiCAD module icons (own SVG system, currentColor) ─────────
    'module.implant-planning': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/implant-planning.svg' },
        description: 'Implant planning module',
    },
    'module.dicom-viewer': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/dicom-viewer.svg' },
        description: 'DICOM viewer module',
    },
    'module.smile-design': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/smile-design.svg' },
        description: 'Smile design module',
    },
    'module.crown-bridge': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/crown-bridge.svg' },
        description: 'Crown and bridge module',
    },
    'module.surgical-guide': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/surgical-guide.svg' },
        description: 'Surgical guide module',
    },
    'module.splint': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/splint.svg' },
        description: 'Splint design module',
    },
    'module.articulator': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/articulator.svg' },
        description: 'Articulator module',
    },
    'module.ortho-aligners': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/ortho-aligners.svg' },
        description: 'Ortho aligners module',
    },
    'module.model-creator': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/model-creator.svg' },
        description: 'Model creator module',
    },
    'module.cam-fabrication': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/cam-fabrication.svg' },
        description: 'CAM fabrication module',
    },
    'module.asset-vault': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/asset-vault.svg' },
        description: 'Asset vault module',
    },
    'module.jobs-monitor': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/jobs-monitor.svg' },
        description: 'Jobs monitor module',
    },
    'module.reports-export': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/reports-export.svg' },
        description: 'Reports and export module',
    },
    'module.settings-libraries': {
        module: 'module',
        source: { kind: 'svg', path: '/icons/tlanticad/modules/settings-libraries.svg' },
        description: 'Settings and libraries module',
    },

    // ── TlantiCAD dental tools (own SVG system, currentColor) ─────────
    'tool.abutment-design': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/abutment-design.svg' },
        description: 'Abutment design tool',
    },
    'tool.crown-bottom': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/crown-bottom.svg' },
        description: 'Crown bottom tool',
    },
    'tool.choose-library-tooth': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/choose-library-tooth.svg' },
        description: 'Choose library tooth tool',
    },
    'tool.freeforming': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/freeforming.svg' },
        description: 'Freeforming tool',
    },
    'tool.emergence-profile': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/emergence-profile.svg' },
        description: 'Emergence profile tool',
    },
    'tool.minimum-thickness': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/minimum-thickness.svg' },
        description: 'Minimum thickness tool',
    },
    'tool.model-alignment': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/model-alignment.svg' },
        description: 'Model alignment tool',
    },
    'tool.copy-mirror-extract': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/copy-mirror-extract.svg' },
        description: 'Copy, mirror or extract tooth tool',
    },
    'tool.tooth-axes': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/tooth-axes.svg' },
        description: 'Show or hide tooth axes tool',
    },
    'tool.select-implant-type': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/select-implant-type.svg' },
        description: 'Select implant type tool',
    },
    'tool.insertion-direction': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/insertion-direction.svg' },
        description: 'Insertion direction tool',
    },
    'tool.measurement': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/measurement.svg' },
        description: 'Measurement tool',
    },
    'tool.sectional-view': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/sectional-view.svg' },
        description: 'Sectional view tool',
    },
    'tool.screenshot': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/screenshot.svg' },
        description: 'Screenshot tool',
    },
    'tool.dicom-control': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/dicom-control.svg' },
        description: 'DICOM control tool',
    },
    'tool.export-pdf': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/export-pdf.svg' },
        description: 'Export PDF tool',
    },
    'tool.export-stl': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/export-stl.svg' },
        description: 'Export STL tool',
    },
    'tool.export-3mf': {
        module: 'tools',
        source: { kind: 'svg', path: '/icons/tlanticad/tools/export-3mf.svg' },
        description: 'Export 3MF tool',
    },

    // ── TlantiCAD actions (own SVG system, currentColor) ──────────────
    'action.import-asset': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/import-asset.svg' },
        description: 'Import asset action',
    },
    'action.fit-camera': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/fit-camera.svg' },
        description: 'Fit camera action',
    },
    'action.lock-object': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/lock-object.svg' },
        description: 'Lock object action',
    },
    'action.opacity': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/opacity.svg' },
        description: 'Opacity action',
    },
    'action.validate': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/validate.svg' },
        description: 'Validate action',
    },
    'action.cancel-job': {
        module: 'actions',
        source: { kind: 'svg', path: '/icons/tlanticad/actions/cancel-job.svg' },
        description: 'Cancel job action',
    },

    // ── shell / toolbar ────────────────────────────────────────────────
    'shell.save': {
        module: 'shell',
        source: { kind: 'exocad', path: '/icons/SaveIcon.svg' },
        description: 'Save current project',
    },
    'shell.reset': {
        module: 'shell',
        source: { kind: 'lucide', icon: RefreshCw },
        description: 'Reset viewport',
    },
    'shell.layout': {
        module: 'shell',
        source: { kind: 'lucide', icon: LayoutGrid },
        description: 'Layout / window arrangement',
    },
    'shell.screenshot': {
        module: 'shell',
        source: { kind: 'lucide', icon: Camera },
        description: 'Capture screenshot',
    },
    'shell.window-level': {
        module: 'shell',
        source: { kind: 'lucide', icon: Sun },
        description: 'Window / Level (W/L)',
    },
    'shell.ruler': {
        module: 'shell',
        source: { kind: 'lucide', icon: Ruler },
        description: 'Measurement ruler tool',
    },
    'shell.info': {
        module: 'shell',
        source: { kind: 'exocad', path: '/icons/Info.svg' },
        description: 'Info / about',
    },
    'shell.settings': {
        module: 'shell',
        source: { kind: 'exocad', path: '/icons/SettingsIcon.svg' },
        description: 'Application settings',
    },
    'shell.support': {
        module: 'shell',
        source: { kind: 'lucide', icon: Stethoscope },
        description: 'Support / help',
    },
    'shell.exit': {
        module: 'shell',
        source: { kind: 'lucide', icon: LogOut },
        description: 'Exit / logout',
    },
    'shell.undo': {
        module: 'shell',
        source: { kind: 'lucide', icon: Undo2 },
        description: 'Undo last action',
    },
    'shell.redo': {
        module: 'shell',
        source: { kind: 'lucide', icon: Redo2 },
        description: 'Redo action',
    },

    // ── module brand icons (provided SVG illustrations) ────────────────
    'viewer.brand': {
        module: 'viewer',
        source: { kind: 'exocad', path: '/icons/module-brand/dicom-viewer.svg' },
        description: 'DICOM Viewer module brand icon',
    },
    'implant.brand': {
        module: 'implant',
        source: { kind: 'exocad', path: '/icons/module-brand/implant.svg' },
        description: 'Implant module brand icon',
    },
    'smile.brand': {
        module: 'smile',
        source: { kind: 'exocad', path: '/icons/module-brand/smile.svg' },
        description: 'Smile & Ortho module brand icon',
    },
    'abutment.brand': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/module-brand/abutment.svg' },
        description: 'Abutment module brand icon',
    },

    // ── DICOM viewer ───────────────────────────────────────────────────
    'viewer.volume-render': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Box },
        description: '3D volume rendering',
    },
    'viewer.mpr': {
        module: 'viewer',
        source: { kind: 'lucide', icon: SplitSquareHorizontal },
        description: 'Multi-planar reformat',
    },
    'viewer.panoramic': {
        module: 'viewer',
        source: { kind: 'lucide', icon: MoveHorizontal },
        description: 'Panoramic reconstruction',
    },
    'viewer.clipping-plane': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Scissors },
        description: 'Clipping plane tool',
    },
    'viewer.zoom-in': {
        module: 'viewer',
        source: { kind: 'exocad', path: '/icons/ZoomInIcon.svg' },
        description: 'Zoom in',
    },
    'viewer.zoom-out': {
        module: 'viewer',
        source: { kind: 'exocad', path: '/icons/ZoomOutIcon.svg' },
        description: 'Zoom out',
    },
    'viewer.pan': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Hand },
        description: 'Pan',
    },
    'viewer.rotate': {
        module: 'viewer',
        source: { kind: 'lucide', icon: RotateCcw },
        description: 'Rotate',
    },
    'viewer.crosshair': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Target },
        description: 'Crosshair / landmark',
    },
    'viewer.layers': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Layers },
        description: 'Toggle segmentation overlay',
    },
    'viewer.maximize': {
        module: 'viewer',
        source: { kind: 'lucide', icon: Maximize },
        description: 'Maximize viewport',
    },

    // ── crown segmentation (RealGUIDE reference) ──────────────────────
    'crown-seg.toggle': {
        module: 'crown-seg',
        source: { kind: 'exocad', path: '/icons/worktype/AnatomicCrown.WorkType.svg' },
        description: 'Open crown segmentation panel',
    },
    'crown-seg.maxilla': {
        module: 'crown-seg',
        source: {
            kind: 'exocad',
            path: '/icons/FullDenture.AutoSelect.UpperJaw.Icon.svg',
        },
        description: 'Target maxilla (upper jaw)',
    },
    'crown-seg.mandible': {
        module: 'crown-seg',
        source: {
            kind: 'exocad',
            path: '/icons/FullDenture.AutoSelect.LowerJaw.Icon.svg',
        },
        description: 'Target mandible (lower jaw)',
    },
    'crown-seg.clear-all': {
        module: 'crown-seg',
        source: { kind: 'exocad', path: '/icons/ClearToothWorkIcon.svg' },
        description: 'Clear all selections',
    },
    'crown-seg.auto': {
        module: 'crown-seg',
        source: { kind: 'lucide', icon: Wand2 },
        description: 'Automatic segmentation (TGN AI)',
    },
    'crown-seg.keep-crowns': {
        module: 'crown-seg',
        source: { kind: 'lucide', icon: CircleDot },
        description: 'Keep already segmented crowns',
    },
    'crown-seg.extract-gingiva': {
        module: 'crown-seg',
        source: { kind: 'lucide', icon: Droplet },
        description: 'Extract gingiva segmentation',
    },

    // ── implant / surgical guide ──────────────────────────────────────
    'implant.any': {
        module: 'implant',
        source: { kind: 'exocad', path: '/icons/ImplantType.AnyImplant.Parameter.svg' },
        description: 'Any implant placeholder',
    },
    'implant.stock-abutment': {
        module: 'implant',
        source: {
            kind: 'exocad',
            path: '/icons/ImplantType.StockAbutment.Parameter.svg',
        },
        description: 'Stock abutment',
    },
    'implant.custom-abutment': {
        module: 'implant',
        source: {
            kind: 'exocad',
            path: '/icons/worktype/ImplantType.CustomAbutment.Parameter.svg',
        },
        description: 'Custom abutment',
    },
    'implant.post-core': {
        module: 'post-core',
        source: { kind: 'exocad', path: '/icons/worktype/ImplantType.PostAndCore.Parameter.svg' },
        description: 'Post and core',
    },
    'implant.surgical-guide': {
        module: 'surgical-guide',
        source: { kind: 'exocad', path: '/icons/SurgicalGuide.svg' },
        description: 'Surgical guide',
    },
    'implant.planning': {
        module: 'implant',
        source: { kind: 'exocad', path: '/icons/worktype/ImplantPlanningTooth.WorkType.svg' },
        description: 'Implant planning',
    },

    // ── splint ────────────────────────────────────────────────────────
    'splint.bite': {
        module: 'splint',
        source: { kind: 'exocad', path: '/icons/worktype/BiteSplint.WorkType.svg' },
        description: 'Bite splint',
    },

    // ── smile design ──────────────────────────────────────────────────
    'smile.design': {
        module: 'smile',
        source: { kind: 'lucide', icon: Smile },
        description: 'Smile design',
    },
    'smile.ai-crown': {
        module: 'smile',
        source: { kind: 'exocad', path: '/icons/CADAICrown.svg' },
        description: 'AI crown proposal',
    },

    // ── AI / segmentation ─────────────────────────────────────────────
    'ai.brain': {
        module: 'ai',
        source: { kind: 'lucide', icon: Brain },
        description: 'AI / neural segmentation',
    },
    'ai.chart': {
        module: 'ai',
        source: { kind: 'lucide', icon: PieChart },
        description: 'AI statistics / report',
    },

    // ── common ────────────────────────────────────────────────────────
    'common.close': {
        module: 'common',
        source: { kind: 'lucide', icon: X },
        description: 'Close',
    },
    'common.add': {
        module: 'common',
        source: { kind: 'lucide', icon: Plus },
        description: 'Add',
    },
    'common.remove': {
        module: 'common',
        source: { kind: 'lucide', icon: Minus },
        description: 'Remove',
    },
    'common.trash': {
        module: 'common',
        source: { kind: 'lucide', icon: Trash2 },
        description: 'Delete permanently',
    },
    'common.folder': {
        module: 'common',
        source: { kind: 'lucide', icon: Folder },
        description: 'Folder',
    },
    'common.image': {
        module: 'common',
        source: { kind: 'lucide', icon: Image },
        description: 'Image',
    },
    'common.file': {
        module: 'common',
        source: { kind: 'lucide', icon: FileText },
        description: 'File',
    },
    'common.share': {
        module: 'common',
        source: { kind: 'lucide', icon: Share2 },
        description: 'Share',
    },
    'common.upload': {
        module: 'common',
        source: { kind: 'exocad', path: '/icons/UploadIcon.svg' },
        description: 'Upload',
    },
    'common.download': {
        module: 'common',
        source: { kind: 'lucide', icon: Download },
        description: 'Download',
    },
    'common.print': {
        module: 'common',
        source: { kind: 'exocad', path: '/icons/PrintIcon.svg' },
        description: 'Print',
    },
    'common.settings': {
        module: 'common',
        source: { kind: 'lucide', icon: Settings },
        description: 'Settings',
    },
    'common.tools': {
        module: 'common',
        source: { kind: 'lucide', icon: Wrench },
        description: 'Tools',
    },
    'common.play': {
        module: 'common',
        source: { kind: 'lucide', icon: Play },
        description: 'Play / run',
    },
    'common.pencil': {
        module: 'common',
        source: { kind: 'lucide', icon: Pencil },
        description: 'Edit',
    },
    'common.sliders': {
        module: 'common',
        source: { kind: 'lucide', icon: Sliders },
        description: 'Sliders / parameters',
    },
    'common.info': {
        module: 'common',
        source: { kind: 'lucide', icon: Info },
        description: 'Info',
    },
    'common.warning': {
        module: 'common',
        source: { kind: 'lucide', icon: AlertTriangle },
        description: 'Warning',
    },
    'common.danger': {
        module: 'common',
        source: { kind: 'lucide', icon: TriangleAlert },
        description: 'Danger',
    },
    'common.eye': {
        module: 'common',
        source: { kind: 'lucide', icon: Eye },
        description: 'Show',
    },
    'common.eye-off': {
        module: 'common',
        source: { kind: 'lucide', icon: EyeOff },
        description: 'Hide',
    },
    'common.home': {
        module: 'common',
        source: { kind: 'lucide', icon: Home },
        description: 'Home',
    },
    'common.chevron-down': {
        module: 'common',
        source: { kind: 'lucide', icon: ChevronDown },
        description: 'Expand',
    },
    'common.arrow-left': {
        module: 'common',
        source: { kind: 'lucide', icon: ArrowLeft },
        description: 'Previous',
    },
    'common.arrow-right': {
        module: 'common',
        source: { kind: 'lucide', icon: ArrowRight },
        description: 'Next',
    },
    'common.activity': {
        module: 'common',
        source: { kind: 'lucide', icon: Activity },
        description: 'Activity / progress',
    },
    'common.grid': {
        module: 'common',
        source: { kind: 'lucide', icon: Grid3x3 },
        description: 'Grid',
    },
    'common.circle': {
        module: 'common',
        source: { kind: 'lucide', icon: Circle },
        description: 'Circle / radio',
    },
    'common.square': {
        module: 'common',
        source: { kind: 'lucide', icon: Square },
        description: 'Square / checkbox',
    },
    'common.crop': {
        module: 'common',
        source: { kind: 'lucide', icon: Crop },
        description: 'Crop',
    },
    'common.move': {
        module: 'common',
        source: { kind: 'lucide', icon: Move },
        description: 'Move',
    },
    'common.save': {
        module: 'common',
        source: { kind: 'lucide', icon: Save },
        description: 'Save',
    },
    'common.upload-cloud': {
        module: 'common',
        source: { kind: 'lucide', icon: Upload },
        description: 'Upload to cloud',
    },
    'common.printer': {
        module: 'common',
        source: { kind: 'lucide', icon: Printer },
        description: 'Printer',
    },
    'common.zoom-in': {
        module: 'common',
        source: { kind: 'lucide', icon: ZoomIn },
        description: 'Zoom in',
    },
    'common.zoom-out': {
        module: 'common',
        source: { kind: 'lucide', icon: ZoomOut },
        description: 'Zoom out',
    },

    // ── CAD wizard workflow (V23–V52 roadmap) ──────────────────────────
    'workflow.margin-detect': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/margin-detect.svg' },
        description: 'Auto-detect preparation margin (Detect mode)',
    },
    'workflow.margin-correct-draw': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/margin-correct-draw.svg' },
        description: 'Correct / draw margin manually',
    },
    'workflow.margin-repair-draw': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/margin-repair-draw.svg' },
        description: 'Repair scan geometry around margin',
    },
    'workflow.margin-subgingival': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/margin-subgingival.svg' },
        description: 'Subgingival margin detection hint',
    },
    'workflow.margin-supragingival': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/margin-supragingival.svg' },
        description: 'Supragingival margin detection hint',
    },
    'workflow.insertion-direction': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/insertion-direction.svg' },
        description: 'Insertion direction axis editor',
    },
    'workflow.undercut': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/undercut.svg' },
        description: 'Undercut visualization heatmap',
    },
    'workflow.cement-gap': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/cement-gap.svg' },
        description: 'Cement gap visualization',
    },
    'workflow.preop-scan': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/preop-scan.svg' },
        description: 'Pre-op scan / Waxup wizard step',
    },
    'workflow.cement-brush': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/cement-brush.svg' },
        description: 'Paint cement-gap zone',
    },
    'workflow.no-cement-brush': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/no-cement-brush.svg' },
        description: 'Paint no-cement-gap zone (green)',
    },
    'workflow.crown-border': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/crown-border.svg' },
        description: 'Crown border parameters (horizontal/angled/vertical)',
    },
    'workflow.block-undercuts': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/block-undercuts.svg' },
        description: 'Block-out undercuts advanced option',
    },
    'workflow.milling-tool': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/milling-tool.svg' },
        description: 'Milling tool compensation (round bur)',
    },
    'workflow.bullnose-tool': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/bullnose-tool.svg' },
        description: 'Bullnose / flat-end tool compensation',
    },
    'workflow.tooth-place-simple': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/tooth-place-simple.svg' },
        description: 'Tooth placement · Simple mode',
    },
    'workflow.tooth-place-advanced': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/tooth-place-advanced.svg' },
        description: 'Tooth placement · Advanced (Instant Anatomic Morphing)',
    },
    'workflow.tooth-place-chain': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/tooth-place-chain.svg' },
        description: 'Tooth placement · Chain mode',
    },
    'workflow.tooth-library-switch': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/tooth-library-switch.svg' },
        description: 'Switch tooth library',
    },
    'workflow.symmetry': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/symmetry.svg' },
        description: 'Left/right symmetry for tooth placement',
    },
    'workflow.free-form-brush': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/free-form-brush.svg' },
        description: 'Freeforming brush',
    },
    'workflow.min-thickness': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/min-thickness.svg' },
        description: 'Minimum thickness heatmap',
    },
    'workflow.distances': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/distances.svg' },
        description: 'Show distances between antagonist / adjacent',
    },
    'workflow.connector': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/connector.svg' },
        description: 'Connector designer',
    },
    'workflow.show-hide-groups': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/show-hide-groups.svg' },
        description: 'Show/Hide groups manager',
    },
    'workflow.expert-mode': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/expert-mode.svg' },
        description: 'Expert mode (context-menu + advanced options)',
    },
    'workflow.wizard-mode': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/wizard-mode.svg' },
        description: 'Wizard mode (guided step-by-step)',
    },
    'workflow.measurement': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/measurement.svg' },
        description: 'Measurement tool (distance / thickness / angle)',
    },
    'workflow.sectional-view': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/sectional-view.svg' },
        description: 'Sectional view (plane cut)',
    },
    'workflow.custom-view': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/custom-view.svg' },
        description: 'Add / name custom view',
    },
    'workflow.articulator': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/articulator.svg' },
        description: 'Virtual articulator',
    },
    'workflow.adjust-light': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/adjust-light.svg' },
        description: 'Adjust light from view',
    },
    'workflow.annotation': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/annotation.svg' },
        description: 'Annotation editor',
    },
    'workflow.align-meshes': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/align-meshes.svg' },
        description: 'Align meshes tool',
    },
    'workflow.context-menu': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/context-menu.svg' },
        description: 'Context menu (right-click)',
    },
    'workflow.free-text-search': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/free-text-search.svg' },
        description: 'Free-text feature search',
    },

    // ── Freeforming wizard (V53–V82 roadmap) ──────────────────────────
    'freeforming.adapt': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/adapt.svg' },
        description: 'Adapt restoration to antagonist/adjacents',
    },
    'freeforming.preset-cusps': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/preset-cusps.svg' },
        description: 'Preset: edit individual cusps',
    },
    'freeforming.preset-tooth-parts': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/preset-tooth-parts.svg' },
        description: 'Preset: edit mesial/distal/buccal/lingual',
    },
    'freeforming.preset-entire-tooth': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/preset-entire-tooth.svg' },
        description: 'Preset: edit entire tooth',
    },
    'freeforming.preset-ridge': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/preset-ridge.svg' },
        description: 'Preset: edit ridges / bulges',
    },
    'freeforming.occlusal-only': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/occlusal-only.svg' },
        description: 'Restrict transforms to occlusal direction',
    },
    'freeforming.lock-cusp-tips': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/lock-cusp-tips.svg' },
        description: 'Lock cusp tips during morphing',
    },
    'freeforming.lock-equator': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/lock-equator.svg' },
        description: 'Lock equator ring during morphing',
    },
    'freeforming.basal': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/basal.svg' },
        description: 'Basal adaptation (pontic → gingiva)',
    },
    'freeforming.static-occlusion': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/static-occlusion.svg' },
        description: 'Static occlusion adaptation',
    },
    'freeforming.dynamic-occlusion': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/dynamic-occlusion.svg' },
        description: 'Dynamic occlusion (articulator-based)',
    },
    'freeforming.approximal': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/approximal.svg' },
        description: 'Approximal adaptation',
    },
    'freeforming.direct-cut': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/direct-cut.svg' },
        description: 'Direct-cut adaptation',
    },
    'freeforming.shape-preserving': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/shape-preserving.svg' },
        description: 'Shape-preserving adaptation',
    },
    'freeforming.pull-sideways': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/pull-sideways.svg' },
        description: 'Pull sideways to adjacent teeth',
    },
    'freeforming.blocked-out-adjacent': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/blocked-out-adjacent.svg' },
        description: 'Blocked-out adjacent',
    },
    'freeforming.cut-with-disk': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/cut-with-disk.svg' },
        description: 'Cut with disk',
    },
    'freeforming.disk-add': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/disk-add.svg' },
        description: 'Add disk',
    },
    'freeforming.disk-tooth-axis': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/disk-tooth-axis.svg' },
        description: 'Orient disks toward tooth axis',
    },
    'freeforming.disk-view': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/disk-view.svg' },
        description: 'Orient disks toward current view',
    },
    'freeforming.paint-moving': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/paint-moving.svg' },
        description: 'Paint moving (green) areas',
    },
    'freeforming.paint-elastic': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/paint-elastic.svg' },
        description: 'Paint elastic (yellow) areas',
    },
    'freeforming.paint-static': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/paint-static.svg' },
        description: 'Paint static (blue) areas',
    },
    'freeforming.pull-moving': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/pull-moving.svg' },
        description: 'Pull painted moving parts',
    },
    'freeforming.emboss': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/emboss.svg' },
        description: 'Text attachment: emboss',
    },
    'freeforming.deboss': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/deboss.svg' },
        description: 'Text attachment: deboss',
    },
    'freeforming.insertion-top': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/insertion-top.svg' },
        description: 'Attachment insertion: Top (scan orientation)',
    },
    'freeforming.insertion-view': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/insertion-view.svg' },
        description: 'Attachment insertion: View direction',
    },
    'freeforming.insertion-surface': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/insertion-surface.svg' },
        description: 'Attachment insertion: perpendicular to surface',
    },
    'freeforming.brush-round': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/brush-round.svg' },
        description: 'Brush type: round ball',
    },
    'freeforming.brush-pointed': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/brush-pointed.svg' },
        description: 'Brush type: pointed knife',
    },
    'freeforming.brush-cylinder': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/brush-cylinder.svg' },
        description: 'Brush type: flat cylinder',
    },
    'freeforming.cut-intersections': {
        module: 'freeforming',
        source: { kind: 'exocad', path: '/icons/freeforming/cut-intersections.svg' },
        description: 'Cut all intersections (antagonist / adjacents / gingiva)',
    },

    // ─── V83–V87 · Primary Telescope Design ──────────────────────────
    'workflow.telescope': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/telescope.svg' },
        description: 'Primary Telescope Design wizard step',
    },
    'telescope.control-point': {
        module: 'telescope',
        source: { kind: 'exocad', path: '/icons/telescope/control-point.svg' },
        description: 'Lattice control point (draggable ball)',
    },
    'telescope.hat': {
        module: 'telescope',
        source: { kind: 'exocad', path: '/icons/telescope/hat.svg' },
        description: 'Segment state: friction surface (hat)',
    },
    'telescope.ufo': {
        module: 'telescope',
        source: { kind: 'exocad', path: '/icons/telescope/ufo.svg' },
        description: 'Segment state: conical (UFO)',
    },
    'telescope.tooth': {
        module: 'telescope',
        source: { kind: 'exocad', path: '/icons/telescope/tooth.svg' },
        description: 'Segment state: full anatomic (tooth)',
    },
    'telescope.deepen': {
        module: 'telescope',
        source: { kind: 'exocad', path: '/icons/telescope/deepen.svg' },
        description: 'Deepen surface area (Ctrl+Shift-click)',
    },

    // ─── V88–V90 · Merge & Save Restorations ─────────────────────────
    'workflow.merge-save': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/merge-save.svg' },
        description: 'Merge and Save Restorations wizard step',
    },

    // ─── V92–V93 · Screw Holes ───────────────────────────────────────
    'workflow.screw-holes': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/screw-holes.svg' },
        description: 'Screw Holes wizard step (post-merge)',
    },
    'screw.channel': {
        module: 'screw',
        source: { kind: 'exocad', path: '/icons/screw/channel.svg' },
        description: 'Screw channel definition',
    },
    'screw.cut-hole': {
        module: 'screw',
        source: { kind: 'exocad', path: '/icons/screw/cut-hole.svg' },
        description: 'Per-tooth chart: cut hole (green)',
    },
    'screw.no-hole': {
        module: 'screw',
        source: { kind: 'exocad', path: '/icons/screw/no-hole.svg' },
        description: 'Per-tooth chart: no hole (red)',
    },

    // ─── V99–V105 · Bite Splint workflow ─────────────────────────────
    'workflow.bite-splint': {
        module: 'workflow',
        source: { kind: 'exocad', path: '/icons/workflow/bite-splint.svg' },
        description: 'Bite Splint workflow entry',
    },
    'bite-splint.standard': {
        module: 'bite-splint',
        source: { kind: 'exocad', path: '/icons/bite-splint/standard.svg' },
        description: 'Bite splint mode: Standard',
    },
    'bite-splint.fill-gap': {
        module: 'bite-splint',
        source: { kind: 'exocad', path: '/icons/bite-splint/fill-gap.svg' },
        description: 'Bite splint mode: Standard — fill gap for missing tooth',
    },
    'bite-splint.anatomical': {
        module: 'bite-splint',
        source: { kind: 'exocad', path: '/icons/bite-splint/anatomical.svg' },
        description: 'Bite splint mode: Anatomical — use anatomy from tooth library',
    },
    'bite-splint.block-undercut': {
        module: 'bite-splint',
        source: { kind: 'exocad', path: '/icons/bite-splint/block-undercut.svg' },
        description: 'Block out undercuts (Design Bite Splint Bottom)',
    },
    'bite-splint.ai-cloud': {
        module: 'bite-splint',
        source: { kind: 'exocad', path: '/icons/bite-splint/ai-cloud.svg' },
        description: 'Cloud services — AI segmentation / margin detection',
    },

    // ─── V106–V107 · Tools (sectional view + articulator already registered above) ───

    // ─── V109–V112 · Productivity layer ──────────────────────────────
    'palette.command': {
        module: 'palette',
        source: { kind: 'exocad', path: '/icons/palette/command.svg' },
        description: 'Universal command palette (Cmd/Ctrl+K)',
    },
    'palette.keybindings': {
        module: 'palette',
        source: { kind: 'exocad', path: '/icons/palette/keybindings.svg' },
        description: 'Keybindings editor',
    },
    'palette.workspace-layout': {
        module: 'palette',
        source: { kind: 'exocad', path: '/icons/palette/workspace-layout.svg' },
        description: 'Workspace layouts switcher',
    },
    'palette.dashboard': {
        module: 'palette',
        source: { kind: 'exocad', path: '/icons/palette/dashboard.svg' },
        description: 'Lab throughput dashboard',
    },

    // ─── V143 · Abutment design ──────────────────────────────────────
    'abutment.style-cylindrical': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/style-cylindrical.svg' },
        description: 'Abutment style — Cylindrical (small shoulder, smooth)',
    },
    'abutment.style-angular': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/style-angular.svg' },
        description: 'Abutment style — Angular (moderate shoulder, pronounced angularity)',
    },
    'abutment.style-standard': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/style-standard.svg' },
        description: 'Abutment style — Standard (average shoulder, moderate angularity)',
    },
    'abutment.style-legacy': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/style-legacy.svg' },
        description: 'Abutment style — Legacy (3.2 software replication)',
    },
    'abutment.screw-channel-straight': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/screw-channel-straight.svg' },
        description: 'Straight screw channel',
    },
    'abutment.screw-channel-angulated': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/screw-channel-angulated.svg' },
        description: 'Angulated screw channel (clickable / draggable)',
    },
    'abutment.fissure-line': {
        module: 'abutment',
        source: { kind: 'exocad', path: '/icons/abutment/fissure-line.svg' },
        description: 'Connect fissure controls (central fossa adjustment)',
    },
};

export type AppIconName = keyof typeof APP_ICONS;

/**
 * Report icons that resolve to `missing` — used by docs generation so the
 * user knows which pictograms need commissioning.
 */
export function listMissingIcons(): Array<{ name: string; entry: IconEntry }> {
    return Object.entries(APP_ICONS)
        .filter(([, entry]) => entry.source.kind === 'missing')
        .map(([name, entry]) => ({ name, entry }));
}

export function resolveIcon(name: string): IconEntry | null {
    return APP_ICONS[name] ?? null;
}

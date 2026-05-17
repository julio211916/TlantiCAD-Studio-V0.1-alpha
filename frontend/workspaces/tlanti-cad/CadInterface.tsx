import React, { lazy, useState, useRef, useCallback, Suspense, useEffect, useMemo } from 'react';
import { Icons } from './Icons';
import { THEME, ToolMode, FileData, MeshMetadata, DicomMetadata, Language, ThemeMode } from '../types';
import { TRANSLATIONS } from '../utils/translations';
import { handleDirectoryUpload, handleFileUpload, type FileUploadEntry } from '../utils/fileLoader';
import { calculateMeshStats, subdivideGeometry } from '../utils/meshUtils';
import * as THREE from 'three';
import { v4 as uuidv4 } from 'uuid';
import clsx from 'clsx';
import { DicomControls } from './DicomControls';
import { DropdownNavigation } from './ui/dropdown-navigation';
import { CanvasScene } from './CanvasScene';
import { CadCommandPanels } from './CadCommandPanels';
import { CadOverlays } from './CadOverlays';
import { ModulePanelErrorBoundary } from './ModulePanelErrorBoundary';
import { CadWorkbenchShell } from './CadWorkbenchShell';
import { CadWizardSlot } from './CadWizardSlot';
import type { MergeToothPayload } from '@/features/merge-save';
import {
    ArticulatorContainer,
    InfluencingTeethDialog,
    JawMotionOverlay,
    createBackendArticulatorAdapter,
} from '@/features/articulator';
import type { JawFrame } from '@/features/articulator';
import { hotkeyRegistry } from '@/features/hotkeys';
import { commandRegistry } from '@/features/command-palette';
import {
    defineCadCommandPanelsViewModel,
    defineCanvasSceneViewModel,
} from './view-models/cad-workbench-view-models';
import { useCadInterfaceViewModel } from './view-models/useCadInterfaceViewModel';
import { type PublicAssetLibraryActionMode } from '@/components/asset-library/PublicAssetLibraryBrowser';
import { TlantiWorkspacePreloader } from '../TlantiWorkspacePreloader';
import { useToast } from './ui/use-toast';
import { WORKSPACE_TITLES } from '../workspace.config';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { isTauriRuntime } from '@/platform/desktop-system';
import { inferCadPresetEffect } from '@/lib/public-asset-cad-presets';
import { consumePendingCadImport } from '@/lib/launcher-pending-import';
import { getSmileDesignPlaybookById } from '@/lib/smile-design-playbooks';
import { loadTlantiDbState, saveTlantiDbState, subscribeTlantiDbState, type TlantiCase, type TlantiDbPreferences } from '@/stores/tlantidb-case-store';
import { useSmileDesignWorkflowStore } from '@/stores/smile-design-workflow-store';
import {
    CAD_MODULE_ROADMAP_DEFINITIONS,
    CAD_EXPERT_TOOL_MODE_IDS,
    CAD_PRODUCT_MODULE_DEFINITIONS,
    CAD_VIEWPORT_TOOL_MODE_IDS,
    MeshVaultImportUseCase,
    TauriMeshVault,
    listCadProductModules,
    resolveCadToolDefinition,
    resolveCadProductModuleForRoute,
    resolveTlantiModuleDefinition,
    type CadCoreAssetKind,
    type CadCoreModuleId,
    type TlantiCadModuleWorkflowPhase,
    type TlantiCadProductModuleId,
} from '@/core';
import type { PublicAssetLibraryItem } from '@/lib/public-asset-library';
import { buildCaseStatusItems, resolveWorkspaceModuleDefinition } from '@/lib/workspace-shell';
import { WorkspaceStatusStrip } from '@/components/ui/workspace-status-strip';
import { DentalCadShellBar } from '@/features/cad-shell/components/DentalCadShellBar';
import { DentalCadWorkbenchRails } from '@/features/cad-shell/components/DentalCadWorkbenchRails';
import {
    DENTAL_CAD_SHELL_ACTIONS,
    DENTAL_CAD_PRODUCT_MODULE_TOOLSETS,
    DENTAL_CAD_MODULE_TOOLSETS,
    type DentalCadShellActionId,
} from '@/features/cad-shell/config/dental-cad-shell';
import {
    Brush, Slice, Maximize, Ruler, Globe, Box, Grid3X3, Save, Eye, EyeOff, MoreHorizontal,
    ArrowDownToLine, Copy, Upload, Layers, LibraryBig, PenTool, Merge, Minimize2, Minimize, Cylinder, AlignCenter, RotateCw, Magnet,
    Settings, Languages, Sun, Moon, Sparkles, ChevronRight, ChevronLeft, BrainCircuit, Activity, HeartPulse,
    Calculator, Triangle, Circle, Square, ChevronDown, ChevronUp, ScanLine, Printer, Smile, Pencil, Trash2, Keyboard, MousePointer2,
    Undo2, Redo2, Download, Search, SortAsc, Clock, Filter, ZoomIn, ZoomOut, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Crosshair, Scissors, Cloud, PieChart, Cuboid, Move, X, Crop, Activity as ToothIcon,
    Cpu, Hash, Users, FolderPlus, Syringe, Target, Settings2, HelpCircle, BookmarkPlus, FolderTree, Bot, MoonStar
} from 'lucide-react';
import { AnimatePresence } from 'framer-motion';
import { FloatingDock } from '@/components/ui/floating-dock';
import { getBrowserCapabilityState } from '@/lib/browser-capability-state';
import type { CadModuleSurface } from '@/lib/workspace-shell-contract';

const TlantiOhifViewer = lazy(() => import('./dicom-viewer/DicomSeriesViewer').then((module) => ({ default: module.TlantiOhifViewer })));
const CadToolsWorkflowsPanel = lazy(() => import('./cad/CadToolsWorkflowsPanel'));
const FileManager = lazy(() => import('./FileManager').then((module) => ({ default: module.FileManager })));
const PropertiesPanel = lazy(() => import('./PropertiesPanel').then((module) => ({ default: module.PropertiesPanel })));
const Odontogram = lazy(() => import('./Odontogram').then((module) => ({ default: module.Odontogram })));

const CAD_TOOL_ICON_BY_MODE = {
    SELECT: Icons.Select,
    MOVE: Icons.Move,
    ROTATE: Icons.Rotate,
    SCALE: Icons.Scale,
    CLIP: Slice,
    CROP: Crop,
    BOOLEAN_CUT: Scissors,
    SCULPT: Brush,
    SEGMENT: PieChart,
    MEASURE: Icons.Measure,
    INSERTION: ArrowDownToLine,
    CROWN: Layers,
    COPY: Copy,
    FREEFORM: Brush,
    CONNECTORS: Merge,
    THICKNESS: Minimize2,
    ARTICULATOR: Box,
    ALIGN: AlignCenter,
    EXPORT_PROD: Printer,
} as const;

const CAD_TOOL_LABEL_BY_MODE: Partial<Record<keyof typeof CAD_TOOL_ICON_BY_MODE, string>> = {
    SELECT: 'Select (Q)',
    MOVE: 'Move (W)',
    ROTATE: 'Rotate (E)',
    SCALE: 'Scale (R)',
    MEASURE: 'Measure (Ctrl+R)',
};
const DicomViewerModule = lazy(() => import('./DicomViewerModule').then((module) => ({ default: module.DicomViewerModule })));
const ImplantModule = lazy(() => import('./ImplantModule').then((module) => ({ default: module.ImplantModule })));
const SurgicalGuideModule = lazy(() => import('./SurgicalGuideModule').then((module) => ({ default: module.SurgicalGuideModule })));
const SplintModule = lazy(() => import('./SplintModule').then((module) => ({ default: module.SplintModule })));
const CephModule = lazy(() => import('./CephModule').then((module) => ({ default: module.CephModule })));
const FabModule = lazy(() => import('./FabModule').then((module) => ({ default: module.FabModule })));
const AlignersModule = lazy(() => import('./AlignersModule').then((module) => ({ default: module.AlignersModule })));
const CadAssetLibraryPanel = lazy(() => import('./cad/CadAssetLibraryPanel').then((module) => ({ default: module.CadAssetLibraryPanel })));
const CadGroupSelectorPanel = lazy(() => import('./cad/CadGroupSelectorPanel').then((module) => ({ default: module.CadGroupSelectorPanel })));
const CadWorkflowGuidePanel = lazy(() => import('./cad/CadWorkflowGuidePanel').then((module) => ({ default: module.CadWorkflowGuidePanel })));
const CadVoiceCopilotPanel = lazy(() => import('@/features/voice-copilot/components/CadVoiceCopilotPanel').then((module) => ({ default: module.CadVoiceCopilotPanel })));
const SmileDesignWorkflowPanel = lazy(() => import('@/features/smile-design/components/SmileDesignWorkflowPanel').then((module) => ({ default: module.default })));

interface CadInterfaceProps {
    language: Language;
    setLanguage: (lang: Language) => void;
    themeMode: ThemeMode;
    setThemeMode: (mode: ThemeMode) => void;
    caseId?: string;
    moduleId?: string;
    onBackToDb?: () => void;
}

const DICOM_FILE_PATTERN = /\.(dcm|dicom|ima)$/i;
const ZIP_FILE_PATTERN = /\.zip$/i;
const PATH_BACKED_MESH_FILE_PATTERN = /\.(stl|obj|ply|3mf|dcm|dicom|ima)$/i;
const DICOM_CONTEXT_MODULES = new Set([
    'dicom',
    'implant',
    'guide',
    'splint',
    'orthocad',
    'aligners',
    'ceph',
    'fab',
]);

function normalizePath(path: string) {
    return path.replace(/\\/g, '/');
}

function getPathDirectory(path: string) {
    const normalized = normalizePath(path);
    const lastSlash = normalized.lastIndexOf('/');
    return lastSlash === -1 ? '' : normalized.slice(0, lastSlash);
}

function getPathBasename(path: string) {
    const normalized = normalizePath(path);
    const lastSlash = normalized.lastIndexOf('/');
    return lastSlash === -1 ? normalized : normalized.slice(lastSlash + 1);
}

function getLastDirectoryName(path: string) {
    const normalized = normalizePath(path).replace(/\/+$/, '');
    const lastSlash = normalized.lastIndexOf('/');
    return lastSlash === -1 ? normalized : normalized.slice(lastSlash + 1);
}

function inferMeshVaultKindFromPath(path: string): CadCoreAssetKind | null {
    const lower = path.toLowerCase();
    if (/\.(dcm|dicom|ima)$/.test(lower)) return 'dicom-series';
    if (/\.stl$/.test(lower)) return 'stl-mesh';
    if (/\.obj$/.test(lower)) return 'obj-mesh';
    if (/\.ply$/.test(lower)) return 'ply-mesh';
    if (/\.3mf$/.test(lower)) return 'manufacturing-export';
    return null;
}

function formatMeshVaultImportMessage(summary: { queued: number; completed: number; fallback: number }) {
    const parts: string[] = [];
    if (summary.completed) {
        parts.push(`${summary.completed} completed in Mesh Vault`);
    }
    if (summary.queued) {
        parts.push(`${summary.queued} queued in Mesh Vault`);
    }
    if (summary.fallback) {
        parts.push(`${summary.fallback} imported in browser fallback`);
    }

    return parts.length ? `${parts.join(' / ')}.` : undefined;
}

type PathBackedImportResult = Awaited<ReturnType<MeshVaultImportUseCase['execute']>> & {
    meshVaultStatus: 'completed' | 'queued';
};

type FunctionalSelectionKey = 'antagonists' | 'truSmile' | 'colorTexture' | 'cutView' | 'smileView' | 'cloud';

type SavedViewPreset = {
    id: string;
    name: string;
    position: [number, number, number];
    target: [number, number, number];
};

export const CadInterface: React.FC<CadInterfaceProps> = ({ language, setLanguage, themeMode, setThemeMode, caseId, moduleId, onBackToDb }) => {
    const { toast } = useToast();
    const viewport = useViewportProfile();
    const defaultNavigationSensitivity: TlantiDbPreferences['navigationSensitivity'] = {
        zoom: 0.8,
        pan: 0.8,
        rotation: 0.8,
    };
    const defaultCadUiMode: TlantiDbPreferences['cadUiMode'] = 'clinical-clean';
    const defaultControlScheme: TlantiDbPreferences['controlScheme'] = 'cad';
    const initialWorkspacePreferences = (() => {
        try {
            return loadTlantiDbState().preferences;
        } catch {
            return null;
        }
    })();
    const [navigationSensitivity, setNavigationSensitivity] = useState<TlantiDbPreferences['navigationSensitivity']>(() => {
        return initialWorkspacePreferences?.navigationSensitivity ?? defaultNavigationSensitivity;
    });
    const [cadUiMode, setCadUiMode] = useState<TlantiDbPreferences['cadUiMode']>(() => {
        return initialWorkspacePreferences?.cadUiMode ?? defaultCadUiMode;
    });
    const [isExpertMode, setIsExpertMode] = useState(false);
    const [isNavOpen, setIsNavOpen] = useState(false);
    const [isOdontogramOpen, setIsOdontogramOpen] = useState(false);
    const [isDicomModuleOpen, setIsDicomModuleOpen] = useState(false);
    const [isImplantModuleOpen, setIsImplantModuleOpen] = useState(false);
    const [isSurgicalGuideModuleOpen, setIsSurgicalGuideModuleOpen] = useState(false);
    const [isSplintModuleOpen, setIsSplintModuleOpen] = useState(false);
    const [isCephModuleOpen, setIsCephModuleOpen] = useState(false);
    const [isFabModuleOpen, setIsFabModuleOpen] = useState(false);
    const [isAlignersModuleOpen, setIsAlignersModuleOpen] = useState(false);
    const [isToolsWorkflowsOpen, setIsToolsWorkflowsOpen] = useState(false);
    const [isFileManagerOpen, setIsFileManagerOpen] = useState(false);
    const [isPropertiesPanelOpen, setIsPropertiesPanelOpen] = useState(false);
    const [isAssetLibraryOpen, setIsAssetLibraryOpen] = useState(false);
    const [isGroupSelectorOpen, setIsGroupSelectorOpen] = useState(false);
    const [isCadGuideOpen, setIsCadGuideOpen] = useState(false);
    const [isSmileWorkflowOpen, setIsSmileWorkflowOpen] = useState(false);
    const [isCadWizardOpen, setIsCadWizardOpen] = useState(true); // V113: wizard mounted by default in CAD shell
    const [isArticulatorOpen, setIsArticulatorOpen] = useState(false); // V207
    const [articulatorFrames, setArticulatorFrames] = useState<JawFrame[]>([]); // V210
    const [isInfluencingTeethOpen, setIsInfluencingTeethOpen] = useState(false); // V211
    const [isCopilotOpen, setIsCopilotOpen] = useState(false);
    const [activeCadPreset, setActiveCadPreset] = useState<PublicAssetLibraryItem | null>(null);
    const [activeDentalLibrary, setActiveDentalLibrary] = useState<PublicAssetLibraryItem | null>(null);
    const [activeOverlayAsset, setActiveOverlayAsset] = useState<PublicAssetLibraryItem | null>(null);
    const [activeOverlayFileId, setActiveOverlayFileId] = useState<string | null>(null);
    const browserCapabilities = useMemo(() => getBrowserCapabilityState(), []);
    const [activeOverlayAlignment, setActiveOverlayAlignment] = useState<'front' | 'plane'>('front');
    const [moduleLoadingLabel, setModuleLoadingLabel] = useState<string | null>(null);
    const [savedViews, setSavedViews] = useState<SavedViewPreset[]>([]);
    const [functionalSelections, setFunctionalSelections] = useState<Record<FunctionalSelectionKey, boolean>>({
        antagonists: false,
        truSmile: false,
        colorTexture: false,
        cutView: false,
        smileView: false,
        cloud: false,
    });
    const t = TRANSLATIONS[language];
    const selectedSmilePlaybookId = useSmileDesignWorkflowStore((state) => state.selectedPlaybookId);
    const smileCurrentStageByPlaybook = useSmileDesignWorkflowStore((state) => state.currentStageByPlaybook);
    const smileNextStage = useSmileDesignWorkflowStore((state) => state.nextStage);
    const smilePreviousStage = useSmileDesignWorkflowStore((state) => state.previousStage);
    const selectSmilePlaybook = useSmileDesignWorkflowStore((state) => state.selectPlaybook);

    const [activeTool, setActiveTool] = useState<ToolMode>('SELECT');

    // History State for Undo/Redo
    const [filesInternal, setFilesInternal] = useState<FileData[]>([]);
    const [history, setHistory] = useState<FileData[][]>([[]]);
    const [historyPointer, setHistoryPointer] = useState<number>(0);

    const setFiles = useCallback((valOrFunc: React.SetStateAction<FileData[]>) => {
        setFilesInternal((prev) => {
            const nextState = typeof valOrFunc === 'function' ? (valOrFunc as any)(prev) : valOrFunc;

            setHistory((prevHistory) => {
                const newHistory = prevHistory.slice(0, historyPointer + 1);
                newHistory.push(nextState);
                // Limit history size to prevent memory leaks
                if (newHistory.length > 30) {
                    newHistory.shift();
                    setHistoryPointer(p => p - 1);
                }
                return newHistory;
            });
            setHistoryPointer((prevPointer) => Math.min(prevPointer + 1, 30));

            return nextState;
        });
    }, [historyPointer]);

    const undo = useCallback(() => {
        if (historyPointer > 0) {
            const newPointer = historyPointer - 1;
            setHistoryPointer(newPointer);
            setFilesInternal(history[newPointer]);
        }
    }, [history, historyPointer]);

    const redo = useCallback(() => {
        if (historyPointer < history.length - 1) {
            const newPointer = historyPointer + 1;
            setHistoryPointer(newPointer);
            setFilesInternal(history[newPointer]);
        }
    }, [history, historyPointer]);

    const files = filesInternal;

    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const selectedFile = files.find((file) => file.id === selectedId);
    const cadViewModel = useCadInterfaceViewModel({ files, selectedFile });
    const [transformSpace, setTransformSpace] = useState<'local' | 'world'>('local');
    const [gridVisible, setGridVisible] = useState(true);
    const [snapValue, setSnapValue] = useState<number | null>(null);

    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [shortcutsOpen, setShortcutsOpen] = useState(false);

    const [controlScheme, setControlScheme] = useState<'standard' | 'cad'>(initialWorkspacePreferences?.controlScheme ?? defaultControlScheme);
    const dragStartRef = useRef<{ x: number, y: number } | null>(null);

    // Sidebar Sections State
    const [openSections, setOpenSections] = useState<Record<string, boolean>>({
        layers: true,
        dental: false,
        smile: false,
        algebra: false,
        analysis: true,
        dicom: true,
        ai: false,
        settings: false,
        meshTools: false
    });

    const toggleSection = (section: string) => {
        setOpenSections(prev => ({ ...prev, [section]: !prev[section] }));
    };

    const closeClinicalModulePanels = useCallback((keep?: CadModuleSurface) => {
        if (keep !== 'odontogram') setIsOdontogramOpen(false);
        if (keep !== 'dicom') setIsDicomModuleOpen(false);
        if (keep !== 'implant') setIsImplantModuleOpen(false);
        if (keep !== 'guide') setIsSurgicalGuideModuleOpen(false);
        if (keep !== 'splint') setIsSplintModuleOpen(false);
        if (keep !== 'smile') setIsSmileWorkflowOpen(false);
        if (keep !== 'ceph') setIsCephModuleOpen(false);
        if (keep !== 'fab') setIsFabModuleOpen(false);
        if (keep !== 'aligners') setIsAlignersModuleOpen(false);
        // V143 — wizard slot (margin/insertion/...) only belongs to CAD-design modules.
        // Close it when navigating to clinical modules so the panel does not bleed across.
        if (keep !== 'wizard') setIsCadWizardOpen(false);
    }, []);

    // V143 — when the user closes a clinical module panel via its ✕ button we must
    // also drop `?module=` from the URL so a subsequent refresh does not re-open
    // the same module (BUG-011 root cause: URL persists after close).
    const exitClinicalModule = useCallback(() => {
        if (typeof window !== 'undefined') {
            const url = new URL(window.location.href);
            if (url.searchParams.has('module')) {
                url.searchParams.delete('module');
                window.history.replaceState({}, '', url.toString());
            }
        }
        handledModuleLaunchRef.current = null;
        closeClinicalModulePanels();
        setIsCadWizardOpen(true);
    }, [closeClinicalModulePanels]);

    const openModuleWithPreloader = useCallback((label: string, open: () => void) => {
        if (moduleLoaderTimerRef.current) {
            window.clearTimeout(moduleLoaderTimerRef.current);
        }

        setModuleLoadingLabel(label);
        moduleLoaderTimerRef.current = window.setTimeout(() => {
            try {
                open();
            } catch (error) {
                console.error('[CadInterface] module open failed', label, error);
            }
            setModuleLoadingLabel(null);
            moduleLoaderTimerRef.current = null;
        }, 720);

        // V143 — safety net: if for any reason the preloader is not cleared by the
        // primary timer (HMR race, suspended import, etc.) force-clear it after 4s
        // so the user is never stuck on the splash (BUG-002 root cause).
        window.setTimeout(() => {
            setModuleLoadingLabel((current) => (current === label ? null : current));
        }, 4000);
    }, []);

    // Global palette is owned by App.tsx (CommandPalette + useCommandPalette);
    // local code dispatches a synthetic Cmd/Ctrl+K to open it from buttons /
    // context menus / voice copilot.
    const openCommandPalette = useCallback(() => {
        if (typeof window === 'undefined') return;
        window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true }));
    }, []);

    const launchImplantModule = useCallback(() => {
        closeClinicalModulePanels('implant');
        openModuleWithPreloader(WORKSPACE_TITLES.implant, () => setIsImplantModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchGuideModule = useCallback(() => {
        closeClinicalModulePanels('guide');
        openModuleWithPreloader(WORKSPACE_TITLES.guide, () => setIsSurgicalGuideModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchDicomModule = useCallback(() => {
        closeClinicalModulePanels('dicom');
        openModuleWithPreloader('DICOM Viewer', () => setIsDicomModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchSplintModule = useCallback(() => {
        closeClinicalModulePanels('splint');
        openModuleWithPreloader(WORKSPACE_TITLES.splint, () => setIsSplintModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchOdontogramModule = useCallback(() => {
        if (isOdontogramOpen) {
            setIsOdontogramOpen(false);
            return;
        }
        closeClinicalModulePanels('odontogram');
        openModuleWithPreloader(WORKSPACE_TITLES.odontogram, () => setIsOdontogramOpen(true));
    }, [closeClinicalModulePanels, isOdontogramOpen, openModuleWithPreloader]);

    const launchAlignersModule = useCallback(() => {
        closeClinicalModulePanels('aligners');
        openModuleWithPreloader('Aligners', () => setIsAlignersModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchCephModule = useCallback(() => {
        closeClinicalModulePanels('ceph');
        openModuleWithPreloader('Cefalometría', () => setIsCephModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    const launchFabModule = useCallback(() => {
        closeClinicalModulePanels('fab');
        openModuleWithPreloader('Fabricación', () => setIsFabModuleOpen(true));
    }, [closeClinicalModulePanels, openModuleWithPreloader]);

    // Single source of truth for deep-link `?module=…` routing, used by the
    // mount-time switch and the `runModuleAction('command-palette')` voice path.
    const cadModuleLaunchers = useMemo<Map<string, () => void>>(() => {
        const onCadDesign = (tool: ToolMode) => {
            setIsCadWizardOpen(true);
            setActiveTool(tool);
        };
        return new Map<string, () => void>([
            ['cad', () => onCadDesign('SELECT')],
            ['partials', () => onCadDesign('SEGMENT')],
            ['orthocad', () => {
                selectSmilePlaybook('ai-mockup');
                closeClinicalModulePanels('smile');
                setIsSmileWorkflowOpen(true);
                setActiveTool('SCULPT');
            }],
            ['dicom', () => { launchDicomModule(); setActiveTool('MEASURE'); }],
            ['ceph', () => { launchCephModule(); setActiveTool('MEASURE'); }],
            ['fab', () => { launchFabModule(); setActiveTool('SELECT'); }],
            ['aligners', () => { launchAlignersModule(); setActiveTool('SCULPT'); }],
            ['guide', () => { launchGuideModule(); setActiveTool('MEASURE'); }],
            ['splint', () => { launchSplintModule(); setActiveTool('MEASURE'); }],
            ['implant', () => { launchImplantModule(); setActiveTool('MEASURE'); }],
            ['model-creator', () => setActiveTool('SELECT')],
        ]);
    }, [closeClinicalModulePanels, launchAlignersModule, launchCephModule, launchDicomModule, launchFabModule, launchGuideModule, launchImplantModule, launchSplintModule, selectSmilePlaybook]);

    // Renaming State
    const [renamingId, setRenamingId] = useState<string | null>(null);
    const [renameValue, setRenameValue] = useState('');

    // File Manager Search & Sort
    const [searchQuery, setSearchQuery] = useState('');
    const [sortOption, setSortOption] = useState<'date' | 'alpha'>('date');

    const filteredAndSortedFiles = useMemo(() => {
        let result = [...files];
        if (searchQuery) {
            result = result.filter(f => f.name.toLowerCase().includes(searchQuery.toLowerCase()));
        }
        if (sortOption === 'alpha') {
            result.sort((a, b) => a.name.localeCompare(b.name));
        }
        return result;
    }, [files, searchQuery, sortOption]);

    // Sidebar Tabs
    const [activeSidebarTab, setActiveSidebarTab] = useState<'project' | 'tools' | 'ai'>('project');

    // Dental State
    const [selectedTeeth, setSelectedTeeth] = useState<number[]>([]);

    const [aiPrompt, setAiPrompt] = useState('');
    const [aiResponse, setAiResponse] = useState('');
    const [isAiProcessing, setIsAiProcessing] = useState(false);

    const [measurePoints, setMeasurePoints] = useState<THREE.Vector3[]>([]);
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number, type: 'object' | 'canvas', targetId?: string } | null>(null);

    const [cropStart, setCropStart] = useState<{ x: number, y: number } | null>(null);
    const [cropEnd, setCropEnd] = useState<{ x: number, y: number } | null>(null);
    const [cropPlanes, setCropPlanes] = useState<THREE.Plane[]>([]);

    const fileInputRef = useRef<HTMLInputElement>(null);
    const directoryInputRef = useRef<HTMLInputElement>(null);
    const orbitRef = useRef<any>(null);
    const transformRef = useRef<any>(null);
    const moduleLoaderTimerRef = useRef<number | null>(null);
    const [selectedMesh, setSelectedMesh] = useState<THREE.Object3D | null>(null);

    const activeCaseContext = useMemo<TlantiCase | null>(() => {
        if (typeof window === 'undefined') {
            return null;
        }

        try {
            const state = loadTlantiDbState();
            if (caseId) {
                return state.cases.find((item) => item.id === caseId) ?? null;
            }

            return state.cases.find((item) => item.id === state.activeCaseId) ?? state.cases[0] ?? null;
        } catch {
            return null;
        }
    }, [caseId]);

    useEffect(() => {
        return subscribeTlantiDbState((state) => {
            setCadUiMode(state.preferences.cadUiMode ?? defaultCadUiMode);
            setNavigationSensitivity(state.preferences.navigationSensitivity ?? defaultNavigationSensitivity);
            setControlScheme(state.preferences.controlScheme ?? defaultControlScheme);
        });
    }, []);

    // Hotkeys registered while CAD shell is mounted: Articulator (Ctrl+J),
    // module launchers (Ctrl+1..7) and viewport toggles (g grid, shift+e expert)
    // so the global hotkey help overlay and command palette discover them.
    useEffect(() => {
        const disposers = [
            hotkeyRegistry.register({
                chord: 'ctrl+j',
                label: 'Toggle Articulator',
                description: 'Show / hide the virtual articulator panel',
                context: 'global',
                run: () => setIsArticulatorOpen((v) => !v),
                paletteAction: {
                    id: 'articulator.toggle',
                    label: 'Toggle Articulator panel',
                    kind: 'tool',
                    keywords: ['articulator', 'jaw', 'occlusion', 'condyle', 'bennett'],
                },
            }),
            hotkeyRegistry.register({
                chord: 'g',
                label: 'Toggle grid',
                description: 'Show / hide viewport ground grid',
                context: 'global',
                run: () => setGridVisible((value) => !value),
                paletteAction: {
                    id: 'cad.toggle.grid',
                    label: 'Toggle viewport grid',
                    kind: 'toggle',
                    keywords: ['grid', 'viewport', 'reference'],
                },
            }),
            hotkeyRegistry.register({
                chord: 'shift+e',
                label: 'Toggle expert mode',
                description: 'Switch CAD shell between assistant and expert UI',
                context: 'global',
                run: () => setIsExpertMode((value) => !value),
                paletteAction: {
                    id: 'cad.toggle.expert',
                    label: 'Toggle expert mode',
                    kind: 'toggle',
                    keywords: ['expert', 'assistant', 'ui', 'mode'],
                },
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+1',
                label: 'Open Implant module',
                context: 'global',
                run: () => launchImplantModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+2',
                label: 'Open Surgical Guide module',
                context: 'global',
                run: () => launchGuideModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+3',
                label: 'Open Splint module',
                context: 'global',
                run: () => launchSplintModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+4',
                label: 'Open DICOM Viewer',
                context: 'global',
                run: () => launchDicomModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+5',
                label: 'Open Cephalometry',
                context: 'global',
                run: () => launchCephModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+6',
                label: 'Open Fabrication',
                context: 'global',
                run: () => launchFabModule(),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+7',
                label: 'Open Aligners',
                context: 'global',
                run: () => launchAlignersModule(),
            }),
        ];
        return () => {
            for (const d of disposers) d();
        };
    }, [launchAlignersModule, launchCephModule, launchDicomModule, launchFabModule, launchGuideModule, launchImplantModule, launchSplintModule]);

    // Register module-launcher and toggle actions in the global command palette.
    useEffect(() => {
        return commandRegistry.registerAll([
            {
                id: 'cad.module.implant',
                label: 'Open Implant module',
                kind: 'navigation',
                keywords: ['implant', 'planning', 'screw', 'fixture'],
                run: () => launchImplantModule(),
            },
            {
                id: 'cad.module.guide',
                label: 'Open Surgical Guide module',
                kind: 'navigation',
                keywords: ['guide', 'surgical', 'sleeve'],
                run: () => launchGuideModule(),
            },
            {
                id: 'cad.module.splint',
                label: 'Open Splint module',
                kind: 'navigation',
                keywords: ['splint', 'nightguard', 'occlusion'],
                run: () => launchSplintModule(),
            },
            {
                id: 'cad.module.dicom',
                label: 'Open DICOM Viewer',
                kind: 'navigation',
                keywords: ['dicom', 'cbct', 'study', 'radiology'],
                run: () => launchDicomModule(),
            },
            {
                id: 'cad.module.ceph',
                label: 'Open Cephalometry module',
                kind: 'navigation',
                keywords: ['ceph', 'cephalometry', 'tracing', 'landmarks'],
                run: () => launchCephModule(),
            },
            {
                id: 'cad.module.fab',
                label: 'Open Fabrication module',
                kind: 'navigation',
                keywords: ['fab', 'cam', 'manufacture', 'export'],
                run: () => launchFabModule(),
            },
            {
                id: 'cad.module.aligners',
                label: 'Open Aligners module',
                kind: 'navigation',
                keywords: ['aligners', 'ortho', 'staging'],
                run: () => launchAlignersModule(),
            },
            {
                id: 'cad.module.odontogram',
                label: 'Open Odontogram',
                kind: 'navigation',
                keywords: ['odontogram', 'teeth', 'chart'],
                run: () => launchOdontogramModule(),
            },
            {
                id: 'cad.toggle.properties',
                label: 'Toggle properties panel',
                kind: 'toggle',
                keywords: ['properties', 'inspector', 'panel'],
                run: () => setIsPropertiesPanelOpen((value) => !value),
            },
            {
                id: 'cad.toggle.asset-library',
                label: 'Toggle asset library',
                kind: 'toggle',
                keywords: ['asset', 'library', 'catalog'],
                run: () => setIsAssetLibraryOpen((value) => !value),
            },
        ]);
    }, [launchAlignersModule, launchCephModule, launchDicomModule, launchFabModule, launchGuideModule, launchImplantModule, launchOdontogramModule, launchSplintModule]);


    const persistCadUiModePreference = useCallback((mode: TlantiDbPreferences['cadUiMode']) => {
        setCadUiMode(mode);

        try {
            const state = loadTlantiDbState();
            saveTlantiDbState({
                ...state,
                preferences: {
                    ...state.preferences,
                    cadUiMode: mode,
                },
            });
        } catch {
            // Ignore persistence failures in browser-only fallbacks.
        }
    }, []);

    /**
     * V209/V211 — persist articulator state in the active TlantiCase.
     * Both the vendor preset and the influencing-FDI list end up under
     * `case.articulator`.
     */
    const persistArticulatorPatch = useCallback(
        (patch: Partial<NonNullable<TlantiCase['articulator']>>) => {
            const targetId = caseId ?? loadTlantiDbState().activeCaseId;
            if (!targetId) return;
            try {
                const state = loadTlantiDbState();
                const cases = state.cases.map((c) =>
                    c.id === targetId
                        ? { ...c, articulator: { ...(c.articulator ?? {}), ...patch } }
                        : c,
                );
                saveTlantiDbState({ ...state, cases });
            } catch {
                // Ignore persistence failures in browser-only fallbacks.
            }
        },
        [caseId],
    );

    const persistControlSchemePreference = useCallback((scheme: 'standard' | 'cad') => {
        setControlScheme(scheme);

        try {
            const state = loadTlantiDbState();
            saveTlantiDbState({
                ...state,
                preferences: {
                    ...state.preferences,
                    controlScheme: scheme,
                },
            });
        } catch {
            // Ignore persistence failures in browser-only fallbacks.
        }
    }, []);

    const caseAssetStats = useMemo(() => {
        const assets = activeCaseContext?.assets ?? [];
        return {
            total: assets.length,
            models: assets.filter((asset) => asset.category === 'model').length,
            dicom: assets.filter((asset) => asset.category === 'dicom').length,
            images: assets.filter((asset) => asset.category === 'image').length,
            documents: assets.filter((asset) => asset.category === 'document' || asset.category === 'report').length,
        };
    }, [activeCaseContext]);

    const activeModule = useMemo(() => resolveWorkspaceModuleDefinition(moduleId), [moduleId]);
    const canonicalModuleId = useMemo(() => resolveTlantiModuleDefinition(moduleId).id, [moduleId]);
    const activeDicomFile = useMemo(() => {
        if (selectedFile?.type === 'DICOM') {
            return selectedFile;
        }

        if (DICOM_CONTEXT_MODULES.has(canonicalModuleId)) {
            return files.find((file) => file.type === 'DICOM');
        }

        return undefined;
    }, [canonicalModuleId, files, selectedFile]);
    const [activeProductModuleId, setActiveProductModuleId] = useState<TlantiCadProductModuleId>(() => resolveCadProductModuleForRoute(moduleId).id);
    const activeProductModule = CAD_PRODUCT_MODULE_DEFINITIONS[activeProductModuleId];
    const activeModuleRoadmap = CAD_MODULE_ROADMAP_DEFINITIONS[activeProductModuleId];
    const productModules = useMemo(() => listCadProductModules(), []);
    const meshVaultImportUseCase = useMemo(() => new MeshVaultImportUseCase(new TauriMeshVault()), []);
    const activeMeshVaultCaseId = activeCaseContext?.id ?? caseId ?? 'workspace-unassigned';

    const bgClass = 'bg-black text-text-primary';
    const panelClass = 'bg-surface border border-border';
    const textMuted = 'text-text-secondary';
    const caseStatusItems = useMemo(() => buildCaseStatusItems({
        moduleId: activeModule.id,
        pipeline: activeCaseContext?.pipeline,
        storagePath: activeCaseContext?.storagePath,
        interopXmlPath: activeCaseContext?.lastInteropXmlPath,
    }), [activeCaseContext?.lastInteropXmlPath, activeCaseContext?.pipeline, activeCaseContext?.storagePath, activeModule.id]);

    const activeSmilePlaybook = useMemo(() => getSmileDesignPlaybookById(selectedSmilePlaybookId), [selectedSmilePlaybookId]);
    const assistantSteps = useMemo(() => activeSmilePlaybook.stages.map((stage) => stage.title), [activeSmilePlaybook]);
    const assistantStepIndex = smileCurrentStageByPlaybook[selectedSmilePlaybookId] ?? 0;
    const handledModuleLaunchRef = useRef<string | null>(null);

    const functionalSelectionItems = useMemo(() => [
        { key: 'antagonists' as const, label: 'Distancia antagonistas', activeColor: '#125A2C' },
        { key: 'truSmile' as const, label: 'TruSmile', activeColor: '#3163B7' },
        { key: 'colorTexture' as const, label: 'Color / Textura', activeColor: '#FDBD83' },
        { key: 'cutView' as const, label: 'Vista de corte', activeColor: '#B01825' },
        { key: 'smileView' as const, label: 'Vista sonrisa', activeColor: '#9f00a7' },
        { key: 'cloud' as const, label: 'Cloud', activeColor: '#5b9bf6' },
    ], []);

    useEffect(() => {
        if (!selectedId) setSelectedMesh(null);
    }, [selectedId]);

    useEffect(() => {
        if (!activeDicomFile || selectedId === activeDicomFile.id) {
            return;
        }

        if (DICOM_CONTEXT_MODULES.has(canonicalModuleId)) {
            setSelectedId(activeDicomFile.id);
        }
    }, [activeDicomFile, canonicalModuleId, selectedId]);

    useEffect(() => {
        if (!viewport.isCompact) {
            setIsFileManagerOpen(false);
            setIsPropertiesPanelOpen(false);
        }

        if (!selectedId) {
            setIsPropertiesPanelOpen(false);
        }
    }, [viewport.isCompact, selectedId]);

    useEffect(() => {
        return () => {
            if (moduleLoaderTimerRef.current) {
                window.clearTimeout(moduleLoaderTimerRef.current);
            }
        };
    }, []);

    useEffect(() => {
        setActiveProductModuleId(resolveCadProductModuleForRoute(moduleId).id);
    }, [moduleId]);

    useEffect(() => {
        if (!moduleId || handledModuleLaunchRef.current === moduleId) {
            return;
        }

        handledModuleLaunchRef.current = moduleId;
        // V143 — full reset of module-scoped panels before activating the new flow.
        closeClinicalModulePanels();

        const launcher = cadModuleLaunchers.get(moduleId);
        if (launcher) {
            launcher();
        } else {
            // Unknown module → restore the default CAD wizard rather than
            // leaving the canvas headless.
            setIsCadWizardOpen(true);
        }
    }, [cadModuleLaunchers, closeClinicalModulePanels, moduleId]);

    useEffect(() => {
        if (activeTool !== 'MEASURE') {
            setMeasurePoints([]);
        }
    }, [activeTool]);

    const deleteFile = useCallback((id: string) => {
        setFiles(prev => prev.filter(f => f.id !== id));
        setSelectedId(prev => prev === id ? null : prev);
    }, []);

    const setView = useCallback((view: 'TOP' | 'BOTTOM' | 'FRONT' | 'BACK' | 'LEFT' | 'RIGHT') => {
        if (!orbitRef.current) return;
        const controls = orbitRef.current;
        const distance = controls.object.position.distanceTo(controls.target);
        const target = controls.target.clone();

        const pos = new THREE.Vector3();
        switch (view) {
            case 'TOP': pos.set(0, distance, 0); break;
            case 'BOTTOM': pos.set(0, -distance, 0); break;
            case 'FRONT': pos.set(0, 0, distance); break;
            case 'BACK': pos.set(0, 0, -distance); break;
            case 'RIGHT': pos.set(distance, 0, 0); break;
            case 'LEFT': pos.set(-distance, 0, 0); break;
        }

        controls.object.position.copy(target.add(pos));
        controls.update();
    }, []);

    const addCustomView = useCallback(() => {
        if (!orbitRef.current) {
            return;
        }

        const controls = orbitRef.current;
        const nextView: SavedViewPreset = {
            id: uuidv4(),
            name: `Vista ${savedViews.length + 1}`,
            position: [controls.object.position.x, controls.object.position.y, controls.object.position.z],
            target: [controls.target.x, controls.target.y, controls.target.z],
        };

        setSavedViews((prev) => [...prev.slice(-5), nextView]);
        toast(`${nextView.name} guardada.`, 'success');
    }, [savedViews.length, toast]);

    const applySavedView = useCallback((view: SavedViewPreset) => {
        if (!orbitRef.current) {
            return;
        }

        orbitRef.current.object.position.set(...view.position);
        orbitRef.current.target.set(...view.target);
        orbitRef.current.update();
    }, []);

    const toggleFunctionalSelection = useCallback((key: FunctionalSelectionKey) => {
        setFunctionalSelections((prev) => ({
            ...prev,
            [key]: !prev[key],
        }));

        if (key === 'cutView') {
            setActiveTool((prev) => (prev === 'CLIP' ? 'SELECT' : 'CLIP'));
        }

        if (key === 'smileView') {
            setIsOdontogramOpen((prev) => !prev);
        }

        if (key === 'cloud') {
            toast('Servicios en la nube listos para revisión de caso.', 'info');
        }
    }, [toast]);

    const focusSelection = useCallback((id?: string) => {
        if (!orbitRef.current) return;
        let center = new THREE.Vector3(0, 0, 0);
        let radius = 5;
        const targetId = id || selectedId;

        if (targetId === selectedId && selectedMesh) {
            const box = new THREE.Box3().setFromObject(selectedMesh);
            box.getCenter(center);
            const size = box.getSize(new THREE.Vector3());
            radius = Math.max(size.x, size.y, size.z);
        } else if (files.length > 0) {
            radius = 20; // Default view if no specific selection
        }

        const controls = orbitRef.current;
        const direction = controls.object.position.clone().sub(controls.target).normalize();
        const dist = radius * 2.0;

        controls.target.copy(center);
        controls.object.position.copy(center.clone().add(direction.multiplyScalar(dist)));
        controls.update();
    }, [selectedMesh, files, selectedId]);

    // Auto-focus on selection change
    useEffect(() => {
        if (selectedId && files.length > 0) {
            // Small delay to ensure mesh is mounted and ref is set
            const timer = setTimeout(() => focusSelection(selectedId), 200);
            return () => clearTimeout(timer);
        }
    }, [selectedId, files.length, focusSelection]);

    const handleCenterView = useCallback((point: THREE.Vector3) => {
        if (orbitRef.current) {
            orbitRef.current.target.copy(point);
            orbitRef.current.update();
        }
    }, []);

    const updateFileProperty = useCallback((id: string, prop: Partial<FileData>) => {
        setFiles(prev => prev.map(f => f.id === id ? { ...f, ...prop } : f));
    }, []);

    const [zoomTrigger, setZoomTrigger] = useState(0);

    const moveFileUp = useCallback((id: string) => {
        setFiles(prev => {
            const index = prev.findIndex(f => f.id === id);
            if (index <= 0) return prev;
            const newFiles = [...prev];
            [newFiles[index - 1], newFiles[index]] = [newFiles[index], newFiles[index - 1]];
            return newFiles;
        });
    }, []);

    const moveFileDown = useCallback((id: string) => {
        setFiles(prev => {
            const index = prev.findIndex(f => f.id === id);
            if (index === -1 || index === prev.length - 1) return prev;
            const newFiles = [...prev];
            [newFiles[index + 1], newFiles[index]] = [newFiles[index], newFiles[index + 1]];
            return newFiles;
        });
    }, []);

    const handleObjectClick = useCallback((id: string, point: THREE.Vector3, event: MouseEvent) => {
        // Hotkey Logic from ExoCAD
        if (event.ctrlKey && event.shiftKey) {
            // No op or specific advanced
        } else if (event.ctrlKey) {
            // CTRL + Click -> Hide Object
            updateFileProperty(id, { visible: false });
            return;
        } else if (event.shiftKey) {
            // SHIFT + Click -> Toggle Transparency
            const f = files.find(file => file.id === id);
            if (f) updateFileProperty(id, { opacity: f.opacity < 0.9 ? 1.0 : 0.5 });
            return;
        }

        if (activeTool === 'MEASURE') {
            setMeasurePoints(prev => [...prev, point]);
        } else {
            setSelectedId(id);
        }
    }, [activeTool, files]);



    // Renaming logic
    const handleRenameStart = (id: string, currentName: string) => {
        setRenamingId(id);
        setRenameValue(currentName);
    };

    const handleRenameSave = () => {
        if (renamingId) {
            updateFileProperty(renamingId, { name: renameValue });
            setRenamingId(null);
        }
    };

    const exportSTL = useCallback(async (id: string) => {
        if (!selectedMesh || selectedId !== id) return;
        const { STLExporter } = await import('three-stdlib');
        const exporter = new STLExporter();
        const result = exporter.parse(selectedMesh);
        const blob = new Blob([result], { type: 'application/octet-stream' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = `model_${id.substring(0, 6)}.stl`;
        link.click();
    }, [selectedId, selectedMesh]);

    // Global Keyboard Shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (renamingId) return;
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

            const ctrl = e.ctrlKey || e.metaKey;

            // Global Tools
            if (ctrl && e.code === 'KeyR') {
                e.preventDefault();
                setActiveTool('MEASURE');
                return;
            }
            if (ctrl && e.code === 'KeyS') {
                e.preventDefault();
                if (selectedId) void exportSTL(selectedId);
                else alert("Select an object to save/export.");
                return;
            }
            if (ctrl && e.code === 'KeyZ') {
                e.preventDefault();
                if (e.shiftKey) {
                    redo();
                } else {
                    undo();
                }
                return;
            }
            if (ctrl && e.code === 'KeyY') {
                e.preventDefault();
                redo();
                return;
            }

            // Group Selectors (Heuristics based on name)
            const toggleGroup = (keyword: string) => {
                setFiles(prev => prev.map(f => {
                    if (f.name.toLowerCase().includes(keyword.toLowerCase())) {
                        return { ...f, visible: !f.visible };
                    }
                    return f;
                }));
            };

            if (!ctrl && !e.shiftKey) {
                switch (e.code) {
                    case 'KeyA': toggleGroup('antagonist'); break;
                    case 'KeyS': toggleGroup('scan'); break;
                    case 'KeyG': toggleGroup('gingiva'); break;
                    case 'KeyW': toggleGroup('wax'); break;
                    case 'KeyX': toggleGroup('upper'); break; // Maxilla
                    case 'KeyN': toggleGroup('lower'); break; // Mandible
                    case 'KeyD': toggleGroup('dicom'); break;
                }
            }

            switch (e.key.toLowerCase()) {
                case 'q': setActiveTool('SELECT'); break;
                case 'w': setActiveTool('MOVE'); break;
                case 'e': setActiveTool('ROTATE'); break;
                case 'r': if (!ctrl) setActiveTool('SCALE'); break;
                case 'escape':
                    setSelectedId(null);
                    setContextMenu(null);
                    setMeasurePoints([]);
                    setShortcutsOpen(false);
                    break;
                case 'delete':
                case 'backspace':
                    if (selectedId) deleteFile(selectedId);
                    break;
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [selectedId, deleteFile, exportSTL, setView, focusSelection, renamingId, files]);

    const notifyDicomWarnings = useCallback((importedFiles: FileData[]) => {
        const warnings = importedFiles
            .filter((file) => file.type === 'DICOM')
            .flatMap((file) => file.dicomMetadata?.warnings ?? []);

        warnings.forEach((warning) => {
            toast(warning, 'info');
        });
    }, [toast]);

    const updateFileMetadata = useCallback((id: string, metadata: MeshMetadata) => {
        setFiles(prev => prev.map(f => f.id === id ? { ...f, metadata } : f));
    }, []);

    const updateDicomMetadata = useCallback((id: string, metadata: DicomMetadata) => {
        setFiles(prev => prev.map(f => f.id === id ? { ...f, dicomMetadata: metadata } : f));
    }, []);

    const focusFirstDicomFile = useCallback((preferredFiles?: FileData[]) => {
        const candidate = preferredFiles?.find((file) => file.type === 'DICOM')
            ?? files.find((file) => file.type === 'DICOM');

        if (!candidate) {
            return false;
        }

        setSelectedId(candidate.id);
        return true;
    }, [files]);

    const triggerUpload = () => fileInputRef.current?.click();
    const triggerDirectoryUpload = () => directoryInputRef.current?.click();

    const appendImportedFiles = useCallback((
        importedFiles: FileData[],
        options?: {
            successMessage?: string;
            openFileManager?: boolean;
        },
    ) => {
        if (!importedFiles.length) {
            return false;
        }

        setFiles(prev => [...prev, ...importedFiles]);
        notifyDicomWarnings(importedFiles);
        setSelectedId(importedFiles[importedFiles.length - 1]?.id ?? null);
        setOpenSections(prev => ({ ...prev, layers: true }));
        setSidebarOpen(true);

        if (options?.openFileManager) {
            setIsFileManagerOpen(true);
        }

        if (options?.successMessage) {
            toast(options.successMessage, 'success');
        }

        return true;
    }, [notifyDicomWarnings, setFiles, toast]);

    const importPathBackedAsset = useCallback(async (
        sourcePath: string,
        options?: {
            displayName?: string;
            kind?: CadCoreAssetKind;
            role?: string;
            semanticUsage?: FileData['semanticUsage'];
            semanticTags?: string[];
            sourceRoot?: string;
            sourceRelativePath?: string;
            opacity?: number;
            scale?: [number, number, number];
        },
    ): Promise<PathBackedImportResult | null> => {
        const kind = options?.kind ?? inferMeshVaultKindFromPath(sourcePath);
        if (!kind) {
            return null;
        }

        const result = await meshVaultImportUseCase.execute({
            caseId: activeMeshVaultCaseId,
            sourcePath,
            kind,
            moduleId: activeProductModule.id as CadCoreModuleId,
            role: options?.role ?? 'mesh-vault',
            displayName: options?.displayName,
            metadata: {
                importMode: 'mesh-vault',
                source: 'desktop-path',
            },
        });

        let url: string | undefined;
        if (result.file.meshVault?.storagePath) {
            try {
                const { convertFileSrc } = await import('@tauri-apps/api/core');
                url = convertFileSrc(result.file.meshVault.storagePath);
            } catch {
                url = undefined;
            }
        }

        return {
            ...result,
            meshVaultStatus: result.file.meshVault?.storagePath ? 'completed' : 'queued',
            file: {
                ...result.file,
                url,
                semanticUsage: options?.semanticUsage,
                semanticTags: options?.semanticTags,
                sourceRoot: options?.sourceRoot,
                sourceRelativePath: options?.sourceRelativePath,
                opacity: options?.opacity ?? result.file.opacity,
                scale: options?.scale ?? result.file.scale,
            },
        };
    }, [activeMeshVaultCaseId, activeProductModule.id, meshVaultImportUseCase]);

    const collectAbsoluteImportEntries = useCallback(async (paths: string[]): Promise<FileUploadEntry[]> => {
        const dedupedEntries = new Map<string, FileUploadEntry>();
        const scannedDicomDirectories = new Set<string>();
        const [{ readDir, readFile }, { join }] = await Promise.all([
            import('@tauri-apps/plugin-fs'),
            import('@tauri-apps/api/path'),
        ]);

        const addEntry = async (absolutePath: string, relativePath?: string) => {
            if (dedupedEntries.has(absolutePath)) {
                return;
            }

            const bytes = await readFile(absolutePath);
            dedupedEntries.set(absolutePath, {
                file: new File([bytes], getPathBasename(absolutePath), { type: 'application/octet-stream' }),
                sourcePath: absolutePath,
                relativePath,
            });
        };

        for (const absolutePath of paths) {
            if (ZIP_FILE_PATTERN.test(absolutePath) || !DICOM_FILE_PATTERN.test(absolutePath)) {
                await addEntry(absolutePath);
                continue;
            }

            const directoryPath = getPathDirectory(absolutePath);
            if (!directoryPath) {
                await addEntry(absolutePath);
                continue;
            }

            if (scannedDicomDirectories.has(directoryPath)) {
                await addEntry(absolutePath, `${getLastDirectoryName(directoryPath)}/${getPathBasename(absolutePath)}`);
                continue;
            }

            scannedDicomDirectories.add(directoryPath);
            const siblings = await readDir(directoryPath);
            const dicomSiblingFiles = siblings.filter((entry) => entry.isFile && DICOM_FILE_PATTERN.test(entry.name));

            if (dicomSiblingFiles.length <= 1) {
                await addEntry(absolutePath, `${getLastDirectoryName(directoryPath)}/${getPathBasename(absolutePath)}`);
                continue;
            }

            for (const sibling of dicomSiblingFiles) {
                const siblingAbsolutePath = await join(directoryPath, sibling.name);
                await addEntry(siblingAbsolutePath, `${getLastDirectoryName(directoryPath)}/${sibling.name}`);
            }
        }

        return Array.from(dedupedEntries.values());
    }, []);

    const parseEntriesForBrowserCadImport = useCallback(async (entries: FileUploadEntry[]) => {
        const zipEntries = entries.filter((entry) => ZIP_FILE_PATTERN.test(entry.file.name));
        const regularEntries = entries.filter((entry) => !ZIP_FILE_PATTERN.test(entry.file.name));
        const importedFiles: FileData[] = [];

        if (regularEntries.length) {
            importedFiles.push(...await handleDirectoryUpload(regularEntries));
        }

        for (const zipEntry of zipEntries) {
            importedFiles.push(...await handleFileUpload(zipEntry.file, { sourcePath: zipEntry.sourcePath }));
        }

        return importedFiles;
    }, []);

    const importEntriesIntoCad = useCallback(async (entries: FileUploadEntry[], options?: { successMessage?: string; openFileManager?: boolean }) => {
        const importedFiles = await parseEntriesForBrowserCadImport(entries);

        if (!appendImportedFiles(importedFiles, options)) {
            return null;
        }

        if (DICOM_CONTEXT_MODULES.has(canonicalModuleId)) {
            focusFirstDicomFile(importedFiles);
        }

        return importedFiles;
    }, [appendImportedFiles, canonicalModuleId, focusFirstDicomFile, parseEntriesForBrowserCadImport]);

    const onFileChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            setLoading(true);
            try {
                const importedFiles: FileData[] = [];
                for (let i = 0; i < e.target.files.length; i++) {
                    const uploaded = await handleFileUpload(e.target.files[i]);
                    importedFiles.push(...uploaded);
                }

                if (!appendImportedFiles(importedFiles)) {
                    toast('The selected files could not be imported into CAD.', 'error');
                }
            } finally {
                setLoading(false);
                e.target.value = '';
            }
        }
    }, [appendImportedFiles, toast]);

    const onDirectoryChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            setLoading(true);
            try {
                const importedFiles = await handleDirectoryUpload(
                    Array.from(e.target.files).map((file) => ({
                        file,
                        relativePath: file.webkitRelativePath || file.name,
                    })),
                );

                if (!appendImportedFiles(importedFiles, {
                    successMessage: `${importedFiles.length} item(s) imported from folder.`,
                    openFileManager: true,
                })) {
                    toast('No supported DICOM, model or image files were found in the selected folder.', 'error');
                }
            } finally {
                setLoading(false);
                e.target.value = '';
            }
        }
    }, [appendImportedFiles, toast]);

    const importFilesFromAbsolutePaths = useCallback(async (paths: string[]) => {
        if (!paths.length) {
            return;
        }

        setLoading(true);
        try {
            if (isTauriRuntime()) {
                const importedFiles: FileData[] = [];
                const fallbackPaths: string[] = [];
                const meshVaultSummary = { queued: 0, completed: 0, fallback: 0 };

                for (const sourcePath of Array.from(new Set(paths))) {
                    if (!PATH_BACKED_MESH_FILE_PATTERN.test(sourcePath) || ZIP_FILE_PATTERN.test(sourcePath)) {
                        fallbackPaths.push(sourcePath);
                        continue;
                    }

                    const imported = await importPathBackedAsset(sourcePath, {
                        displayName: getPathBasename(sourcePath),
                    });
                    if (imported?.file) {
                        importedFiles.push(imported.file);
                        meshVaultSummary[imported.meshVaultStatus] += 1;
                    }
                }

                if (fallbackPaths.length) {
                    const fallbackEntries = await collectAbsoluteImportEntries(fallbackPaths);
                    const fallbackImported = await parseEntriesForBrowserCadImport(fallbackEntries);
                    importedFiles.push(...fallbackImported);
                    meshVaultSummary.fallback += fallbackImported.length;
                }

                if (!appendImportedFiles(importedFiles, {
                    successMessage: formatMeshVaultImportMessage(meshVaultSummary),
                    openFileManager: true,
                })) {
                    toast('The selected files could not be imported into CAD.', 'error');
                    return;
                }

                if (DICOM_CONTEXT_MODULES.has(canonicalModuleId)) {
                    focusFirstDicomFile(importedFiles);
                }
                return;
            }

            const entries = await collectAbsoluteImportEntries(paths);
            const importedFiles = await importEntriesIntoCad(entries, {
                successMessage: `${entries.length} file(s) imported into CAD.`,
            });

            if (!importedFiles) {
                toast('The selected files could not be imported into CAD.', 'error');
                return;
            }
        } catch (error) {
            console.error(error);
            toast('Desktop import failed. Try again or use the browser picker.', 'error');
        } finally {
            setLoading(false);
        }
    }, [appendImportedFiles, canonicalModuleId, collectAbsoluteImportEntries, focusFirstDicomFile, importEntriesIntoCad, importPathBackedAsset, parseEntriesForBrowserCadImport, toast]);

    const importDirectoryFromAbsolutePath = useCallback(async (directoryPath: string) => {
        setLoading(true);
        try {
            if (isTauriRuntime()) {
                const imported = await importPathBackedAsset(directoryPath, {
                    displayName: getLastDirectoryName(directoryPath),
                    kind: 'dicom-series',
                    role: 'dicom-series',
                });
                if (!appendImportedFiles(imported?.file ? [imported.file] : [], {
                    successMessage: imported?.file
                        ? formatMeshVaultImportMessage({
                            queued: imported.meshVaultStatus === 'queued' ? 1 : 0,
                            completed: imported.meshVaultStatus === 'completed' ? 1 : 0,
                            fallback: 0,
                        })
                        : undefined,
                    openFileManager: true,
                })) {
                    toast('No supported DICOM, model or image files were found in the selected folder.', 'error');
                }
                return;
            }

            const [{ readDir, readFile }, { join }] = await Promise.all([
                import('@tauri-apps/plugin-fs'),
                import('@tauri-apps/api/path'),
            ]);

            const entries: Array<{ file: File; sourcePath: string; relativePath: string }> = [];

            const walkDirectory = async (absoluteDir: string, relativeDir = ''): Promise<void> => {
                const dirEntries = await readDir(absoluteDir);

                for (const entry of dirEntries) {
                    const absoluteEntry = await join(absoluteDir, entry.name);
                    const relativeEntry = relativeDir ? `${relativeDir}/${entry.name}` : entry.name;

                    if (entry.isDirectory) {
                        await walkDirectory(absoluteEntry, relativeEntry);
                        continue;
                    }

                    if (!entry.isFile) {
                        continue;
                    }

                    const bytes = await readFile(absoluteEntry);
                    entries.push({
                        file: new File([bytes], entry.name, { type: 'application/octet-stream' }),
                        sourcePath: absoluteEntry,
                        relativePath: relativeEntry,
                    });
                }
            };

            await walkDirectory(directoryPath);
            const importedFiles = await importEntriesIntoCad(entries, {
                successMessage: `${entries.length} item(s) imported from folder.`,
                openFileManager: true,
            });
            if (!importedFiles) {
                toast('No supported DICOM, model or image files were found in the selected folder.', 'error');
            }
        } catch (error) {
            console.error(error);
            toast('Desktop folder import failed. Try again or use the browser picker.', 'error');
        } finally {
            setLoading(false);
        }
    }, [appendImportedFiles, importEntriesIntoCad, importPathBackedAsset, toast]);

    useEffect(() => {
        const pendingImport = consumePendingCadImport();
        if (!pendingImport) {
            return;
        }

        void (async () => {
            if (pendingImport.directoryPath) {
                await importDirectoryFromAbsolutePath(pendingImport.directoryPath);
                return;
            }

            if (pendingImport.paths?.length) {
                await importFilesFromAbsolutePaths(pendingImport.paths);
                return;
            }

            if (pendingImport.files?.length) {
                setLoading(true);
                try {
                    const importedFiles: FileData[] = [];
                    for (const file of pendingImport.files) {
                        const uploaded = await handleFileUpload(file);
                        importedFiles.push(...uploaded);
                    }

                    if (appendImportedFiles(importedFiles, { openFileManager: true })) {
                        if (DICOM_CONTEXT_MODULES.has(canonicalModuleId)) {
                            focusFirstDicomFile(importedFiles);
                        }
                        toast('DICOM study loaded from launcher.', 'success');
                    }
                } catch (error) {
                    console.error(error);
                    toast('Could not import the pending DICOM study.', 'error');
                } finally {
                    setLoading(false);
                }
            }
        })();
    }, [appendImportedFiles, canonicalModuleId, focusFirstDicomFile, importDirectoryFromAbsolutePath, importFilesFromAbsolutePaths, toast]);

    const handleImportAction = useCallback(async () => {
        const preferDirectoryImport = moduleId === 'dicom' || activeDicomFile?.type === 'DICOM';

        if (isTauriRuntime()) {
            try {
                const { open } = await import('@tauri-apps/plugin-dialog');

                if (preferDirectoryImport) {
                    const directoryResult = await open({
                        multiple: false,
                        directory: true,
                        title: 'Select a DICOM folder',
                    });

                    if (typeof directoryResult === 'string') {
                        await importDirectoryFromAbsolutePath(directoryResult);
                        return;
                    }
                }

                const result = await open({
                    multiple: true,
                    directory: false,
                    filters: [
                        { name: 'CAD and imaging', extensions: ['zip', 'dcm', 'dicom', 'ima', 'jpg', 'jpeg', 'png', 'obj', 'stl', 'ply', 'glb', 'gltf'] },
                    ],
                });

                if (!result) {
                    return;
                }

                const paths = (Array.isArray(result) ? result : [result]).filter((item): item is string => typeof item === 'string');
                await importFilesFromAbsolutePaths(paths);
                return;
            } catch (error) {
                console.error(error);
                toast('Desktop file picker unavailable. Falling back to the browser picker.', 'info');
            }
        }

        if (preferDirectoryImport && isTauriRuntime()) {
            triggerDirectoryUpload();
            return;
        }

        triggerUpload();
    }, [activeDicomFile?.type, importDirectoryFromAbsolutePath, importFilesFromAbsolutePaths, moduleId, toast]);

    const alignOverlayFile = useCallback((fileId: string, alignment: 'front' | 'plane') => {
        setFiles((prev) => prev.map((file) => {
            if (file.id !== fileId) {
                return file;
            }

            return {
                ...file,
                position: [0, 0, 0],
                rotation: alignment === 'plane' ? [-Math.PI / 2, 0, 0] : [0, 0, 0],
                scale: alignment === 'plane' ? [1.15, 1.15, 1.15] : [1.4, 1.4, 1.4],
                opacity: 0.55,
            };
        }));
        setActiveOverlayAlignment(alignment);
    }, [setFiles]);

    const buildSceneArchive = useCallback(async () => {
        const { default: JSZip } = await import('jszip');
        const zip = new JSZip();

        files.forEach(file => {
            if (file.buffer) {
                let ext = '';
                if (file.type === 'MODEL' && !file.name.includes('.')) ext = '.stl';
                else if (file.type === 'DICOM' && !file.name.includes('.')) ext = '.dcm';
                else if (file.type === 'IMAGE' && !file.name.includes('.')) ext = '.jpg';

                const filename = file.name.includes('.') ? file.name : `${file.name}${ext}`;
                zip.file(filename, file.buffer);
            } else if (file.buffers) {
                const folder = zip.folder(file.name);
                file.buffers.forEach((buf, i) => {
                    folder?.file(`slice_${i}.dcm`, buf);
                });
            }
        });

        const sceneMeta = files.map(f => ({
            id: f.id,
            name: f.name,
            type: f.type,
            position: f.position,
            rotation: f.rotation,
            scale: f.scale,
            visible: f.visible,
            opacity: f.opacity,
            wireframe: f.wireframe,
            dicomAdjustments: f.dicomAdjustments,
            windowCenter: f.windowCenter,
            windowWidth: f.windowWidth,
            sliceIndex: f.sliceIndex
        }));

        zip.file('scene.json', JSON.stringify(sceneMeta, null, 2));
        return zip;
    }, [files]);

    const exportSceneToZip = useCallback(async () => {
        if (!files.length) {
            toast('Import or create CAD content before exporting.', 'info');
            return;
        }

        setLoading(true);
        try {
            const zip = await buildSceneArchive();
            const content = await zip.generateAsync({ type: 'blob' });
            const link = document.createElement('a');
            link.href = URL.createObjectURL(content);
            link.download = `${activeCaseContext?.caseNumber ?? 'tlanti4cad_scene'}.zip`;
            link.click();
            toast('Scene archive exported successfully.', 'success');
        } catch (error) {
            console.error('Export failed', error);
            toast('Failed to export the current scene.', 'error');
        }
        setLoading(false);
    }, [activeCaseContext?.caseNumber, buildSceneArchive, files.length, toast]);

    const handleExportAction = useCallback(async () => {
        if (!files.length) {
            toast('Import or create CAD content before exporting.', 'info');
            return;
        }

        if (isTauriRuntime()) {
            setLoading(true);
            try {
                const { save } = await import('@tauri-apps/plugin-dialog');
                const { writeFile } = await import('@tauri-apps/plugin-fs');
                const targetPath = await save({
                    defaultPath: `${activeCaseContext?.caseNumber ?? 'tlanti4cad_scene'}.zip`,
                    filters: [{ name: 'ZIP archive', extensions: ['zip'] }],
                });

                if (!targetPath || Array.isArray(targetPath)) {
                    return;
                }

                const zip = await buildSceneArchive();
                const bytes = await zip.generateAsync({ type: 'uint8array' });
                await writeFile(targetPath, bytes);
                toast(`Scene exported to ${targetPath.split(/[/\\]/).pop()}.`, 'success');
                return;
            } catch (error) {
                console.error(error);
                toast('Desktop export failed. Falling back to browser download.', 'error');
            } finally {
                setLoading(false);
            }
        }

        await exportSceneToZip();
    }, [activeCaseContext?.caseNumber, buildSceneArchive, exportSceneToZip, files.length, toast]);

    const insertLibraryAssetIntoCad = useCallback(async (asset: PublicAssetLibraryItem, semantic = false, preferredAlignment?: 'front' | 'plane') => {
        if (!asset.absolutePath || !isTauriRuntime()) {
            toast('Asset library is available in desktop runtime only.', 'info');
            return;
        }

        try {
            const imported = await importPathBackedAsset(asset.absolutePath, {
                displayName: asset.name,
                role: asset.semanticUsage ?? 'library-asset',
                semanticUsage: asset.semanticUsage,
                semanticTags: asset.autoTags,
                sourceRoot: asset.root,
                sourceRelativePath: asset.relativePath,
                opacity: semantic && asset.semanticUsage === 'overlay' ? 0.55 : undefined,
                scale: semantic && asset.semanticUsage === 'overlay' ? [1.4, 1.4, 1.4] : undefined,
            });

            if (!imported?.file) {
                toast('This library asset is indexed, but CAD cannot place it yet.', 'info');
                return;
            }

            const enriched = [imported.file];

            setFiles((prev) => [...prev, ...enriched]);
            setSelectedId(enriched[0]?.id ?? null);
            setIsFileManagerOpen(true);
            if (semantic && asset.semanticUsage === 'overlay') {
                setActiveOverlayAsset(asset);
                setActiveOverlayFileId(enriched[0]?.id ?? null);
                if (enriched[0]?.id) {
                    alignOverlayFile(enriched[0].id, preferredAlignment ?? 'front');
                }
            }
            toast(
                `${asset.name}: ${formatMeshVaultImportMessage({
                    queued: imported.meshVaultStatus === 'queued' ? 1 : 0,
                    completed: imported.meshVaultStatus === 'completed' ? 1 : 0,
                    fallback: 0,
                })}`,
                'success',
            );
        } catch (error) {
            console.error(error);
            toast('Could not insert the selected library asset into CAD.', 'error');
        }
    }, [alignOverlayFile, importPathBackedAsset, toast]);

    const applySemanticAssetInCad = useCallback(async (asset: PublicAssetLibraryItem) => {
        const presetEffect = inferCadPresetEffect(asset);

        if (asset.semanticUsage === 'overlay') {
            await insertLibraryAssetIntoCad(asset, true, presetEffect.preferredOverlayAlignment);
            return;
        }

        if (asset.semanticUsage === 'preset') {
            setActiveCadPreset(asset);
            if (presetEffect.activeTool) setActiveTool(presetEffect.activeTool);
            if (presetEffect.gridVisible !== undefined) setGridVisible(presetEffect.gridVisible);
            if (presetEffect.transformSpace) setTransformSpace(presetEffect.transformSpace);
            if (presetEffect.controlScheme) setControlScheme(presetEffect.controlScheme);
            setSnapValue(presetEffect.snapValue ?? null);
            toast(presetEffect.summary, 'success');
            return;
        }

        if (asset.semanticUsage === 'dental-library') {
            setActiveDentalLibrary(asset);
            toast(`Dental library active: ${asset.label}.`, 'success');
            return;
        }

        toast('This asset does not expose a semantic action in CAD yet.', 'info');
    }, [insertLibraryAssetIntoCad, toast]);

    const handleCadAssetLibraryAction = useCallback(async (asset: PublicAssetLibraryItem, mode: PublicAssetLibraryActionMode) => {
        if (mode === 'cad') {
            await insertLibraryAssetIntoCad(asset);
            return;
        }

        if (mode === 'semantic') {
            await applySemanticAssetInCad(asset);
        }
    }, [applySemanticAssetInCad, insertLibraryAssetIntoCad]);

    const handleCadAssetLibraryBatchAction = useCallback(async (assets: PublicAssetLibraryItem[], mode: PublicAssetLibraryActionMode) => {
        for (const asset of assets) {
            if (mode === 'cad') {
                await insertLibraryAssetIntoCad(asset);
            }

            if (mode === 'semantic') {
                await applySemanticAssetInCad(asset);
            }
        }
    }, [applySemanticAssetInCad, insertLibraryAssetIntoCad]);

    const toggleVisibility = (id: string) => {
        const f = files.find(f => f.id === id);
        if (f) updateFileProperty(id, { visible: !f.visible });
    };
    const setOpacity = (id: string, opacity: number) => updateFileProperty(id, { opacity });
    const toggleWireframe = (id: string) => {
        const f = files.find(f => f.id === id);
        if (f) updateFileProperty(id, { wireframe: !f.wireframe });
    };

    const resetView = () => orbitRef.current?.reset();

    const handleZoom = (delta: number) => {
        if (orbitRef.current) {
            const camera = orbitRef.current.object;
            const target = orbitRef.current.target;
            const direction = new THREE.Vector3().subVectors(camera.position, target).normalize();
            const distance = camera.position.distanceTo(target);
            const newDistance = Math.max(0.1, distance * (1 + delta));
            camera.position.copy(target).add(direction.multiplyScalar(newDistance));
            orbitRef.current.update();
        }
    };

    const handlePan = (dx: number, dy: number) => {
        if (orbitRef.current) {
            const camera = orbitRef.current.object;
            const target = orbitRef.current.target;
            const right = new THREE.Vector3(1, 0, 0).applyQuaternion(camera.quaternion);
            const up = new THREE.Vector3(0, 1, 0).applyQuaternion(camera.quaternion);
            const distance = camera.position.distanceTo(target);
            const panSpeed = distance * 0.1 * navigationSensitivity.pan;

            const panVector = new THREE.Vector3()
                .addScaledVector(right, dx * panSpeed)
                .addScaledVector(up, dy * panSpeed);

            camera.position.add(panVector);
            target.add(panVector);
            orbitRef.current.update();
        }
    };

    const handleTransformChange = () => {
        if (selectedMesh && selectedId) {
            setFiles(prev => prev.map(f => {
                if (f.id === selectedId) {
                    return {
                        ...f,
                        position: [selectedMesh.position.x, selectedMesh.position.y, selectedMesh.position.z],
                        rotation: [selectedMesh.rotation.x, selectedMesh.rotation.y, selectedMesh.rotation.z],
                        scale: [selectedMesh.scale.x, selectedMesh.scale.y, selectedMesh.scale.z]
                    };
                }
                return f;
            }));
        }
    };

    const handleContextMenu = useCallback((event: React.MouseEvent | THREE.Event, type: 'object' | 'canvas', id?: string) => {
        const nativeEvent = 'nativeEvent' in event
            ? event.nativeEvent
            : (event as any).sourceEvent ?? (event as any).nativeEvent;

        nativeEvent?.preventDefault?.();
        const x = 'clientX' in event ? event.clientX : nativeEvent?.clientX ?? 0;
        const y = 'clientY' in event ? event.clientY : nativeEvent?.clientY ?? 0;
        setContextMenu({ x, y, type, targetId: id });
        if (id) setSelectedId(id);
    }, []);

    const isDicomWorkspaceOpen = selectedFile?.type === 'DICOM';

    useEffect(() => {
        if (!isDicomWorkspaceOpen) {
            return;
        }

        setIsFileManagerOpen(false);
        setIsAssetLibraryOpen(false);
        setIsGroupSelectorOpen(false);
        setIsCadGuideOpen(false);
        setIsSmileWorkflowOpen(false);
        setIsCopilotOpen(false);
        setIsPropertiesPanelOpen(false);
    }, [isDicomWorkspaceOpen]);

    const tools = useMemo(() => CAD_VIEWPORT_TOOL_MODE_IDS.map((id) => {
        const definition = resolveCadToolDefinition(id);
        return {
            id,
            label: CAD_TOOL_LABEL_BY_MODE[id] ?? definition?.label ?? id,
            icon: CAD_TOOL_ICON_BY_MODE[id],
            commandId: definition?.commandId ?? `cad.tool.${id.toLowerCase()}`,
        };
    }), []);

    const expertTools = useMemo(() => CAD_EXPERT_TOOL_MODE_IDS.map((id) => {
        const definition = resolveCadToolDefinition(id);
        return {
            id,
            label: CAD_TOOL_LABEL_BY_MODE[id] ?? definition?.label ?? id,
            icon: CAD_TOOL_ICON_BY_MODE[id],
            commandId: definition?.commandId ?? `cad.tool.${id.toLowerCase()}`,
        };
    }), []);

    const dockItems = [
        ...tools.map(t => ({
            icon: t.icon,
            label: t.label,
            active: String(activeTool) === t.id,
            onClick: () => setActiveTool(t.id as ToolMode)
        })),
        ...(isExpertMode ? expertTools.map(t => ({
            icon: t.icon,
            label: t.label,
            active: String(activeTool) === t.id,
            onClick: () => {
                if (!cadViewModel.hasModel && ['INSERTION', 'CROWN', 'CONNECTORS', 'THICKNESS', 'ARTICULATOR', 'ALIGN', 'EXPORT_PROD'].includes(t.id)) {
                    toast(cadViewModel.blockedModelReason, 'info');
                    void handleImportAction();
                    return;
                }

                setActiveTool(t.id as ToolMode);
            }
        })) : [])
    ];

    const toggleViewportFullscreen = useCallback(() => {
        if (typeof document === 'undefined') return;
        try {
            if (!document.fullscreenElement) {
                void document.documentElement.requestFullscreen();
            } else {
                void document.exitFullscreen();
            }
        } catch (error) {
            console.warn('[CadInterface] fullscreen toggle failed', error);
        }
    }, []);

    const clinicalDockItems = [
        ...tools
            .filter((tool) => ['SELECT', 'MOVE', 'ROTATE', 'SCALE', 'MEASURE'].includes(tool.id))
            .map((tool) => ({
                icon: tool.icon,
                label: tool.label,
                active: activeTool === tool.id,
                onClick: () => setActiveTool(tool.id as ToolMode),
            })),
        {
            icon: Maximize,
            label: 'Pantalla completa',
            active: false,
            onClick: toggleViewportFullscreen,
        },
    ];

    const applePaletteItems = tools
        .filter((tool) => ['SELECT', 'MOVE', 'ROTATE', 'SCALE', 'CLIP', 'MEASURE', 'SEGMENT'].includes(tool.id))
        .map((tool) => ({
            icon: tool.icon,
            label: tool.label,
            active: activeTool === tool.id,
            onClick: () => setActiveTool(tool.id as ToolMode),
        }));

    const isClinicalClean = cadUiMode === 'clinical-clean';
    const isApplePro = cadUiMode === 'apple-pro';
    const floatingDockItems = useMemo(() => {
        const sourceItems = isClinicalClean
            ? clinicalDockItems
            : viewport.isCompact && !isExpertMode
                ? dockItems.slice(0, 6)
                : dockItems;

        return sourceItems.map((item) => {
            const Icon = item.icon as React.ComponentType<{ className?: string }>;
            return {
                title: item.label,
                onClick: item.onClick,
                icon: <Icon className="size-5" />,
            };
        });
    }, [clinicalDockItems, dockItems, isClinicalClean, isExpertMode, viewport.isCompact]);

    const activeFunctionalItems = functionalSelectionItems.filter((item) => functionalSelections[item.key]);
    const shouldShowAdvancedSummary = isExpertMode && (activeFunctionalItems.length > 0 || savedViews.length > 0);
    const activeToolLabel = [...tools, ...expertTools].find((tool) => tool.id === activeTool)?.label ?? activeTool;
    const activeSelectionSummary = cadViewModel.activeSelectionLabel;
    const moduleToolset = DENTAL_CAD_PRODUCT_MODULE_TOOLSETS[activeProductModule.id]
        ?? DENTAL_CAD_MODULE_TOOLSETS[(activeModule.id as keyof typeof DENTAL_CAD_MODULE_TOOLSETS)]
        ?? DENTAL_CAD_MODULE_TOOLSETS.cad;

    const activeModuleAction = useMemo<DentalCadShellActionId | null>(() => {
        switch (activeModule.id) {
            case 'orthocad':
                if (isSmileWorkflowOpen) return 'smile-workflow';
                if (isOdontogramOpen) return 'odontogram';
                if (isCopilotOpen) return 'voice-copilot';
                if (activeTool === 'SCULPT') return 'sculpt';
                if (activeTool === 'MEASURE') return 'measure';
                return null;
            case 'dicom':
                if (isDicomModuleOpen) return 'dicom-import';
                if (selectedFile?.type === 'DICOM') return 'dicom-mpr';
                if (isPropertiesPanelOpen) return 'dicom-metadata';
                if (activeTool === 'MEASURE') return 'measure';
                return null;
            case 'partials':
                if (activeTool === 'SEGMENT') return 'segment';
                if (isCopilotOpen) return 'voice-copilot';
                return null;
            case 'ceph':
                if (selectedFile?.type === 'DICOM') return 'dicom-mpr';
                if (isPropertiesPanelOpen) return 'dicom-metadata';
                return 'measure';
            case 'fab':
                if (isFileManagerOpen) return 'layers-panel';
                return 'guide-export';
            case 'aligners':
                if (isAlignersModuleOpen) return 'odontogram';
                if (activeTool === 'SCULPT') return 'sculpt';
                return 'measure';
            case 'implant':
                if (isImplantModuleOpen) return 'implant-planning';
                if (activeTool === 'MEASURE') return 'implant-measure';
                return null;
            case 'guide':
                if (isSurgicalGuideModuleOpen) return 'guide-wizard';
                if (activeTool === 'MEASURE') return 'measure';
                return null;
            case 'splint':
                if (isSplintModuleOpen) return 'splint-workflow';
                if (activeTool === 'MEASURE') return 'measure';
                return null;
            case 'layers':
                if (isFileManagerOpen) return 'layers-panel';
                if (isGroupSelectorOpen) return 'groups-panel';
                return null;
            default:
                if (activeTool === 'SELECT') return 'select';
                if (activeTool === 'MOVE') return 'move';
                if (activeTool === 'ROTATE') return 'rotate';
                if (activeTool === 'SCALE') return 'scale';
                if (activeTool === 'MEASURE') return 'measure';
                if (activeTool === 'SCULPT') return 'sculpt';
                if (activeTool === 'SEGMENT') return 'segment';
                return null;
        }
    }, [activeModule.id, activeTool, isAlignersModuleOpen, isCopilotOpen, isDicomModuleOpen, isFileManagerOpen, isGroupSelectorOpen, isImplantModuleOpen, isOdontogramOpen, isPropertiesPanelOpen, isSurgicalGuideModuleOpen, isSmileWorkflowOpen, isSplintModuleOpen, selectedFile?.type]);

    const activeWorkflowPhase = useMemo<TlantiCadModuleWorkflowPhase>(() => {
        const phaseForCurrentTool = activeModuleAction
            ? activeModuleRoadmap.workflow.find((phase) => (phase.tools as readonly string[]).includes(activeModuleAction))
            : undefined;

        return phaseForCurrentTool ?? activeModuleRoadmap.workflow[0];
    }, [activeModuleAction, activeModuleRoadmap]);

    const runModuleAction = useCallback((actionId: DentalCadShellActionId) => {
        if (cadViewModel.isActionBlocked(actionId)) {
            toast(cadViewModel.blockedModelReason, 'info');
            void handleImportAction();
            return;
        }

        switch (actionId) {
            case 'select':
                setActiveTool('SELECT');
                break;
            case 'move':
                setActiveTool('MOVE');
                break;
            case 'rotate':
                setActiveTool('ROTATE');
                break;
            case 'scale':
                setActiveTool('SCALE');
                break;
            case 'measure':
            case 'implant-measure':
                setActiveTool('MEASURE');
                break;
            case 'sculpt':
                setActiveTool('SCULPT');
                break;
            case 'segment':
                setActiveTool('SEGMENT');
                setIsCopilotOpen(true);
                break;
            case 'margin':
                setActiveTool('MEASURE');
                toast('Margin tool staged: backend margin detection will persist as a clinical command.', 'info');
                break;
            case 'axis':
                setActiveTool('ROTATE');
                toast('Insertion axis mode active for the current dental module.', 'info');
                break;
            case 'contacts':
                setActiveTool('MEASURE');
                toast('Contact and occlusion analysis is queued through clinical jobs in the next backend cut.', 'info');
                break;
            case 'thickness':
                setActiveTool('MEASURE');
                toast('Thickness validation staged as a mesh analysis job.', 'info');
                break;
            case 'repair':
                setActiveTool('SCULPT');
                toast('Mesh repair will run through MeshLib/Tauri jobs; preview tool active.', 'info');
                break;
            case 'offset':
                setActiveTool('SCALE');
                toast('Offset/shell parameters staged for backend mesh job.', 'info');
                break;
            case 'trim':
                setActiveTool('BOOLEAN_CUT');
                break;
            case 'base':
                setIsAssetLibraryOpen(true);
                toast('Model base presets are available from the dental asset library.', 'info');
                break;
            case 'label':
                setActiveTool('SCULPT');
                toast('Label placement uses scene annotations until filesystem asset manifest is wired.', 'info');
                break;
            case 'boolean':
                setActiveTool('BOOLEAN_CUT');
                toast('Boolean operation staged for MeshLib backend job.', 'info');
                break;
            case 'implant-library':
                setIsAssetLibraryOpen(true);
                toast('Open local implant/platform library for this module.', 'info');
                break;
            case 'abutment-design':
                launchImplantModule();
                toast('Abutment workflow selected: emergence, screw channel and cement gap.', 'info');
                break;
            case 'bar-design':
                setActiveTool('SEGMENT');
                toast('Bar path workflow selected for partial denture design.', 'info');
                break;
            case 'telescope-fit':
                setActiveTool('MEASURE');
                toast('Telescope fit workflow selected: spacing/friction validation.', 'info');
                break;
            case 'manufacturing-export':
                void handleExportAction();
                break;
            case 'dicom-import':
                void handleImportAction();
                break;
            case 'dicom-mpr':
                if (selectedFile?.type !== 'DICOM') {
                    void handleImportAction();
                    toast('Importa una serie DICOM para abrir revisión radiológica.', 'info');
                } else {
                    toast('DICOM Viewer listo para este caso.', 'success');
                }
                break;
            case 'dicom-metadata':
                if (selectedFile) {
                    setIsPropertiesPanelOpen(true);
                }
                break;
            case 'smile-workflow':
                closeClinicalModulePanels('smile');
                setIsSmileWorkflowOpen(true);
                break;
            case 'smile-photos':
                setIsAssetLibraryOpen(true);
                toast('Abre fotos/referencias desde la asset library del caso.', 'info');
                break;
            case 'odontogram':
                launchOdontogramModule();
                break;
            case 'implant-planning':
                launchImplantModule();
                break;
            case 'guide-wizard':
                launchGuideModule();
                break;
            case 'splint-workflow':
            case 'splint-export':
                launchSplintModule();
                break;
            case 'guide-export':
                void handleExportAction();
                break;
            case 'layers-panel':
                setIsFileManagerOpen(true);
                break;
            case 'groups-panel':
                setIsGroupSelectorOpen(true);
                break;
            case 'voice-copilot':
                setIsCopilotOpen((prev) => !prev);
                break;
            case 'command-palette':
                openCommandPalette();
                break;
            default:
                break;
        }
    }, [cadViewModel, closeClinicalModulePanels, handleExportAction, handleImportAction, launchGuideModule, launchImplantModule, launchOdontogramModule, launchSplintModule, openCommandPalette, selectedFile, toast]);

    const selectProductModule = useCallback((nextModuleId: TlantiCadProductModuleId) => {
        if (cadViewModel.isProductModuleBlocked(nextModuleId)) {
            toast(cadViewModel.blockedModelReason, 'info');
            void handleImportAction();
            return;
        }

        setActiveProductModuleId(nextModuleId);
        closeClinicalModulePanels();

        switch (nextModuleId) {
            case 'tlanticad-implant':
            case 'tlanticad-abutment':
                launchImplantModule();
                break;
            case 'tlanticad-waxup':
                closeClinicalModulePanels('smile');
                setIsSmileWorkflowOpen(true);
                setActiveTool('SCULPT');
                break;
            case 'tlanticad-model':
                setIsAssetLibraryOpen(true);
                setActiveTool('SELECT');
                break;
            case 'tlanticad-bar':
                setActiveTool('SEGMENT');
                break;
            case 'tlanticad-bite-splint':
                launchSplintModule();
                break;
            case 'tlanticad-freeform':
                setActiveTool('SCULPT');
                break;
            case 'tlanticad-bridge':
            case 'tlanticad-telescope':
                setActiveTool('MEASURE');
                break;
            case 'tlanticad-crown':
            default:
                setActiveTool('SELECT');
                break;
        }
    }, [cadViewModel, closeClinicalModulePanels, handleImportAction, launchImplantModule, launchSplintModule, toast]);

    const cycleSnapValue = useCallback(() => {
        const snapSequence: Array<number | null> = [null, 0.25, 0.5, 1, 2];
        const currentIndex = snapSequence.findIndex((value) => value === snapValue);
        const nextIndex = currentIndex === -1 ? 1 : (currentIndex + 1) % snapSequence.length;
        setSnapValue(snapSequence[nextIndex]);
    }, [snapValue]);

    // Legacy local palette retired — global palette in App.tsx owns Cmd/Ctrl+K
    // and reads from `commandRegistry`. The CadCommandPanels overlay below is
    // kept as a no-op to preserve the prop contract until the panel is removed.
    const commandPaletteActions = useMemo<any[]>(() => [], []);



    const formatNumber = (num: number | undefined) => {
        if (num === undefined) return '-';
        return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
    };

    const measurementLines = useMemo(() => {
        const lines = [];
        for (let i = 0; i < measurePoints.length - 1; i += 2) {
            const start = measurePoints[i];
            const end = measurePoints[i + 1];
            const dist = start.distanceTo(end);
            const mid = start.clone().add(end).multiplyScalar(0.5);
            lines.push({ start, end, dist, mid, index: i });
        }
        return lines;
    }, [measurePoints]);

    const commandPanelsViewModel = useMemo(() => defineCadCommandPanelsViewModel({
        shortcutsOpen,
        onCloseShortcuts: () => setShortcutsOpen(false),
        themeMode,
        controlScheme,
        onControlSchemeChange: persistControlSchemePreference,
        commandPaletteOpen: false,
        onCloseCommandPalette: () => undefined,
        commandPaletteActions,
        activeProductModule,
        activeRoadmap: activeModuleRoadmap,
        activeWorkflowPhase,
    }), [activeModuleRoadmap, activeProductModule, activeWorkflowPhase, commandPaletteActions, controlScheme, persistControlSchemePreference, shortcutsOpen, themeMode]);

    const canvasSceneViewModel = useMemo(() => defineCanvasSceneViewModel({
        files,
        selectedId,
        selectedMesh,
        activeTool,
        themeMode,
        gridVisible,
        zoomTrigger,
        cropStart,
        cropEnd,
        cropPlanes,
        measurePoints,
        measurementLines,
        transformSpace,
        snapValue,
        navigationSensitivity,
        controlScheme,
        orbitRef,
        transformRef,
        onSelectedMeshChange: setSelectedMesh,
        onSetCropPlanes: setCropPlanes,
        onObjectClick: handleObjectClick,
        onCenterView: handleCenterView,
        onContextMenu: handleContextMenu,
        onMetadataLoaded: updateFileMetadata,
        onTransformChange: handleTransformChange,
    }), [
        activeTool,
        controlScheme,
        cropEnd,
        cropPlanes,
        cropStart,
        files,
        gridVisible,
        handleCenterView,
        handleContextMenu,
        handleObjectClick,
        handleTransformChange,
        measurePoints,
        measurementLines,
        navigationSensitivity,
        orbitRef,
        selectedId,
        selectedMesh,
        snapValue,
        themeMode,
        transformRef,
        transformSpace,
        updateFileMetadata,
        zoomTrigger,
    ]);

    // V117 — Compose the CadWizardSlot view-model from the active case so the
    // wizard panels receive real meshPath / caseFolderPath / teethPayload /
    // conditional-step flags instead of the V113 placeholders.
    const cadWizardViewModel = useMemo(() => {
        const teeth = activeCaseContext?.toothMap ?? {};
        const selectedTeeth = Object.entries(teeth)
            .filter(([, t]) => t?.selected)
            .map(([key, t]) => {
                const fdi = parseInt(key.replace('tooth-', ''), 10);
                return {
                    fdi,
                    state: t,
                };
            })
            .filter((entry) => Number.isFinite(entry.fdi));

        const teethPayload: MergeToothPayload[] = selectedTeeth.map(({ fdi, state }) => ({
            tooth: fdi,
            workTypeId: state.workTypeId,
            material: state.material,
            shade: state.shade,
            workTimeMinutes: state.workTimeMinutes,
        }));

        const firstTooth = selectedTeeth[0];
        const needsPreopWaxup = selectedTeeth.some((entry) => {
            const s = entry.state;
            return Boolean(
                s.additionalScans?.preOpModel ||
                    s.additionalScans?.waxup ||
                    s.usePreOpModel,
            );
        });
        const needsAbutment = selectedTeeth.some((entry) => {
            const s = entry.state;
            return (
                s.workTypeId === 'custom-abutment' ||
                s.implantMode === 'custom-abutment'
            );
        });

        // Find scan / mesh asset path for the active case. Asset roles use the
        // `TlantiCaseAssetRole` taxonomy (prep-scan, antagonist-scan, …); the
        // tooth-prep mesh is `prep-scan`, pre-op data lives in `pre-op-photo`,
        // and waxup is currently classified as `restoration-model` with a
        // 'waxup' tag/name match.
        const meshAsset = (activeCaseContext?.assets ?? []).find(
            (asset) => asset.role === 'prep-scan',
        );
        const preopAsset = (activeCaseContext?.assets ?? []).find(
            (asset) => asset.role === 'pre-op-photo',
        );
        const waxupAsset = (activeCaseContext?.assets ?? []).find(
            (asset) =>
                asset.role === 'restoration-model' && /waxup/i.test(asset.name ?? ''),
        );

        return {
            toothFdi: firstTooth?.fdi,
            material: firstTooth?.state.material ?? null,
            meshPath: meshAsset?.relativePath ?? null,
            preopPath: preopAsset?.relativePath ?? null,
            waxupPath: waxupAsset?.relativePath ?? null,
            caseFolderPath: activeCaseContext?.storagePath ?? null,
            caseId: activeCaseContext?.id ?? caseId ?? null,
            teethPayload,
            needsPreopWaxup,
            needsAbutment,
        };
    }, [activeCaseContext, caseId]);

    return (
        <div
            className={clsx("relative flex h-dvh w-full flex-col transition-colors duration-500", bgClass, viewport.isCompact ? 'overflow-auto' : 'overflow-hidden')}
            onPointerDown={(e) => {
                dragStartRef.current = { x: e.clientX, y: e.clientY };
                if (activeTool === 'CROP' && (e.target as HTMLElement).tagName === 'CANVAS') {
                    const rect = (e.target as HTMLElement).getBoundingClientRect();
                    const x = e.clientX - rect.left;
                    const y = e.clientY - rect.top;
                    setCropStart({ x, y });
                    setCropEnd({ x, y });
                }
            }}
            onPointerMove={(e) => {
                if (activeTool === 'CROP' && cropStart) {
                    const canvas = document.querySelector('canvas');
                    if (canvas) {
                        const rect = canvas.getBoundingClientRect();
                        const x = e.clientX - rect.left;
                        const y = e.clientY - rect.top;
                        setCropEnd({ x, y });
                    }
                }
            }}
            onPointerUp={(e) => {
                if (activeTool === 'CROP' && cropStart && cropEnd) {
                    setCropStart(null);
                    setCropEnd(null);
                }
            }}
            onContextMenu={(e) => {
                if ((e.target as HTMLElement).tagName === 'CANVAS') {
                    e.preventDefault();
                    // Check for drag
                    if (dragStartRef.current) {
                        const dx = e.clientX - dragStartRef.current.x;
                        const dy = e.clientY - dragStartRef.current.y;
                        if (Math.sqrt(dx * dx + dy * dy) > 5) return; // It was a drag, don't show menu
                    }
                    setContextMenu({ x: e.clientX, y: e.clientY, type: 'canvas' });
                }
            }}
            onClick={() => setContextMenu(null)}
        >
            <CadOverlays
                preloader={(
                    <TlantiWorkspacePreloader
                        visible={Boolean(moduleLoadingLabel) || loading}
                        title={WORKSPACE_TITLES.app}
                        subtitle={moduleLoadingLabel ?? 'Processing Workspace'}
                        themeMode={themeMode}
                    />
                )}
                moduleDialogs={(
                    <AnimatePresence>
                        {isOdontogramOpen && (
                            <ModulePanelErrorBoundary label="Odontograma" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <Odontogram onClose={exitClinicalModule} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isDicomModuleOpen && (
                            <ModulePanelErrorBoundary label="DICOM Viewer" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <DicomViewerModule
                                        themeMode={themeMode}
                                        onClose={exitClinicalModule}
                                        onImportDicom={() => void handleImportAction()}
                                        activeCase={activeCaseContext}
                                        capabilities={browserCapabilities}
                                    />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isImplantModuleOpen && (
                            <ModulePanelErrorBoundary label="Implant" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <ImplantModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isSurgicalGuideModuleOpen && (
                            <ModulePanelErrorBoundary label="Surgical Guide" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <SurgicalGuideModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isSplintModuleOpen && (
                            <ModulePanelErrorBoundary label="Splint" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <SplintModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isCephModuleOpen && (
                            <ModulePanelErrorBoundary label="Ceph" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <CephModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isFabModuleOpen && (
                            <ModulePanelErrorBoundary label="Fab" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <FabModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                        {isAlignersModuleOpen && (
                            <ModulePanelErrorBoundary label="Aligners" themeMode={themeMode} onClose={exitClinicalModule}>
                                <Suspense fallback={null}>
                                    <AlignersModule themeMode={themeMode} onClose={exitClinicalModule} activeCase={activeCaseContext} capabilities={browserCapabilities} />
                                </Suspense>
                            </ModulePanelErrorBoundary>
                        )}
                    </AnimatePresence>
                )}
            />
            <CadCommandPanels
                {...commandPanelsViewModel}
            />
            <input type="file" multiple ref={fileInputRef} className="hidden" onChange={onFileChange} accept=".zip,.dcm,.dicom,.ima,.jpg,.jpeg,.png,.webp,.avif,.svg,.obj,.stl,.ply,.glb,.gltf" title="Import files into CAD" />
            <input type="file" multiple ref={directoryInputRef} className="hidden" onChange={onDirectoryChange} accept=".dcm,.dicom,.ima,.jpg,.jpeg,.png,.webp,.avif,.svg,.obj,.stl,.ply,.glb,.gltf" title="Import folder into CAD" {...({ webkitdirectory: 'true', directory: 'true' } as Record<string, string>)} />
            <div
                className={clsx("relative w-full flex-1", viewport.isCompact ? 'min-h-[100dvh]' : 'min-h-[calc(100dvh-4rem)]')}
                style={{ minHeight: viewport.isCompact ? '100dvh' : 'calc(100dvh - 4rem)' }}
            >
                {activeTool === 'CROP' && (
                    <div className="absolute top-4 left-1/2 -translate-x-1/2 z-50 flex gap-2">
                        <div className="bg-black/70 text-white px-4 py-2 rounded-lg text-sm font-medium backdrop-blur-sm shadow-lg border border-white/10">
                            Draw a box to crop the scene
                        </div>
                    </div>
                )}
                {cropPlanes.length > 0 && (
                    <div className="absolute top-4 right-4 z-50">
                        <button
                            onClick={() => setCropPlanes([])}
                            className="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg text-sm font-medium shadow-lg transition-colors flex items-center gap-2"
                        >
                            <X size={16} /> Reset Crop
                        </button>
                    </div>
                )}
                {activeTool === 'CROP' && cropStart && cropEnd && (
                    <div
                        className="absolute border-2 border-blue-500 bg-blue-500/20 pointer-events-none z-50"
                        style={{
                            left: Math.min(cropStart.x, cropEnd.x),
                            top: Math.min(cropStart.y, cropEnd.y),
                            width: Math.abs(cropEnd.x - cropStart.x),
                            height: Math.abs(cropEnd.y - cropStart.y)
                        }}
                    />
                )}
                <CanvasScene
                    {...canvasSceneViewModel}
                    overlayChildren={
                        articulatorFrames.length > 0 ? (
                            <JawMotionOverlay frames={articulatorFrames} />
                        ) : null
                    }
                />

                {files.length === 0 && !activeCaseContext && (
                    <div
                        className={clsx(
                            'pointer-events-none absolute inset-0 z-10 flex justify-center p-6',
                            viewport.isCompact ? 'items-start px-6 pt-80' : isApplePro ? 'items-start pt-56' : 'items-start pt-44',
                        )}
                    >
                        <div data-visual-qa-empty-state="true" className="pointer-events-auto w-[min(42rem,100%)] max-w-none rounded-md border border-border bg-surface/95 p-6 shadow-lg backdrop-blur-sm">
                            <p className="text-xs uppercase text-text-secondary">CAD Design</p>
                            <h2 className="mt-2 text-balance text-2xl font-display text-text-display">No active case loaded</h2>
                            <p className="mt-2 text-pretty text-sm text-text-secondary">
                                Open a case from the TlantiCAD Workspace to start working in CAD.
                            </p>

                            <WorkspaceStatusStrip items={caseStatusItems} className="mt-4" />

                            <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
                                <div className="rounded-2xl border border-border bg-card px-3 py-3">
                                    <p className="text-[11px] uppercase text-text-secondary">Models</p>
                                    <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{caseAssetStats.models}</p>
                                </div>
                                <div className="rounded-2xl border border-border bg-card px-3 py-3">
                                    <p className="text-[11px] uppercase text-text-secondary">DICOM</p>
                                    <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{caseAssetStats.dicom}</p>
                                </div>
                                <div className="rounded-2xl border border-border bg-card px-3 py-3">
                                    <p className="text-[11px] uppercase text-text-secondary">Images</p>
                                    <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{caseAssetStats.images}</p>
                                </div>
                                <div className="rounded-2xl border border-border bg-card px-3 py-3">
                                    <p className="text-[11px] uppercase text-text-secondary">Documents</p>
                                    <p className="mt-1 text-lg font-semibold tabular-nums text-text-display">{caseAssetStats.documents}</p>
                                </div>
                            </div>

                            <div className="mt-4 flex flex-wrap gap-2">
                                {onBackToDb ? (
                                    <button
                                        type="button"
                                        onClick={onBackToDb}
                                        className="rounded-2xl border border-border bg-card px-4 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
                                    >
                                        Back to project
                                    </button>
                                ) : null}
                            </div>
                        </div>
                    </div>
                )}

                {contextMenu ? (
                    <div
                        className={clsx('absolute z-50 min-w-[220px] overflow-hidden rounded-lg border border-border bg-surface py-1 animate-in fade-in zoom-in-95 duration-100')}
                        style={{ top: contextMenu.y, left: contextMenu.x }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        {contextMenu.type === 'object' && contextMenu.targetId ? (
                            <>
                                <div className="border-b border-border-visible px-4 py-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary">
                                    {isExpertMode ? 'Object menu · expert' : 'Object menu · assistant'}
                                </div>
                                <button onClick={() => { focusSelection(contextMenu.targetId!); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Maximize size={14} /> Zoom to object
                                </button>
                                <button onClick={() => { toggleVisibility(contextMenu.targetId!); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Eye size={14} /> Toggle visibility
                                </button>
                                <button onClick={() => { toggleWireframe(contextMenu.targetId!); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Grid3X3 size={14} /> Toggle wireframe
                                </button>
                                {isExpertMode ? (
                                    <>
                                        <button onClick={() => { const file = files.find((entry) => entry.id === contextMenu.targetId); if (file) { updateFileProperty(file.id, { opacity: file.opacity < 0.95 ? 1 : 0.35 }); } setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                            <Sparkles size={14} /> Toggle transparent
                                        </button>
                                        <button onClick={() => { toast('Insertion direction review enabled for the selected object.', 'info'); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                            <ArrowDownToLine size={14} /> Set insertion direction
                                        </button>
                                        <button onClick={() => { setActiveTool('MEASURE'); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                            <Ruler size={14} /> Measure from here
                                        </button>
                                    </>
                                ) : null}
                                <div className="my-1 h-px bg-border-visible" />
                                <button onClick={() => { exportSTL(contextMenu.targetId!); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Save size={14} /> {t.export_stl}
                                </button>
                                {isExpertMode ? (
                                    <button onClick={() => { deleteFile(contextMenu.targetId!); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-red-400 hover:bg-red-500/10">
                                        <Trash2 size={14} /> Remove object
                                    </button>
                                ) : null}
                            </>
                        ) : (
                            <>
                                <div className="border-b border-border-visible px-4 py-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary">
                                    {isExpertMode ? 'General menu · expert' : 'General menu · assistant'}
                                </div>
                                <button onClick={() => { setGridVisible(!gridVisible); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Grid3X3 size={14} /> Toggle grid
                                </button>
                                <button onClick={() => { resetView(); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Maximize size={14} /> {t.reset_view}
                                </button>
                                <button onClick={() => { openCommandPalette(); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                    <Search size={14} /> Buscar funciones…
                                </button>
                                {isExpertMode ? (
                                    <>
                                        <button onClick={() => { addCustomView(); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                            <BookmarkPlus size={14} /> Añadir vista personalizada
                                        </button>
                                        <button onClick={() => { toggleFunctionalSelection('cutView'); setContextMenu(null); }} className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-text-primary hover:bg-surface-raised">
                                            <Slice size={14} /> Toggle cut view
                                        </button>
                                    </>
                                ) : null}
                            </>
                        )}
                    </div>
                ) : null}

                <CadWorkbenchShell
                    topDock={!isDicomWorkspaceOpen ? (
                        <DentalCadShellBar
                            activeCaseNumber={activeCaseContext?.caseNumber}
                            activeCaseName={activeCaseContext?.name}
                            activeModule={activeModule}
                            activeProductModule={activeProductModule}
                            caseStatusItems={caseStatusItems}
                            themeMode={themeMode}
                            filesCount={files.length}
                            isCompact={viewport.isCompact}
                            isExpertMode={isExpertMode}
                            activeSelectionSummary={activeSelectionSummary}
                            activeToolLabel={activeToolLabel}
                            snapLabel={snapValue ? `Snap ${snapValue}` : 'Snap off'}
                            transformSpace={transformSpace}
                            canUndo={historyPointer > 0}
                            canRedo={historyPointer < history.length - 1}
                            moduleToolset={moduleToolset}
                            activeModuleAction={activeModuleAction}
                            onBackToDb={onBackToDb}
                            onImport={() => void handleImportAction()}
                            onExport={() => void handleExportAction()}
                            onOpenAssetLibrary={() => setIsAssetLibraryOpen(true)}
                            onOpenLayers={() => setIsFileManagerOpen(true)}
                            onOpenProperties={() => selectedId && setIsPropertiesPanelOpen(true)}
                            propertiesDisabled={!selectedId}
                            onOpenCommandPalette={openCommandPalette}
                            onOpenSmileWorkflow={() => {
                                closeClinicalModulePanels('smile');
                                setIsSmileWorkflowOpen(true);
                            }}
                            onOpenImplant={() => launchImplantModule()}
                            onOpenGuide={() => launchGuideModule()}
                            onOpenSplint={() => launchSplintModule()}
                            onOpenOdontogram={() => launchOdontogramModule()}
                            onToggleTheme={() => setThemeMode(themeMode === 'dark' ? 'light' : 'dark')}
                            onUndo={undo}
                            onRedo={redo}
                            onToggleExpertMode={() => setIsExpertMode((prev) => !prev)}
                            onActivateCadCore={() => setActiveTool('SELECT')}
                            onActivateDicomReview={() => {
                                if (selectedFile?.type !== 'DICOM') {
                                    void handleImportAction();
                                    toast('Importa o selecciona una serie DICOM para abrir el flujo radiológico.', 'info');
                                } else {
                                    toast('El viewer DICOM ya está listo para este caso.', 'success');
                                }
                            }}
                            onSelectProductModule={selectProductModule}
                            onSetTool={setActiveTool}
                            onRunModuleAction={runModuleAction}
                        />
                    ) : null}
                    rails={!isDicomWorkspaceOpen ? (
                        <DentalCadWorkbenchRails
                            activeTool={activeTool}
                            activeProductModule={activeProductModule}
                            activeRoadmap={activeModuleRoadmap}
                            activeWorkflowPhase={activeWorkflowPhase}
                            activeModuleAction={activeModuleAction}
                            moduleToolset={moduleToolset}
                            isCompact={viewport.isCompact}
                            isExpertMode={isExpertMode}
                            hasSelection={Boolean(selectedId)}
                            onSetTool={setActiveTool}
                            onRunModuleAction={runModuleAction}
                            onImport={() => void handleImportAction()}
                            onOpenLayers={() => setIsFileManagerOpen(true)}
                            onOpenAssetLibrary={() => setIsAssetLibraryOpen(true)}
                            onOpenCommandPalette={openCommandPalette}
                            onOpenToolsWorkflows={() => setIsToolsWorkflowsOpen(true)}
                            onOpenProperties={() => selectedId && setIsPropertiesPanelOpen(true)}
                            onOpenOdontogram={() => launchOdontogramModule()}
                        />
                    ) : null}
                    activeOverlays={!isDicomWorkspaceOpen && isToolsWorkflowsOpen ? (
                        <Suspense fallback={null}>
                            <CadToolsWorkflowsPanel
                                caseId={caseId}
                                activeModuleId={activeProductModule.id}
                                activeWorkflowPhaseId={activeWorkflowPhase.id}
                                onClose={() => setIsToolsWorkflowsOpen(false)}
                            />
                        </Suspense>
                    ) : null}
                />

                {files.length > 0 && shouldShowAdvancedSummary ? (
                    <div className={clsx('absolute left-0 right-0 z-20 flex justify-center px-4', viewport.isCompact ? 'top-[9.9rem]' : 'top-[8.9rem]')}>
                        <div className={clsx('pointer-events-auto flex max-w-[min(92vw,72rem)] flex-wrap items-center gap-3 rounded-[1.3rem] border border-border/80 bg-surface/84 px-3 py-2 shadow-xl backdrop-blur-xl', viewport.isCompact ? 'justify-start' : 'justify-between')}>
                            <div className="flex flex-wrap items-center gap-2">
                                {activeFunctionalItems.map((item) => (
                                    <button
                                        key={item.key}
                                        type="button"
                                        onClick={() => toggleFunctionalSelection(item.key)}
                                        className="rounded-full border px-3 py-1 text-[11px] font-mono uppercase transition-colors"
                                        style={{
                                            borderColor: item.activeColor,
                                            backgroundColor: `${item.activeColor}1A`,
                                            color: item.activeColor,
                                        }}
                                    >
                                        {item.label}
                                    </button>
                                ))}
                                {savedViews.length
                                    ? savedViews.map((view) => (
                                        <button
                                            key={view.id}
                                            type="button"
                                            onClick={() => applySavedView(view)}
                                            className="rounded-full border border-border bg-card px-3 py-1.5 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
                                        >
                                            {view.name}
                                        </button>
                                    ))
                                    : isExpertMode ? (
                                        <button
                                            type="button"
                                            onClick={addCustomView}
                                            className="rounded-full border border-dashed border-border px-3 py-1 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
                                        >
                                            Add custom view
                                        </button>
                                    ) : null}
                            </div>

                            <div className="flex min-w-[18rem] items-center justify-between gap-3">
                                <div>
                                    <p className="text-[11px] font-mono uppercase text-text-secondary">{isExpertMode ? 'Expert mode' : 'Assistant mode'} · {activeSmilePlaybook.title}</p>
                                    <p className="text-sm text-text-primary">{assistantSteps[assistantStepIndex]}</p>
                                </div>
                                <Bot className="size-4 text-text-secondary" />
                            </div>
                            <div className="flex items-center gap-2">
                                <button
                                    type="button"
                                    onClick={() => smilePreviousStage(selectedSmilePlaybookId)}
                                    className="rounded-full border border-border px-3 py-1 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
                                >
                                    ANT
                                </button>
                                <p className="text-[11px] font-mono uppercase text-text-secondary">Paso {assistantStepIndex + 1} / {assistantSteps.length}</p>
                                <button
                                    type="button"
                                    onClick={() => smileNextStage(selectedSmilePlaybookId)}
                                    className="rounded-full border border-border px-3 py-1 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
                                >
                                    SIG
                                </button>
                            </div>
                        </div>
                    </div>
                ) : null}

                {(activeOverlayAsset || activeCadPreset || activeDentalLibrary) ? (
                    <div className={clsx('absolute left-0 right-0 z-10 flex justify-center px-4', isApplePro ? (viewport.isCompact ? 'top-[11rem]' : 'top-[8.9rem]') : (viewport.isCompact ? 'top-[8.4rem]' : 'top-[7rem]'))}>
                        <div className="pointer-events-auto flex flex-wrap items-center gap-2 rounded-2xl border border-border bg-surface px-3 py-2 text-xs text-text-secondary shadow-lg">
                            {activeOverlayAsset ? (
                                <div className="flex items-center gap-2 rounded-full border border-border px-2 py-1 text-text-primary">
                                    <button type="button" onClick={() => activeOverlayFileId && alignOverlayFile(activeOverlayFileId, 'front')} className={clsx('rounded-full px-2 py-1 hover:bg-surface-raised', activeOverlayAlignment === 'front' && 'bg-surface-raised')} aria-label="Align overlay to front">
                                        Front
                                    </button>
                                    <button type="button" onClick={() => activeOverlayFileId && alignOverlayFile(activeOverlayFileId, 'plane')} className={clsx('rounded-full px-2 py-1 hover:bg-surface-raised', activeOverlayAlignment === 'plane' && 'bg-surface-raised')} aria-label="Align overlay to plane">
                                        Plane
                                    </button>
                                    <button type="button" onClick={() => { setActiveOverlayAsset(null); setActiveOverlayFileId(null); }} className="rounded-full px-3 py-1 hover:bg-surface-raised" aria-label="Clear active overlay">
                                        Overlay · {activeOverlayAsset.label} <X className="ml-2 inline size-3" />
                                    </button>
                                </div>
                            ) : null}
                            {activeCadPreset ? (
                                <button type="button" onClick={() => setActiveCadPreset(null)} className="rounded-full border border-border px-3 py-1 text-text-primary hover:bg-surface-raised" aria-label="Clear active preset">
                                    Preset · {activeCadPreset.label} <X className="ml-2 inline size-3" />
                                </button>
                            ) : null}
                            {activeDentalLibrary ? (
                                <button type="button" onClick={() => setActiveDentalLibrary(null)} className="rounded-full border border-border px-3 py-1 text-text-primary hover:bg-surface-raised" aria-label="Clear active dental library">
                                    Tooth library · {activeDentalLibrary.label} <X className="ml-2 inline size-3" />
                                </button>
                            ) : null}
                        </div>
                    </div>
                ) : null}

                {!isDicomWorkspaceOpen && !viewport.isCompact ? (
                    <div className="pointer-events-none absolute bottom-4 left-4 z-30 max-w-[22rem] rounded-[0.9rem] border border-white/8 bg-[#101215]/90 px-3 py-2 text-[11px] text-text-secondary shadow-[0_14px_32px_rgba(0,0,0,0.22)] backdrop-blur-xl">
                        <div className="flex items-center gap-2 font-medium text-text-primary">
                            <Cpu className="size-3.5 text-[#9ee7bd]" />
                            CAD kernel boundary
                        </div>
                        <div className="mt-1 flex flex-wrap gap-1.5">
                            <span className="rounded-full border border-[#9ee7bd]/30 bg-[#9ee7bd]/10 px-2 py-0.5 text-[#9ee7bd]">
                                Mesh-first active
                            </span>
                            <span className="rounded-full border border-white/10 bg-white/6 px-2 py-0.5 text-text-secondary">
                                OCCT native planned
                            </span>
                            <span className="rounded-full border border-[#8fb4ff]/30 bg-[#8fb4ff]/10 px-2 py-0.5 text-[#8fb4ff]">
                                OCCT WASM preview
                            </span>
                        </div>
                    </div>
                ) : null}

                {activeDicomFile && (
                    <Suspense fallback={null}>
                        <TlantiOhifViewer
                            file={activeDicomFile}
                            onUpdate={updateFileProperty}
                            onMetadataLoaded={updateDicomMetadata}
                            themeMode={themeMode}
                        />
                    </Suspense>
                )}





                {/* Dicom Controls Overlay */}
                {activeDicomFile && (
                    <DicomControls
                        file={activeDicomFile}
                        onUpdate={updateFileProperty}
                        themeMode={themeMode}
                        onStartPlanning={() => {
                            setActiveTool('SELECT');
                            toast('DICOM adjustments applied. Continue planning.', 'success');
                        }}
                    />
                )}

            </div>

            {/* File Manager Sidebar */}
            <Suspense fallback={null}>
                <FileManager
                    files={files}
                    setFiles={setFiles}
                    selectedId={selectedId}
                    setSelectedId={setSelectedId}
                    themeMode={themeMode}
                    open={isFileManagerOpen}
                    onOpenChange={setIsFileManagerOpen}
                />
            </Suspense>

            <Suspense fallback={null}>
                <CadAssetLibraryPanel
                    compact={viewport.isCompact}
                    moduleId={canonicalModuleId}
                    open={isAssetLibraryOpen}
                    onOpenChange={setIsAssetLibraryOpen}
                    onAction={(asset, mode) => void handleCadAssetLibraryAction(asset, mode)}
                    onBatchAction={(assets, mode) => void handleCadAssetLibraryBatchAction(assets, mode)}
                />
            </Suspense>

            {isGroupSelectorOpen ? (
                <Suspense fallback={null}>
                    <CadGroupSelectorPanel
                        files={files}
                        setFiles={setFiles}
                        compact={viewport.isCompact}
                        open={isGroupSelectorOpen}
                        onOpenChange={setIsGroupSelectorOpen}
                    />
                </Suspense>
            ) : null}

            {isCadGuideOpen ? (
                <Suspense fallback={null}>
                    <CadWorkflowGuidePanel
                        compact={viewport.isCompact}
                        open={isCadGuideOpen}
                        onOpenChange={setIsCadGuideOpen}
                        expertMode={isExpertMode}
                    />
                </Suspense>
            ) : null}

            {isSmileWorkflowOpen ? (
                <Suspense fallback={null}>
                    <SmileDesignWorkflowPanel
                        compact={viewport.isCompact}
                        open={isSmileWorkflowOpen}
                        onOpenChange={setIsSmileWorkflowOpen}
                        activeTool={activeTool}
                        onSetTool={setActiveTool}
                    />
                </Suspense>
            ) : null}

            {/* V207 + V209: Articulator floating panel — toggle via Ctrl+J or command palette,
                vendor persisted in case.articulator. */}
            <ArticulatorContainer
                open={isArticulatorOpen}
                onClose={() => setIsArticulatorOpen(false)}
                onFramesChange={setArticulatorFrames}
                vendorId={activeCaseContext?.articulator?.vendorId ?? null}
                onVendorChange={(vendor) =>
                    persistArticulatorPatch({
                        vendorId: vendor?.id ?? null,
                        vendorLabel: vendor?.label ?? null,
                    })
                }
                onOpenInfluencingTeeth={() => setIsInfluencingTeethOpen(true)}
            />

            {/* V211 — Influencing-teeth picker. */}
            <InfluencingTeethDialog
                open={isInfluencingTeethOpen}
                initialFdis={activeCaseContext?.articulator?.influencingFdis ?? []}
                onClose={() => setIsInfluencingTeethOpen(false)}
                onConfirm={(fdis) => {
                    persistArticulatorPatch({ influencingFdis: fdis });
                    // Fire-and-forget: tell backend so future simulations weight these teeth.
                    void createBackendArticulatorAdapter()
                        .setInfluencingTeeth(fdis)
                        .catch(() => undefined);
                }}
            />

            {/* V113 + V117 + V148 + V175: CAD Wizard slot — driven by activeCaseContext via cadWizardViewModel */}
            <CadWizardSlot
                open={isCadWizardOpen}
                onClose={() => setIsCadWizardOpen(false)}
                caseId={cadWizardViewModel.caseId}
                caseFolderPath={cadWizardViewModel.caseFolderPath}
                meshPath={cadWizardViewModel.meshPath}
                preopPath={cadWizardViewModel.preopPath}
                waxupPath={cadWizardViewModel.waxupPath}
                toothFdi={cadWizardViewModel.toothFdi}
                material={cadWizardViewModel.material}
                teethPayload={cadWizardViewModel.teethPayload}
                needsPreopWaxup={cadWizardViewModel.needsPreopWaxup}
                needsAbutment={cadWizardViewModel.needsAbutment}
            />


            {/* Properties Panel */}
            <Suspense fallback={null}>
                <PropertiesPanel
                    file={selectedFile || null}
                    onUpdate={updateFileProperty}
                    themeMode={themeMode}
                    open={viewport.isCompact ? isPropertiesPanelOpen : true}
                    onOpenChange={setIsPropertiesPanelOpen}
                />
            </Suspense>

            {/* 3D Navigation Controls */}
            {!isDicomWorkspaceOpen && !viewport.isCompact && (
                <div className={clsx("absolute bottom-36 left-4 z-40 flex flex-col gap-3 rounded-[1.2rem] border border-white/8 bg-[#101215]/94 p-2.5 shadow-[0_16px_34px_rgba(0,0,0,0.24)] backdrop-blur-xl transition-all duration-300", !isExpertMode && 'hidden')}>
                    <div>
                        <p className="mb-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary">Navigation</p>
                        <div className="flex flex-col gap-1">
                            <button onClick={() => handleZoom(-0.2)} className={clsx("h-10 w-10 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Zoom In">
                                <ZoomIn size={20} />
                            </button>
                            <button onClick={() => handleZoom(0.2)} className={clsx("h-10 w-10 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Zoom Out">
                                <ZoomOut size={20} />
                            </button>
                        </div>
                    </div>

                    <div className={clsx("w-full h-px bg-border")} />

                    <div className="grid grid-cols-3 gap-1">
                        <div />
                        <button onClick={() => handlePan(0, 1)} className={clsx("h-9 w-9 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Pan Up"><ArrowUp size={16} /></button>
                        <div />
                        <button onClick={() => handlePan(-1, 0)} className={clsx("h-9 w-9 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Pan Left"><ArrowLeft size={16} /></button>
                        <button onClick={resetView} className={clsx("h-9 w-9 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-display")} title="Reset View"><Crosshair size={16} /></button>
                        <button onClick={() => handlePan(1, 0)} className={clsx("h-9 w-9 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Pan Right"><ArrowRight size={16} /></button>
                        <div />
                        <button onClick={() => handlePan(0, -1)} className={clsx("h-9 w-9 rounded-xl flex items-center justify-center transition-colors hover:bg-surface-raised text-text-primary")} title="Pan Down"><ArrowDown size={16} /></button>
                        <div />
                    </div>
                </div>
            )}

            {!isDicomWorkspaceOpen && isNavOpen && (
                <div className={clsx('absolute bottom-36 right-6 z-40 flex flex-col gap-3 rounded-[1.5rem] border border-border/80 bg-surface/92 p-3 shadow-[0_20px_40px_rgba(0,0,0,0.35)] backdrop-blur-xl transition-all duration-300', viewport.isCompact && 'bottom-28 right-3')}>
                    <div>
                        <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">View arrows</p>
                        <div className="mt-2 grid grid-cols-3 gap-2">
                            <button onClick={() => setView('TOP')} className="rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Top</button>
                            <button onClick={() => setView('FRONT')} className="rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Front</button>
                            <button onClick={() => setView('RIGHT')} className="rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Right</button>
                            <button onClick={() => setView('LEFT')} className="rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Left</button>
                            <button onClick={resetView} className="rounded-xl border border-border bg-text-display px-2 py-2 text-[11px] font-mono uppercase text-black transition-transform hover:scale-[1.02]">Cube</button>
                            <button onClick={() => setView('BACK')} className="rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Back</button>
                            <button onClick={() => setView('BOTTOM')} className="col-span-3 rounded-xl border border-border bg-card px-2 py-2 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">Bottom</button>
                        </div>
                    </div>

                    <div className="rounded-xl border border-border bg-card p-2">
                        <div className="flex items-center justify-between gap-2">
                            <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Custom views</p>
                            <button onClick={addCustomView} className="rounded-md p-1 text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary" aria-label="Add custom view">
                                <BookmarkPlus className="size-4" />
                            </button>
                        </div>
                        <div className="mt-2 flex flex-wrap gap-2">
                            {savedViews.length ? savedViews.slice(-3).map((view) => (
                                <button key={view.id} onClick={() => applySavedView(view)} className="rounded-full border border-border px-2.5 py-1 text-[11px] font-mono uppercase text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary">
                                    {view.name}
                                </button>
                            )) : (
                                <p className="text-xs text-text-secondary">Save a viewpoint to reuse it here.</p>
                            )}
                        </div>
                    </div>
                </div>
            )}

            <CadWorkbenchShell
                floatingDock={!isDicomWorkspaceOpen && floatingDockItems.length ? (
                    <div className="pointer-events-none absolute inset-x-0 bottom-10 z-40 flex justify-center px-4 md:bottom-12">
                        <div className="pointer-events-auto">
                            <FloatingDock
                                items={floatingDockItems}
                                desktopClassName={clsx(isApplePro && 'hidden')}
                                mobileClassName="translate-y-0"
                            />
                        </div>
                    </div>
                ) : null}
                bottomStatus={!isDicomWorkspaceOpen && !viewport.isCompact ? (
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 z-20 flex h-8 items-center justify-between gap-4 overflow-hidden bg-[linear-gradient(180deg,rgba(5,11,10,0),rgba(5,11,10,0.84))] px-4 text-[10px] text-text-secondary">
                        <div className="flex min-w-0 items-center gap-2 overflow-hidden">
                            <span className="truncate">tool {activeToolLabel.toLowerCase()}</span>
                            <span className="text-text-disabled">/</span>
                            <span className="truncate">{activeSelectionSummary}</span>
                        </div>
                        <div className="flex items-center gap-3">
                            <span>{snapValue ? `snap ${snapValue}` : 'snap off'}</span>
                            <span>{controlScheme}</span>
                            <span>{isExpertMode ? 'expert' : 'assistant'}</span>
                        </div>
                    </div>
                ) : null}
                copilot={!isDicomWorkspaceOpen && !viewport.isCompact && isCopilotOpen ? (
                    <div className="absolute right-4 top-[7.2rem] bottom-12 z-30 w-[22rem]">
                        <Suspense fallback={null}>
                            <CadVoiceCopilotPanel
                                activeTool={activeTool}
                                activeToolLabel={activeToolLabel}
                                selectedFile={selectedFile ?? null}
                                fileCount={files.length}
                                isExpertMode={isExpertMode}
                                moduleId={moduleId}
                                onSetTool={setActiveTool}
                            />
                        </Suspense>
                    </div>
                ) : null}
            />

        </div>
    );
};

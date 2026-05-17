import React, { Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { motion } from 'framer-motion';
import { Language, ThemeMode } from '../types';
import { TRANSLATIONS } from '../utils/translations';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { useTlantiDbCaseViewModel } from '@/hooks/useTlantiDbCaseViewModel';
import {
    FilePlus, Copy,
    Settings, User, Printer, Share2, Factory, ScanFace, Braces,
    Layers, Activity, Sun, Moon,
    Scan, Target, Syringe, Smile, FolderOpen, Boxes, ArrowRight, MoonStar
} from 'lucide-react';
import clsx from 'clsx';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/interfaces-select';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { Switch } from '@/components/ui/interfaces-switch';
import { useToast } from '@/components/ui/use-toast';
import { APP_ABOUT } from '@/lib/app-about';
import { loadBackendCatalog, loadBackendWorkspaceCatalog, loadGeometryProbe, loadPythonBridgeStatus, loadSystemRuntimeReport, type BackendCatalog, type BackendWorkspaceCatalog, type GeometryProbe, type PythonBridgeStatus, type SystemRuntimeReport } from '@/lib/backend-integrations';
import { recommendPythonWorkflow } from '@/lib/python-workflow-router';
import { getDisplayToothNumber, getToothNumberingLegend } from '@/lib/dental-numbering';
import { buildConnectorKey, getDentalConnectorCandidates, getRestorationLabel } from '@/lib/dental-workflow';
import type { DentalImplantMode, DentalProductionMethod } from '@/lib/dental-workflow';
import { exportRuntimeDiagnosticsReport, formatRuntimeDiagnostics, getRuntimeDiagnostics, type RuntimeDiagnostics } from '@/lib/runtime-diagnostics';
import { derivePerformanceProfileFromRuntimeReport, shouldAutoApplyRuntimeProfile } from '@/lib/tlantidb-machine-profile';
import { formatTimeZoneLabel, getGroupedTimeZones } from '@/lib/timezone-groups';
import { deriveChecklistFromCase, derivePipelineFromClinicalAssets } from '@/lib/tlantidb-clinical-workflow';
import { resolveCaseStatusFromClinicalProgress, transitionTlantiCaseStatus } from '@/lib/tlantidb-case-state-machine';
import { inferDentalLibraryDefaults } from '@/lib/public-asset-dental-defaults';
import { inferClinicalAssetTags } from '@/lib/tlanticad-asset-classification';
import type { PublicAssetLibraryActionMode } from '@/components/asset-library/PublicAssetLibraryBrowser';
import { copyTextToClipboard, isTauriRuntime, openPathInSystem, persistCurrentWindowState, revealPathInSystem, showDesktopNotification } from '@/platform/desktop-system';
import { deleteTlanticadAsset, exportTlanticadCaseFolder, importTlanticadAssetsFromAbsolutePaths, importTlanticadAssetsFromWebFiles, openTlanticadCaseFromDisk, openTlanticadCaseFromWebFile, relinkTlanticadAssetFromDisk, replaceTlanticadAssetFromDisk, resolveTlanticadAssetPath, saveTlanticadCaseFolder, writeMillboxInteropXml } from '@/lib/tlanticad-case-files';
import type { PublicAssetLibraryItem } from '@/lib/public-asset-library';
import { FloatingDock } from '@/components/ui/floating-dock';
import type { CaseNavigationIntent, WorkspaceContextSyncRequest } from '@/lib/workspace-shell-contract';
import { searchTlantiDb, type TlantiDbSearchResult } from '@/lib/tlantidb-search';
import { ClinicalCommandUseCase, resolveCadProductModuleForRoute, resolveTlantiModuleDefinition, TauriClinicalCommandRepository } from '@/core';
import { createDefaultCase, createDefaultTlantiDbState, hydrateTlantiDbState, loadTlantiDbState, saveTlantiDbState, shouldCreateFreshStartupCase, subscribeTlantiDbState, type TlantiCase, type TlantiCaseAsset, type TlantiDbPerformanceProfile, type TlantiDbState, type TlantiToothState } from '@/stores/tlantidb-case-store';
import { setPendingCadImport } from '@/lib/launcher-pending-import';
import { openWorkspaceWindow } from '@/platform/module-windows';
import { TlantiDbWorkspaceHeader } from '@/components/tlantidb/TlantiDbWorkspaceHeader';
import type { TlantiDbSidebarPanelId } from '@/components/tlantidb/TlantiDbSidebar';
import { ModuleImportGateDialog } from '@/components/tlantidb/ModuleImportGateDialog';
import { ToothWorkDefinitionDialog } from '@/components/tlantidb/ToothWorkDefinitionDialog';
import { ClinicalOrderFlowPanel } from '@/components/tlantidb/ClinicalOrderFlowPanel';
import type { CreateCaseWizardDraft } from '@/components/tlantidb/CreateCaseWizard';
import type { TlantiDbWorkloadWizardSubmit } from '@/components/tlantidb/TlantiDbWorkloadWizard';
import { DentalDbWelcomeLauncher } from '@/components/tlantidb/DentalDbWelcomeLauncher';
import {
    LazyPanelFallback,
    SettingsErrorBoundary,
} from '@/components/tlantidb/TlantiDbSettingsInfra';
import {
    DEFAULT_CLINICAL_IMPORT_ACCEPT,
    resolveClinicalImportAccept,
    resolveClinicalImportFilters,
} from '@/lib/tlantidb-clinical-import-filters';
import {
    BackendIntegrationPanel,
    BackendWorkspacePanel,
    ClinicalQualityInteropPanel,
    CollaborationAutoplanningPanel,
    DicomDentalExecutionPanel,
    DicomDentalRoadmapPanel,
    HybridOpsPrecisionPanel,
    InteractiveOdontogram,
    PlatformToolkitPanel,
    PublicAssetLibraryBrowser,
    SystemRuntimeSettingsPanel,
    TlantiDbActionPanel,
    TlantiDbCaseBrowserSheet,
    TlantiDbClinicalAssetsPanel,
    TlantiDbSidebar,
    TlantiDbWorkloadWizard,
    ToothWorkflowSheet,
} from '@/components/tlantidb/tlantidb-lazy-panels';
import { applyWorkloadToCase, getMissingRequiredAssets, getWorkloadStatus, normalizeWorkloadModuleTarget } from '@/lib/tlantidb-workloads';
import { useTlantiDbUiStore } from '@/components/tlantidb/application/useTlantiDbUiStore';
import { TLANTIDB_WORKSPACE_MODULES } from '@/components/tlantidb/domain/workspace-modules';

interface Props {
    onEnter: (options?: { caseId?: string; moduleId?: string }) => void;
    onSyncWorkspaceContext: (request: WorkspaceContextSyncRequest) => void;
    language: Language;
    setLanguage: (lang: Language) => void;
    themeMode: ThemeMode;
    setThemeMode: (mode: ThemeMode) => void;
    caseId?: string;
    moduleId?: string;
}

type WorkflowHistorySnapshot = Pick<TlantiCase, 'toothMap' | 'connectorOverrides'>;

const DEFAULT_CREATE_CASE_DRAFT: CreateCaseWizardDraft = {
    caseName: 'New restorative case',
    clientName: 'Nuevo paciente',
    orderNumber: '',
    patientName: '',
    patientBirthDate: '',
    technicianName: '',
    laboratoryName: '',
    dueAt: '',
    clinicalNotes: '',
};

function ensureRouteCaseInState(state: TlantiDbState, routeCaseId?: string): TlantiDbState {
    if (!routeCaseId) {
        return state;
    }

    const existing = state.cases.find((item) => item.id === routeCaseId);
    if (existing) {
        return {
            ...state,
            activeCaseId: existing.id,
        };
    }

    const routeCase = createDefaultCase({
        id: routeCaseId,
        name: `Caso ${routeCaseId.slice(0, 8)}`,
        status: 'case-data-complete',
    });

    return {
        ...state,
        activeCaseId: routeCase.id,
        cases: [routeCase, ...state.cases],
    };
}

export const TlantiDB: React.FC<Props> = ({ onEnter, onSyncWorkspaceContext, language, setLanguage, themeMode, setThemeMode, caseId, moduleId }) => {
    const t = TRANSLATIONS[language];
    const activeWorkspaceModuleId = useMemo(() => resolveTlantiModuleDefinition(moduleId).id, [moduleId]);
    const viewport = useViewportProfile();
    const { toast } = useToast();
    const isShortViewport = viewport.isShort;
    const shouldForceFreshStartupRef = useRef(shouldCreateFreshStartupCase(caseId));
    const workflowUndoRef = useRef<WorkflowHistorySnapshot[]>([]);
    const workflowRedoRef = useRef<WorkflowHistorySnapshot[]>([]);
    const [databaseState, setDatabaseState] = useState<TlantiDbState>(() => (
        ensureRouteCaseInState(
            shouldForceFreshStartupRef.current ? createDefaultTlantiDbState() : loadTlantiDbState(),
            caseId,
        )
    ));
    const [isCreateCaseOpen, setIsCreateCaseOpen] = useState(false);
    const [pendingModuleGate, setPendingModuleGate] = useState<{ moduleId: string; label: string } | null>(null);
    const [isToothWorkDialogOpen, setIsToothWorkDialogOpen] = useState(false);
    const [isIntakeSheetOpen, setIsIntakeSheetOpen] = useState(false);
    const [isWorkflowSheetOpen, setIsWorkflowSheetOpen] = useState(false);
    const [createCaseStep, setCreateCaseStep] = useState<1 | 2 | 3>(1);
    const [createCaseDraft, setCreateCaseDraft] = useState<CreateCaseWizardDraft>(() => DEFAULT_CREATE_CASE_DRAFT);
    const [searchResults, setSearchResults] = useState<TlantiDbSearchResult[]>([]);

    // Time State
    const [currentTime, setCurrentTime] = useState(new Date());
    const caseFileInputRef = React.useRef<HTMLInputElement>(null);
    const assetFileInputRef = React.useRef<HTMLInputElement>(null);
    const launcherDicomInputRef = React.useRef<HTMLInputElement>(null);
    const launcherDicomDirectoryInputRef = React.useRef<HTMLInputElement>(null);
    const initialSyncRef = useRef(false);
    const [pendingAssetMutation, setPendingAssetMutation] = useState<{ mode: 'replace' | 'relink'; asset: TlantiCaseAsset } | null>(null);
    const [guidedImportRoles, setGuidedImportRoles] = useState<TlantiCaseAsset['role'][] | null>(null);
    const [assetFileAccept, setAssetFileAccept] = useState(DEFAULT_CLINICAL_IMPORT_ACCEPT);

    const persistDatabaseState = useCallback((updater: TlantiDbState | ((prev: TlantiDbState) => TlantiDbState)) => {
        setDatabaseState((prev) => {
            const next = typeof updater === 'function' ? updater(prev) : updater;
            saveTlantiDbState(next);
            return next;
        });
    }, []);

    useEffect(() => {
        return subscribeTlantiDbState((state) => {
            setDatabaseState(state);
        });
    }, []);

    useEffect(() => {
        let mounted = true;

        void hydrateTlantiDbState({ freshStartup: shouldForceFreshStartupRef.current }).then((state) => {
            if (mounted) {
                setDatabaseState(ensureRouteCaseInState(state, caseId));
            }
        });

        return () => {
            mounted = false;
        };
    }, [caseId]);

    useEffect(() => {
        if (!caseId || initialSyncRef.current) {
            return;
        }

        initialSyncRef.current = true;

        persistDatabaseState((prev) => {
            const existing = prev.cases.find((item) => item.id === caseId);
            if (existing) {
                return { ...prev, activeCaseId: existing.id };
            }

            const newCase = createDefaultCase({
                id: caseId,
                name: `Caso ${caseId.slice(0, 8)}`,
                status: 'case-data-complete',
            });

            return {
                ...prev,
                activeCaseId: newCase.id,
                cases: [newCase, ...prev.cases],
            };
        });
    }, [caseId, persistDatabaseState]);

    const activeCase = useMemo(() => {
        if (!databaseState.cases.length) {
            return createDefaultCase();
        }

        return databaseState.cases.find((item) => item.id === (caseId ?? databaseState.activeCaseId)) ?? databaseState.cases[0];
    }, [caseId, databaseState.activeCaseId, databaseState.cases]);

    const updateActiveCase = useCallback((updater: Partial<TlantiCase> | ((current: TlantiCase) => Partial<TlantiCase>)) => {
        if (!activeCase) {
            return;
        }

        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: activeCase.id,
            cases: prev.cases.map((item) => {
                if (item.id !== activeCase.id) {
                    return item;
                }

                const patch = typeof updater === 'function' ? updater(item) : updater;
                return {
                    ...item,
                    ...patch,
                    updatedAt: new Date().toISOString(),
                };
            }),
        }));
    }, [activeCase, persistDatabaseState]);

    const recordClinicalCaseEvent = useCallback(async (
        event: string,
        payload: Record<string, unknown> = {},
        assetIds: string[] = [],
    ) => {
        if (!activeCase?.id || !isTauriRuntime()) {
            return;
        }

        try {
            const moduleDefinition = resolveCadProductModuleForRoute(activeWorkspaceModuleId);
            const useCase = new ClinicalCommandUseCase(new TauriClinicalCommandRepository());
            await useCase.record({
                caseId: activeCase.id,
                moduleId: moduleDefinition.id,
                toolId: 'clinical-command-record',
                command: {
                    event,
                    caseId: activeCase.id,
                    caseNumber: activeCase.caseNumber,
                    status: activeCase.status,
                    ...payload,
                },
                assetIds,
            });
        } catch (error) {
            console.warn('Could not persist clinical command event', error);
        }
    }, [activeCase?.caseNumber, activeCase?.id, activeCase?.status, activeWorkspaceModuleId]);

    const rememberWorkflowSnapshot = useCallback((current: TlantiCase) => {
        workflowUndoRef.current = [
            ...workflowUndoRef.current.slice(-39),
            {
                toothMap: current.toothMap,
                connectorOverrides: current.connectorOverrides,
            },
        ];
        workflowRedoRef.current = [];
    }, []);

    const undoWorkflowEdit = useCallback(() => {
        const previous = workflowUndoRef.current.pop();
        if (!previous) {
            return;
        }

        updateActiveCase((current) => {
            workflowRedoRef.current = [
                ...workflowRedoRef.current.slice(-39),
                {
                    toothMap: current.toothMap,
                    connectorOverrides: current.connectorOverrides,
                },
            ];
            return previous;
        });
    }, [updateActiveCase]);

    const redoWorkflowEdit = useCallback(() => {
        const next = workflowRedoRef.current.pop();
        if (!next) {
            return;
        }

        updateActiveCase((current) => {
            workflowUndoRef.current = [
                ...workflowUndoRef.current.slice(-39),
                {
                    toothMap: current.toothMap,
                    connectorOverrides: current.connectorOverrides,
                },
            ];
            return next;
        });
    }, [updateActiveCase]);

    const selectedTeeth = useMemo(
        () => Object.entries(activeCase.toothMap).filter(([, value]) => value.selected && !value.antagonist).map(([key]) => Number(key.replace('tooth-', ''))),
        [activeCase.toothMap],
    );

    const antagonistTeeth = useMemo(
        () => Object.entries(activeCase.toothMap).filter(([, value]) => value.selected && value.antagonist).map(([key]) => Number(key.replace('tooth-', ''))),
        [activeCase.toothMap],
    );

    const preferences = databaseState.preferences;
    const timeZone = preferences.timeZone;
    const numberingSystem = preferences.numberingSystem;
    const assetProfile = preferences.assetProfile;
    const operatorAlias = preferences.operatorAlias;
    const navigationSensitivity = preferences.navigationSensitivity;
    const performanceProfile = preferences.performanceProfile;
    const setPreferences = useCallback((patch: Partial<TlantiDbState['preferences']>) => {
        persistDatabaseState((prev) => ({
            ...prev,
            preferences: {
                ...prev.preferences,
                ...patch,
            },
        }));
    }, [persistDatabaseState]);

    const setTimeZone = useCallback((value: string) => {
        setPreferences({ timeZone: value });
    }, [setPreferences]);

    const setNumberingSystem = useCallback((value: 'FDI' | 'UNIVERSAL') => {
        setPreferences({ numberingSystem: value });
    }, [setPreferences]);

    const setAssetProfile = useCallback((value: 'clinical' | 'lab' | 'demo') => {
        setPreferences({ assetProfile: value });
    }, [setPreferences]);

    const setOperatorAlias = useCallback((value: string) => {
        setPreferences({ operatorAlias: value });
    }, [setPreferences]);

    const setNavigationSensitivity = useCallback((key: 'zoom' | 'pan' | 'rotation', value: number) => {
        setPreferences({
            navigationSensitivity: {
                ...navigationSensitivity,
                [key]: value,
            },
        });
    }, [navigationSensitivity, setPreferences]);

    const setPerformanceProfile = useCallback((nextProfile: TlantiDbPerformanceProfile | ((current: TlantiDbPerformanceProfile) => TlantiDbPerformanceProfile)) => {
        const resolved = typeof nextProfile === 'function' ? nextProfile(performanceProfile) : nextProfile;
        setPreferences({ performanceProfile: resolved });
    }, [performanceProfile, setPreferences]);

    const setClientName = useCallback((value: string) => {
        updateActiveCase({ clientName: value });
    }, [updateActiveCase]);

    const setClientId = useCallback((value: string) => {
        updateActiveCase({ clientId: value });
    }, [updateActiveCase]);

    const setPatientName = useCallback((value: string) => {
        updateActiveCase({ patientName: value });
    }, [updateActiveCase]);

    const setPatientDateOfBirth = useCallback((value: string) => {
        updateActiveCase({ patientDateOfBirth: value });
    }, [updateActiveCase]);

    const setOrderNumber = useCallback((value: string) => {
        updateActiveCase({ orderNumber: value });
    }, [updateActiveCase]);

    const setLaboratoryName = useCallback((value: string) => {
        updateActiveCase({ laboratoryName: value });
    }, [updateActiveCase]);

    const setProjectName = useCallback((value: string) => {
        updateActiveCase({ name: value });
    }, [updateActiveCase]);

    const setTechnicianName = useCallback((value: string) => {
        updateActiveCase({ technicianName: value });
    }, [updateActiveCase]);

    const setTechnicianId = useCallback((value: string) => {
        updateActiveCase({ technicianId: value });
    }, [updateActiveCase]);

    const setNotes = useCallback((value: string) => {
        updateActiveCase({ notes: value });
    }, [updateActiveCase]);

    const patchActiveCaseCollaboration = useCallback((updater: (current: NonNullable<TlantiCase['collaboration']>) => NonNullable<TlantiCase['collaboration']>) => {
        updateActiveCase((current) => ({
            collaboration: updater(current.collaboration ?? {
                reviewLock: null,
                comments: [],
                decisions: [],
                approvals: [],
                notifications: [],
                autoPlanningFeedback: {},
            }),
        }));
    }, [updateActiveCase]);

    const patchActiveCaseOperations = useCallback((updater: (current: NonNullable<TlantiCase['operations']>) => NonNullable<TlantiCase['operations']>) => {
        updateActiveCase((current) => ({
            operations: updater(current.operations ?? {
                remoteJobs: [],
                kernelTransition: {
                    preference: 'mesh-first',
                    preferredKernel: 'auto',
                    offlineFallback: true,
                    geometryBenchmarkScore: null,
                    lastPolicyUpdateAt: null,
                },
            }),
        }));
    }, [updateActiveCase]);

    const groupedTimeZones = useMemo(() => getGroupedTimeZones(), []);
    const workspaceLocation = useMemo(() => {
        if (activeCase.storagePath) {
            return activeCase.storagePath.replace(/[/\\][^/\\]+$/, '');
        }

        if (typeof window !== 'undefined') {
            return window.location.origin;
        }

        return 'Unknown workspace location';
    }, [activeCase.storagePath]);
    const aboutTimestamp = useMemo(() => {
        try {
            return new Intl.DateTimeFormat(language === 'es' ? 'es-MX' : 'en-US', {
                dateStyle: 'full',
                timeStyle: 'short',
                timeZone,
            }).format(currentTime);
        } catch {
            return currentTime.toLocaleString();
        }
    }, [currentTime, language, timeZone]);

    useEffect(() => {
        const timer = setInterval(() => setCurrentTime(new Date()), 60_000);
        return () => clearInterval(timer);
    }, []);

    const [isSettingsOpen, setIsSettingsOpen] = useState(false);
    const isLeftPanelOpen = useTlantiDbUiStore((state) => state.isLeftPanelOpen);
    const setIsLeftPanelOpen = useTlantiDbUiStore((state) => state.setIsLeftPanelOpen);
    const activeSidebarPanel = useTlantiDbUiStore((state) => state.activeSidebarPanel);
    const setActiveSidebarPanel = useTlantiDbUiStore((state) => state.setActiveSidebarPanel);
    const [isCaseBrowserOpen, setIsCaseBrowserOpen] = useState(false);
    const [activeToothNumber, setActiveToothNumber] = useState<string | null>(null);
    const hoveredToothNumber = useTlantiDbUiStore((state) => state.hoveredToothNumber);
    const setHoveredToothNumber = useTlantiDbUiStore((state) => state.setHoveredToothNumber);
    const [isToothSheetOpen, setIsToothSheetOpen] = useState(false);
    const [activeToothLibraryAsset, setActiveToothLibraryAsset] = useState<PublicAssetLibraryItem | null>(null);
    const [runtimeDiagnostics, setRuntimeDiagnostics] = useState<RuntimeDiagnostics | null>(null);
    const [runtimeDiagnosticsLoading, setRuntimeDiagnosticsLoading] = useState(false);
    const [backendCatalog, setBackendCatalog] = useState<BackendCatalog | null>(null);
    const [geometryProbe, setGeometryProbe] = useState<GeometryProbe | null>(null);
    const [systemRuntimeReport, setSystemRuntimeReport] = useState<SystemRuntimeReport | null>(null);
    const [pythonBridgeStatus, setPythonBridgeStatus] = useState<PythonBridgeStatus | null>(null);
    const [backendCatalogLoading, setBackendCatalogLoading] = useState(false);
    const [backendWorkspaceCatalog, setBackendWorkspaceCatalog] = useState<BackendWorkspaceCatalog | null>(null);
    const [systemRuntimeLoading, setSystemRuntimeLoading] = useState(false);
    const [systemRuntimeError, setSystemRuntimeError] = useState<string | null>(null);
    const [backendWorkspaceLoading, setBackendWorkspaceLoading] = useState(false);
    const [backendWorkspaceError, setBackendWorkspaceError] = useState<string | null>(null);
    const launcherDicomRecommendation = useMemo(() => recommendPythonWorkflow(pythonBridgeStatus, {
        fileName: 'study.dcm',
        moduleId: 'dicom',
    }), [pythonBridgeStatus]);

    const applyRuntimeProfile = useCallback((report: SystemRuntimeReport, mode: 'auto' | 'manual' = performanceProfile.mode) => {
        const derived = derivePerformanceProfileFromRuntimeReport(report);
        setPerformanceProfile({
            ...derived,
            mode,
            source: mode === 'auto' ? 'auto' : 'manual',
        });
    }, [performanceProfile.mode, setPerformanceProfile]);

    const refreshSystemRuntimeReport = useCallback(async (options?: { autoApply?: boolean }) => {
        setSystemRuntimeLoading(true);
        setSystemRuntimeError(null);

        try {
            const nextReport = await loadSystemRuntimeReport();
            setSystemRuntimeReport(nextReport);

            if (options?.autoApply && shouldAutoApplyRuntimeProfile(performanceProfile, nextReport)) {
                applyRuntimeProfile(nextReport, 'auto');
            }

            return nextReport;
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Unable to load the desktop runtime report.';
            setSystemRuntimeError(message);
            return null;
        } finally {
            setSystemRuntimeLoading(false);
        }
    }, [applyRuntimeProfile, performanceProfile]);

    const refreshBackendWorkspaceCatalog = useCallback(async () => {
        setBackendWorkspaceLoading(true);
        setBackendWorkspaceError(null);

        try {
            const nextCatalog = await loadBackendWorkspaceCatalog();
            setBackendWorkspaceCatalog(nextCatalog);
            return nextCatalog;
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Unable to inspect the Rust crate workspace.';
            setBackendWorkspaceError(message);
            return null;
        } finally {
            setBackendWorkspaceLoading(false);
        }
    }, []);

    const refreshBackendCatalog = useCallback(async () => {
        setBackendCatalogLoading(true);

        try {
            const [nextCatalog, nextProbe] = await Promise.all([
                loadBackendCatalog(),
                loadGeometryProbe(),
            ]);
            setBackendCatalog(nextCatalog);
            setGeometryProbe(nextProbe);
            return nextCatalog;
        } finally {
            setBackendCatalogLoading(false);
        }
    }, []);

    useEffect(() => {
        if (performanceProfile.mode !== 'auto') {
            return;
        }

        void refreshSystemRuntimeReport({ autoApply: true });
    }, [performanceProfile.mode, refreshSystemRuntimeReport]);

    useEffect(() => {
        if (!isSettingsOpen) {
            return;
        }

        let cancelled = false;
        setRuntimeDiagnosticsLoading(true);

        void getRuntimeDiagnostics({
            workspaceLocation,
            caseLocation: activeCase.storagePath ?? 'No local case path saved yet',
        }).then((nextDiagnostics) => {
            if (!cancelled) {
                setRuntimeDiagnostics(nextDiagnostics);
            }
        }).finally(() => {
            if (!cancelled) {
                setRuntimeDiagnosticsLoading(false);
            }
        });

        return () => {
            cancelled = true;
        };
    }, [activeCase.storagePath, isSettingsOpen, workspaceLocation]);

    useEffect(() => {
        if (!isSettingsOpen) {
            return;
        }

        void refreshSystemRuntimeReport({ autoApply: performanceProfile.mode === 'auto' });
        void loadPythonBridgeStatus().then((status) => {
            setPythonBridgeStatus(status);
        });
        void refreshBackendCatalog();
        void refreshBackendWorkspaceCatalog();
    }, [isSettingsOpen, performanceProfile.mode, refreshBackendCatalog, refreshBackendWorkspaceCatalog, refreshSystemRuntimeReport]);

    const copyDiagnostics = useCallback(async () => {
        const diagnostics = runtimeDiagnostics ?? await getRuntimeDiagnostics({
            workspaceLocation,
            caseLocation: activeCase.storagePath ?? 'No local case path saved yet',
        });

        await copyTextToClipboard(formatRuntimeDiagnostics(diagnostics));
        await showDesktopNotification('TlantiCAD Studio', 'Diagnostics copied to clipboard.');
    }, [activeCase.storagePath, runtimeDiagnostics, workspaceLocation]);

    const createCrashReport = useCallback(async () => {
        const diagnostics = runtimeDiagnostics ?? await getRuntimeDiagnostics({
            workspaceLocation,
            caseLocation: activeCase.storagePath ?? 'No local case path saved yet',
        });

        const report = formatRuntimeDiagnostics(diagnostics);
        const savedPath = await exportRuntimeDiagnosticsReport(report);
        if (savedPath) {
            await showDesktopNotification('TlantiCAD Studio', `Crash report ready: ${savedPath}`);
        }
    }, [activeCase.storagePath, runtimeDiagnostics, workspaceLocation]);

    const syncWorkspaceCase = useCallback((nextCaseId: string, intent: CaseNavigationIntent) => {
        onSyncWorkspaceContext({ caseId: nextCaseId, intent });
    }, [onSyncWorkspaceContext]);

    const activateCase = useCallback((nextCaseId: string, intent: CaseNavigationIntent = 'activate') => {
        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: nextCaseId,
            cases: prev.cases.map((item) =>
                item.id === nextCaseId
                    ? {
                        ...item,
                        lastOpenedAt: new Date().toISOString(),
                    }
                    : item,
            ),
        }));
        syncWorkspaceCase(nextCaseId, intent);
    }, [persistDatabaseState, syncWorkspaceCase]);

    const markActiveCasePipeline = useCallback((patch: Partial<NonNullable<TlantiCase['pipeline']>>) => {
        let eventPayload: Record<string, unknown> | null = null;
        updateActiveCase((current) => {
            const nextPipeline = {
                scan: current.pipeline?.scan ?? false,
                design: current.pipeline?.design ?? false,
                model: current.pipeline?.model ?? false,
                manufacture: current.pipeline?.manufacture ?? false,
                export: current.pipeline?.export ?? false,
                ...patch,
            };
            const nextStatus = patch.export
                ? transitionTlantiCaseStatus(current.status, 'exported')
                : patch.manufacture
                    ? transitionTlantiCaseStatus(current.status, 'manufacturing-ready')
                    : patch.design
                        ? transitionTlantiCaseStatus(current.status, 'design-started')
                        : resolveCaseStatusFromClinicalProgress(current, current.assets ?? [], nextPipeline);

            eventPayload = { patch, nextStatus };

            return {
                status: nextStatus,
                pipeline: nextPipeline,
                lastOpenedAt: new Date().toISOString(),
            };
        });

        if (eventPayload) {
            void recordClinicalCaseEvent('pipeline-updated', eventPayload);
        }
    }, [recordClinicalCaseEvent, updateActiveCase]);

    const activeToothState = useMemo(() => {
        if (!activeToothNumber) {
            return undefined;
        }

        return activeCase.toothMap[`tooth-${activeToothNumber}`];
    }, [activeCase.toothMap, activeToothNumber]);

    const hoveredToothState = useMemo(() => {
        if (!hoveredToothNumber) {
            return undefined;
        }

        return activeCase.toothMap[`tooth-${hoveredToothNumber}`];
    }, [activeCase.toothMap, hoveredToothNumber]);

    const applyToothPatch = useCallback((toothNumber: string, patch: Partial<TlantiToothState>) => {
        updateActiveCase((current) => {
            rememberWorkflowSnapshot(current);
            const key = `tooth-${toothNumber}`;
            const existing = current.toothMap[key];
            return {
                toothMap: {
                    ...current.toothMap,
                    [key]: {
                        // Spread existing first so optional fields not in the patch
                        // (workTypeId, workTimeMinutes, additionalScans, biteSplintMode,
                        // minimalThicknessMm, cementGapMm, screwHoleCut, …) are preserved.
                        ...(existing ?? {}),
                        // Overrides + always-required defaults follow.
                        selected: true,
                        antagonist: patch.antagonist ?? existing?.antagonist ?? false,
                        condition: patch.condition ?? existing?.condition ?? patch.restorationType ?? 'anatomic-coping',
                        restorationType: patch.restorationType ?? existing?.restorationType ?? 'anatomic-coping',
                        material: patch.material ?? existing?.material ?? 'zirconia',
                        shade: patch.shade ?? existing?.shade ?? current.materialShade,
                        productionMethod: patch.productionMethod ?? existing?.productionMethod ?? 'inhouse-milling',
                        implantMode: patch.implantMode ?? existing?.implantMode ?? 'none',
                        usePreOpModel: patch.usePreOpModel ?? existing?.usePreOpModel ?? false,
                        useExtraGingivaScan: patch.useExtraGingivaScan ?? existing?.useExtraGingivaScan ?? false,
                        adjacentTooth: patch.adjacentTooth ?? existing?.adjacentTooth ?? false,
                        omitInBridge: patch.omitInBridge ?? existing?.omitInBridge ?? false,
                        // Apply explicit overrides from the patch last to win over `...existing`.
                        ...(patch.workTypeId !== undefined ? { workTypeId: patch.workTypeId } : {}),
                        ...(patch.workTimeMinutes !== undefined ? { workTimeMinutes: patch.workTimeMinutes } : {}),
                        ...(patch.minimalThicknessMm !== undefined ? { minimalThicknessMm: patch.minimalThicknessMm } : {}),
                        ...(patch.cementGapMm !== undefined ? { cementGapMm: patch.cementGapMm } : {}),
                        ...(patch.biteSplintMode !== undefined ? { biteSplintMode: patch.biteSplintMode } : {}),
                        ...(patch.biteSplintAntagonistScan !== undefined ? { biteSplintAntagonistScan: patch.biteSplintAntagonistScan } : {}),
                        ...(patch.screwHoleCut !== undefined ? { screwHoleCut: patch.screwHoleCut } : {}),
                        ...(patch.additionalScans !== undefined
                            ? {
                                  additionalScans: {
                                      ...(existing?.additionalScans ?? {}),
                                      ...patch.additionalScans,
                                  },
                              }
                            : {}),
                    },
                },
            };
        });
    }, [rememberWorkflowSnapshot, updateActiveCase]);

    const clearTooth = useCallback((toothNumber: string) => {
        updateActiveCase((current) => {
            rememberWorkflowSnapshot(current);
            const nextMap = { ...current.toothMap };
            delete nextMap[`tooth-${toothNumber}`];
            return { toothMap: nextMap };
        });
        setIsToothSheetOpen(false);
    }, [rememberWorkflowSnapshot, updateActiveCase]);

    const toggleTooth = useCallback((toothNumber: string, type: 'primary' | 'antagonist' = 'primary', interaction?: { additive?: boolean }) => {
        setActiveToothNumber(toothNumber);

        if (type === 'antagonist') {
            applyToothPatch(toothNumber, {
                antagonist: true,
                restorationType: 'antagonist',
                condition: 'antagonist',
            });
            return;
        }

        const libraryPatch = activeToothLibraryAsset ? inferDentalLibraryDefaults(activeToothLibraryAsset).patch : {};
        applyToothPatch(toothNumber, { antagonist: false, ...libraryPatch });
        if (!interaction?.additive) {
            setIsToothWorkDialogOpen(true);
        }
    }, [activeToothLibraryAsset, applyToothPatch]);

    const selectedToothSummaries = useMemo(() => {
        return Object.entries(activeCase.toothMap)
            .filter(([, value]) => value.selected)
            .map(([key, value]) => ({
                toothNumber: key.replace('tooth-', ''),
                numberingLegend: getToothNumberingLegend(key.replace('tooth-', ''), numberingSystem),
                displayNumber: getDisplayToothNumber(key.replace('tooth-', ''), numberingSystem),
                label: getRestorationLabel(value.restorationType),
                material: value.material ?? 'zirconia',
                shade: value.shade ?? activeCase.materialShade,
            }))
            .sort((a, b) => Number(a.toothNumber) - Number(b.toothNumber));
    }, [activeCase.materialShade, activeCase.toothMap, numberingSystem]);

    const connectorCandidates = useMemo(() => {
        return getDentalConnectorCandidates(activeCase.toothMap, activeCase.connectorOverrides ?? {});
    }, [activeCase.connectorOverrides, activeCase.toothMap]);

    const isPristineCase = useMemo(() => {
        return !activeCase.assets?.length
            && selectedToothSummaries.length === 0
            && !activeCase.clientName.trim()
            && !activeCase.clientId.trim()
            && !(activeCase.patientName ?? '').trim()
            && !(activeCase.patientDateOfBirth ?? '').trim()
            && !(activeCase.orderNumber ?? '').trim()
            && !(activeCase.laboratoryName ?? '').trim()
            && !activeCase.technicianName.trim()
            && !activeCase.technicianId.trim()
            && !activeCase.notes.trim()
            && !activeCase.workloadId;
    }, [activeCase.assets, activeCase.clientId, activeCase.clientName, activeCase.laboratoryName, activeCase.notes, activeCase.orderNumber, activeCase.patientDateOfBirth, activeCase.patientName, activeCase.technicianId, activeCase.technicianName, activeCase.workloadId, selectedToothSummaries.length]);

    const activeConnectorCount = useMemo(() => connectorCandidates.filter((item) => item.active && item.canConnect).length, [connectorCandidates]);
    const omittedBridgeCount = useMemo(() => Object.values(activeCase.toothMap).filter((item) => item.restorationType === 'omit-in-bridge').length, [activeCase.toothMap]);

    /** Asset role summary shown in the pre-module import gate. */
    const moduleImportGateSummary = useMemo(() => {
        const assets = activeCase.assets ?? [];
        const dicomAssetCount = assets.filter((a) => a.role === 'dicom-study').length;
        const modelAssetCount = assets.filter(
            (a) =>
                a.role === 'prep-scan' ||
                a.role === 'antagonist-scan' ||
                a.role === 'bite-registration' ||
                a.role === 'gingiva-scan' ||
                a.role === 'restoration-model',
        ).length;
        return {
            dicomAssetCount,
            modelAssetCount,
            hasAnyData: dicomAssetCount > 0 || modelAssetCount > 0,
        };
    }, [activeCase.assets]);

    const hasCadWorkspaceData = useMemo(() => {
        return moduleImportGateSummary.hasAnyData || selectedToothSummaries.length > 0 || Boolean(activeCase.workloadId);
    }, [activeCase.workloadId, moduleImportGateSummary.hasAnyData, selectedToothSummaries.length]);
    const {
        clinicalChecklist,
        completedChecklistCount,
        nextChecklistItem,
        primaryScanChecklistItem,
        primaryScanReady,
        canLaunchCadModule,
        caseStatusLabel,
        caseStatusTone,
        actionAvailability,
        recentCases,
        labQueueStats,
    } = useTlantiDbCaseViewModel({
        activeCase,
        databaseState,
        isPristineCase,
        selectedToothCount: selectedToothSummaries.length,
    });

    const toggleConnector = useCallback((fromTooth: string, toTooth: string) => {
        const key = buildConnectorKey(fromTooth, toTooth);
        const candidate = connectorCandidates.find((item) => item.key === key);
        if (!candidate?.canConnect) {
            return;
        }

        updateActiveCase((current) => ({
            ...(() => {
                rememberWorkflowSnapshot(current);
                return {};
            })(),
            connectorOverrides: {
                ...(current.connectorOverrides ?? {}),
                [key]: !candidate.active,
            },
        }));
    }, [connectorCandidates, rememberWorkflowSnapshot, updateActiveCase]);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            const target = event.target as HTMLElement | null;
            const isEditable = target?.closest('input, textarea, select, [contenteditable="true"]');
            if (isEditable) {
                return;
            }

            const key = event.key.toLowerCase();
            const hasPlatformModifier = event.metaKey || event.ctrlKey;

            if (hasPlatformModifier && key === 'z') {
                event.preventDefault();
                if (event.shiftKey) {
                    redoWorkflowEdit();
                    return;
                }
                undoWorkflowEdit();
                return;
            }

            if (event.ctrlKey && key === 'y') {
                event.preventDefault();
                redoWorkflowEdit();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [redoWorkflowEdit, undoWorkflowEdit]);

    const handleCreateCase = useCallback(() => {
        const newCase = createDefaultCase({
            name: createCaseDraft.caseName,
            clientName: createCaseDraft.clientName,
            patientName: createCaseDraft.patientName || createCaseDraft.clientName,
            patientDateOfBirth: createCaseDraft.patientBirthDate,
            technicianName: createCaseDraft.technicianName,
            laboratoryName: createCaseDraft.laboratoryName,
            orderNumber: createCaseDraft.orderNumber,
            dueAt: createCaseDraft.dueAt,
            notes: createCaseDraft.clinicalNotes,
            status: 'case-data-complete',
        });

        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: newCase.id,
            cases: [newCase, ...prev.cases],
        }));

        setCreateCaseDraft(DEFAULT_CREATE_CASE_DRAFT);
        setCreateCaseStep(1);
        setIsCreateCaseOpen(false);
        syncWorkspaceCase(newCase.id, 'create');
    }, [createCaseDraft, persistDatabaseState, syncWorkspaceCase]);

    const handleCreateWorkload = useCallback((payload: TlantiDbWorkloadWizardSubmit) => {
        const baseCase = createDefaultCase({
            name: payload.caseName,
            clientName: payload.clientName,
            patientName: payload.clientName,
            activeJaw: payload.activeJaw,
            status: 'case-data-complete',
        });
        const newCase = applyWorkloadToCase(baseCase, {
            workloadId: payload.workloadId,
            toothNumbers: payload.toothNumbers,
            activeJaw: payload.activeJaw,
            materialShade: payload.materialShade,
            occlusionScanType: payload.occlusionScanType,
        });
        const missingAssets = getMissingRequiredAssets(newCase);
        const moduleTarget = normalizeWorkloadModuleTarget(newCase.moduleTarget);

        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: newCase.id,
            cases: [newCase, ...prev.cases],
        }));

        setCreateCaseDraft(DEFAULT_CREATE_CASE_DRAFT);
        setCreateCaseStep(1);
        setIsCreateCaseOpen(false);
        syncWorkspaceCase(newCase.id, 'create');

        if (missingAssets.length) {
            toast(`${missingAssets.length} clinical asset${missingAssets.length === 1 ? '' : 's'} required before export.`, 'info');
        }

        if (payload.openAfterCreate) {
            onEnter({ caseId: newCase.id, moduleId: moduleTarget });
        }
    }, [onEnter, persistDatabaseState, syncWorkspaceCase, toast]);

    const updateCreateCaseDraft = useCallback((patch: Partial<CreateCaseWizardDraft>) => {
        setCreateCaseDraft((current) => ({ ...current, ...patch }));
    }, []);

    const cancelCreateCaseWizard = useCallback(() => {
        setCreateCaseStep(1);
        setIsCreateCaseOpen(false);
    }, []);

    const ensureLauncherCase = useCallback(() => {
        if (isPristineCase) {
            return activeCase;
        }

        const nextCase = createDefaultCase();
        persistDatabaseState({
            ...databaseState,
            activeCaseId: nextCase.id,
            cases: [nextCase, ...databaseState.cases],
        });
        return nextCase;
    }, [activeCase, databaseState, isPristineCase, persistDatabaseState]);

    const launchDicomPlanning = useCallback((payload: { files?: File[]; paths?: string[]; directoryPath?: string }) => {
        const targetCase = ensureLauncherCase();
        if (payload.directoryPath) {
            setPendingCadImport({
                source: 'launcher-dicom',
                directoryPath: payload.directoryPath,
            });
        } else {
            setPendingCadImport({
                source: 'launcher-dicom',
                ...(payload.files ? { files: payload.files } : { paths: payload.paths ?? [] }),
            });
        }
        onEnter({ caseId: targetCase.id, moduleId: 'dicom' });
    }, [ensureLauncherCase, onEnter]);

    const openLauncherDicomPicker = useCallback(async () => {
        if (isTauriRuntime()) {
            try {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const result = await open({
                    title: 'Open DICOM or ZIP study',
                    multiple: true,
                    directory: false,
                    filters: [{ name: 'DICOM / ZIP', extensions: ['zip', 'dcm', 'dicom', 'ima'] }],
                });

                if (!result) {
                    return;
                }

                const paths = (Array.isArray(result) ? result : [result]).filter((item): item is string => typeof item === 'string');
                if (paths.length) {
                    launchDicomPlanning({ paths });
                }
                return;
            } catch (error) {
                console.error(error);
            }
        }

        launcherDicomInputRef.current?.click();
    }, [launchDicomPlanning]);

    const openLauncherDicomFolderPicker = useCallback(async () => {
        if (isTauriRuntime()) {
            try {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const result = await open({
                    title: 'Open DICOM folder',
                    multiple: false,
                    directory: true,
                });

                if (typeof result === 'string') {
                    launchDicomPlanning({ directoryPath: result });
                }
                return;
            } catch (error) {
                console.error(error);
            }
        }

        launcherDicomDirectoryInputRef.current?.click();
    }, [launchDicomPlanning]);

    const handleLauncherDicomChange = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
        const files = Array.from(event.target.files ?? []);
        event.target.value = '';

        if (!files.length) {
            return;
        }

        launchDicomPlanning({ files });
    }, [launchDicomPlanning]);

    const handleLauncherDicomDirectoryChange = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
        const files = Array.from(event.target.files ?? []);
        event.target.value = '';

        if (!files.length) {
            return;
        }

        launchDicomPlanning({ files });
    }, [launchDicomPlanning]);

    const handleDuplicateCase = useCallback(() => {
        const duplicated = createDefaultCase({
            ...activeCase,
            id: crypto.randomUUID(),
            caseNumber: undefined,
            name: `${activeCase.name} Copy`,
            status: 'new',
            createdAt: undefined,
            updatedAt: undefined,
        });

        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: duplicated.id,
            cases: [duplicated, ...prev.cases],
        }));
    }, [activeCase, persistDatabaseState]);

    const handleWorkspaceSearch = useCallback((query: string) => {
        setSearchResults(searchTlantiDb(databaseState, query));
    }, [databaseState]);

    const handleSidebarPanelChange = useCallback((panel: TlantiDbSidebarPanelId) => {
        setActiveSidebarPanel(panel);
        setIsLeftPanelOpen(true);
    }, []);

    const handleSearchResultSelect = useCallback((result: TlantiDbSearchResult) => {
        if (result.caseId) {
            activateCase(result.caseId, 'search-result');
        }

        if (result.actionId === 'open-settings') {
            setIsSettingsOpen(true);
        }

        if (result.actionId === 'toggle-language') {
            setLanguage(language === 'en' ? 'es' : 'en');
        }
    }, [activateCase, language, setLanguage]);

    const saveWorkspaceSnapshot = useCallback(async () => {
        const result = await saveTlanticadCaseFolder(activeCase, databaseState, activeCase.storagePath ?? undefined);
        if (!result) {
            await showDesktopNotification('TlantiCAD Studio', `Guardado cancelado: ${activeCase.caseNumber}.`);
            return;
        }
        const nextStatus = transitionTlantiCaseStatus(activeCase.status, 'case-data-saved');

        persistDatabaseState((prev) => ({
            ...prev,
            cases: prev.cases.map((item) =>
                item.id === activeCase.id
                    ? {
                        ...item,
                        sourceType: 'local',
                        storagePath: result.caseFolderPath,
                        status: nextStatus,
                    }
                    : item,
            ),
        }));

        await recordClinicalCaseEvent('case-snapshot-saved', {
            status: nextStatus,
            storagePath: result.caseFolderPath,
        });
        await persistCurrentWindowState();
        await showDesktopNotification('TlantiCAD Studio', `Caso ${activeCase.caseNumber} guardado en ${result.caseFolderPath}.`);
    }, [activeCase, databaseState, persistDatabaseState, recordClinicalCaseEvent]);

    const exportActiveCase = useCallback(async () => {
        const result = await exportTlanticadCaseFolder(activeCase, databaseState);
        if (!result) {
            await showDesktopNotification('TlantiCAD Studio', `Export cancelado: ${activeCase.caseNumber}.`);
            return;
        }

        markActiveCasePipeline({ export: true });
        const exportedAt = new Date().toISOString();

        persistDatabaseState((prev) => ({
            ...prev,
            cases: prev.cases.map((item) =>
                item.id === activeCase.id
                    ? {
                        ...item,
                        sourceType: 'local',
                        storagePath: result.caseFolderPath,
                        status: 'exported',
                        lastExportedAt: exportedAt,
                    }
                    : item,
            ),
        }));

        await recordClinicalCaseEvent('case-exported', {
            status: 'exported',
            storagePath: result.caseFolderPath,
            exportedAt,
        });
        await showDesktopNotification('TlantiCAD Studio', `Export listo: ${result.caseFolderPath}`);
    }, [activeCase, databaseState, markActiveCasePipeline, persistDatabaseState, recordClinicalCaseEvent]);

    const generateInteropXml = useCallback(async () => {
        const result = await writeMillboxInteropXml(activeCase, databaseState, activeCase.lastInteropXmlPath ?? undefined);
        if (!result) {
            await showDesktopNotification('TlantiCAD Studio', `XML interop cancelado: ${activeCase.caseNumber}.`);
            return;
        }

        markActiveCasePipeline({ export: true });
        const exportedAt = new Date().toISOString();

        persistDatabaseState((prev) => ({
            ...prev,
            cases: prev.cases.map((item) =>
                item.id === activeCase.id
                    ? {
                        ...item,
                        lastInteropXmlPath: result.xmlPath,
                        storagePath: item.storagePath ?? activeCase.storagePath ?? null,
                        status: 'exported',
                        lastExportedAt: exportedAt,
                    }
                    : item,
            ),
        }));

        await recordClinicalCaseEvent('interop-xml-generated', {
            status: 'exported',
            xmlPath: result.xmlPath,
            exportedAt,
        });
        await showDesktopNotification('TlantiCAD Studio', `XML interop generado: ${result.xmlPath}`);
    }, [activeCase, databaseState, markActiveCasePipeline, persistDatabaseState, recordClinicalCaseEvent]);

    const copyActiveCaseReference = useCallback(async () => {
        const reference = `${activeCase.caseNumber} · ${activeCase.name}`;
        // V143 — try the native Web Share API first (best UX on mobile + desktop
        // browsers that support it, including AirDrop integration on Safari).
        // Falls back to clipboard so the user always gets actionable feedback.
        try {
            if (typeof navigator !== 'undefined' && 'share' in navigator && typeof navigator.share === 'function') {
                await navigator.share({
                    title: `TlantiCAD · ${activeCase.caseNumber}`,
                    text: `${reference}\nLab: ${activeCase.laboratoryName ?? '—'}\nClient: ${activeCase.clientName ?? '—'}`,
                });
                toast(`Caso compartido: ${activeCase.caseNumber}`, 'success');
                void showDesktopNotification('TlantiCAD Studio', `Caso compartido: ${activeCase.caseNumber}`);
                return;
            }
        } catch (error) {
            // AbortError = user cancelled — keep silent. Other errors fall through to clipboard.
            if (error instanceof Error && error.name === 'AbortError') return;
        }
        await copyTextToClipboard(reference);
        toast(`Referencia copiada al portapapeles: ${activeCase.caseNumber}`, 'success');
        void showDesktopNotification('TlantiCAD Studio', `Referencia copiada: ${activeCase.caseNumber}`);
    }, [activeCase.caseNumber, activeCase.clientName, activeCase.laboratoryName, activeCase.name, toast]);

    const printActiveCase = useCallback(async () => {
        await showDesktopNotification('TlantiCAD Studio', `Print queue listo para ${activeCase.caseNumber}.`);
    }, [activeCase.caseNumber]);

    const mergeImportedCase = useCallback((caseData: TlantiCase, sourcePath: string) => {
        persistDatabaseState((prev) => {
            const mergedCase = {
                ...caseData,
                sourceType: 'imported' as const,
                lastOpenedAt: new Date().toISOString(),
                storagePath: sourcePath,
                updatedAt: new Date().toISOString(),
            };
            const existingIndex = prev.cases.findIndex((item) => item.id === mergedCase.id || item.caseNumber === mergedCase.caseNumber);
            const cases = existingIndex >= 0
                ? prev.cases.map((item, index) => (index === existingIndex ? mergedCase : item))
                : [mergedCase, ...prev.cases];
            return {
                ...prev,
                activeCaseId: mergedCase.id,
                cases,
            };
        });
    }, [persistDatabaseState]);

    const openCasePicker = useCallback(async () => {
        try {
            if (isTauriRuntime()) {
                const result = await openTlanticadCaseFromDisk();
                if (!result) {
                    await showDesktopNotification('TlantiCAD Studio', 'Apertura cancelada: no se seleccionó un caso.');
                    return;
                }
                mergeImportedCase(result.caseData, result.sourcePath);
                setIsCaseBrowserOpen(false);
                await showDesktopNotification('TlantiCAD Studio', `Caso cargado: ${result.caseData.caseNumber}`);
                return;
            }

            caseFileInputRef.current?.click();
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo abrir el caso seleccionado.');
        }
    }, [mergeImportedCase]);

    const handleCaseFileChange = useCallback(async (event: React.ChangeEvent<HTMLInputElement>) => {
        const [file] = Array.from(event.target.files ?? []);
        event.target.value = '';

        if (!file) {
            await showDesktopNotification('TlantiCAD Studio', 'Apertura cancelada: no se seleccionó archivo.');
            return;
        }

        try {
            const result = await openTlanticadCaseFromWebFile(file);
            mergeImportedCase(result.caseData, result.sourcePath);
            setIsCaseBrowserOpen(false);
        } catch (error) {
            console.error(error);
        }
    }, [mergeImportedCase]);

    const revealCaseFolder = useCallback(async (caseItem: TlantiCase) => {
        if (!caseItem.storagePath) {
            await showDesktopNotification('TlantiCAD Studio', `El caso ${caseItem.caseNumber} todavía no tiene carpeta guardada.`);
            return;
        }

        try {
            await revealPathInSystem(caseItem.storagePath);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo abrir la carpeta del caso.');
        }
    }, []);

    const persistClinicalAssets = useCallback(async (nextAssets: TlantiCaseAsset[], nextStoragePath?: string | null) => {
        const resolvedStoragePath = nextStoragePath ?? activeCase.storagePath ?? null;
        const nextPipeline = derivePipelineFromClinicalAssets(activeCase, nextAssets);
        const checklist = deriveChecklistFromCase(activeCase, nextAssets);
        const nextStatus = resolveCaseStatusFromClinicalProgress(activeCase, nextAssets, nextPipeline, checklist);
        const nextCase = {
            ...activeCase,
            storagePath: resolvedStoragePath,
            assets: nextAssets,
            pipeline: nextPipeline,
            status: nextStatus,
        };

        updateActiveCase({ storagePath: resolvedStoragePath, assets: nextAssets, pipeline: nextPipeline, status: nextStatus });
        await recordClinicalCaseEvent('clinical-assets-updated', {
            status: nextStatus,
            assetCount: nextAssets.length,
            storagePath: resolvedStoragePath,
        }, nextAssets.map((asset) => asset.id));

        if (resolvedStoragePath && isTauriRuntime()) {
            await saveTlanticadCaseFolder(nextCase, databaseState, resolvedStoragePath);
        }
    }, [activeCase, databaseState, recordClinicalCaseEvent, updateActiveCase]);

    const applyGuidedRoles = useCallback((assets: TlantiCaseAsset[], importedAssets?: TlantiCaseAsset[] | null, preferredRoles?: TlantiCaseAsset['role'][] | null) => {
        if (!preferredRoles?.length || !importedAssets?.length) {
            return assets;
        }

        const importedIds = new Set(importedAssets.map((asset) => asset.id));
        let importIndex = 0;
        return assets.map((asset) => {
            if (!importedIds.has(asset.id)) {
                return asset;
            }

            const preferredRole = preferredRoles[Math.min(importIndex, preferredRoles.length - 1)];
            importIndex += 1;
            return {
                ...asset,
                role: preferredRole,
                tags: Array.from(new Set([...asset.tags, ...inferClinicalAssetTags(asset.name, asset.category), preferredRole])),
                classificationSource: 'manual' as const,
            };
        });
    }, []);

    const openClinicalImportForRoles = useCallback(async (preferredRoles?: TlantiCaseAsset['role'][]) => {
        const roles = preferredRoles ?? guidedImportRoles;
        const filters = resolveClinicalImportFilters(roles);
        try {
            if (isTauriRuntime()) {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({
                    title: roles?.length ? 'Import clinical assets for this step' : 'Import clinical assets',
                    multiple: true,
                    directory: false,
                    filters,
                });

                if (!selected) {
                    await showDesktopNotification('TlantiCAD Studio', 'Importación cancelada: no se seleccionaron assets clínicos.');
                    return;
                }

                const paths = (Array.isArray(selected) ? selected : [selected]).filter((item): item is string => typeof item === 'string');
                const result = await importTlanticadAssetsFromAbsolutePaths(activeCase, databaseState, paths);
                if (!result) {
                    await showDesktopNotification('TlantiCAD Studio', 'Importación cancelada: no hubo archivos válidos.');
                    return;
                }

                const nextAssets = applyGuidedRoles(result.assets, result.importedAssets, roles);
                await persistClinicalAssets(nextAssets, result.caseFolderPath);
                setGuidedImportRoles(null);
                await showDesktopNotification('TlantiCAD Studio', `${result.assets.length} assets disponibles en el caso.`);
                return;
            }

            setPendingAssetMutation(null);
            setGuidedImportRoles(roles ?? null);
            setAssetFileAccept(resolveClinicalImportAccept(roles));
            assetFileInputRef.current?.click();
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudieron importar los assets clínicos.');
        }
    }, [activeCase, applyGuidedRoles, databaseState, guidedImportRoles, persistClinicalAssets]);

    const importClinicalAssets = useCallback(async (preferredRoles?: TlantiCaseAsset['role'][]) => {
        await openClinicalImportForRoles(preferredRoles);
    }, [openClinicalImportForRoles]);

    const importDroppedAssets = useCallback(async (files: File[]) => {
        try {
            const result = await importTlanticadAssetsFromWebFiles(activeCase, databaseState, files);
            if (!result) {
                await showDesktopNotification('TlantiCAD Studio', 'Importación cancelada: no hubo archivos válidos.');
                return;
            }

            const nextAssets = applyGuidedRoles(result.assets, result.importedAssets, guidedImportRoles);
            await persistClinicalAssets(nextAssets, result.caseFolderPath);
            setGuidedImportRoles(null);
            await showDesktopNotification('TlantiCAD Studio', `${files.length} assets importados al caso.`);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudieron importar los archivos soltados en el caso.');
        }
    }, [activeCase, applyGuidedRoles, databaseState, guidedImportRoles, persistClinicalAssets]);

    const handleAssetFileChange = useCallback(async (event: React.ChangeEvent<HTMLInputElement>) => {
        const files = Array.from(event.target.files ?? []);
        event.target.value = '';

        if (!files.length) {
            await showDesktopNotification('TlantiCAD Studio', 'Importación cancelada: no se seleccionaron archivos.');
            return;
        }

        if (pendingAssetMutation) {
            const result = await importTlanticadAssetsFromWebFiles(activeCase, databaseState, [files[0]]);
            setPendingAssetMutation(null);
            if (!result) {
                await showDesktopNotification('TlantiCAD Studio', 'Reemplazo cancelado: no hubo archivo válido.');
                return;
            }

            const importedAsset = result.assets[result.assets.length - 1];
            const nextAsset = importedAsset ? { ...importedAsset, id: pendingAssetMutation.asset.id } : pendingAssetMutation.asset;
            const nextAssets = (activeCase.assets ?? []).map((item) => (item.id === pendingAssetMutation.asset.id ? nextAsset : item));
            await persistClinicalAssets(nextAssets, result.caseFolderPath);
            return;
        }

        const result = await importTlanticadAssetsFromWebFiles(activeCase, databaseState, files);
        if (!result) {
            await showDesktopNotification('TlantiCAD Studio', 'Importación cancelada: no hubo archivos válidos.');
            return;
        }

        const nextAssets = applyGuidedRoles(result.assets, result.importedAssets, guidedImportRoles);
        await persistClinicalAssets(nextAssets, result.caseFolderPath);
        setGuidedImportRoles(null);
    }, [activeCase, applyGuidedRoles, databaseState, guidedImportRoles, pendingAssetMutation, persistClinicalAssets]);

    const guideClinicalImport = useCallback((roles: TlantiCaseAsset['role'][]) => {
        setGuidedImportRoles(roles);
        setPendingAssetMutation(null);
        void openClinicalImportForRoles(roles);
    }, [openClinicalImportForRoles]);

    const mergeLibraryTagsIntoImportedAssets = useCallback((
        assets: TlantiCaseAsset[],
        importedAssets: TlantiCaseAsset[] | undefined,
        libraryAsset: PublicAssetLibraryItem,
        manual = false,
    ) => {
        if (!importedAssets?.length) {
            return assets;
        }

        const importedIds = new Set(importedAssets.map((asset) => asset.id));
        return assets.map((asset) => {
            if (!importedIds.has(asset.id)) {
                return asset;
            }

            return {
                ...asset,
                tags: Array.from(new Set([...asset.tags, ...libraryAsset.autoTags, libraryAsset.semanticUsage, libraryAsset.root.toLowerCase()])),
                classificationSource: manual ? 'manual' : asset.classificationSource,
            };
        });
    }, []);

    const mergeBatchLibraryTagsIntoImportedAssets = useCallback((
        assets: TlantiCaseAsset[],
        importedAssets: TlantiCaseAsset[] | undefined,
        libraryAssets: PublicAssetLibraryItem[],
        manual = false,
    ) => {
        if (!importedAssets?.length) {
            return assets;
        }

        const assetMap = new Map(importedAssets.map((importedAsset, index) => [importedAsset.id, libraryAssets[index]]));
        return assets.map((asset) => {
            const matchedLibraryAsset = assetMap.get(asset.id);
            if (!matchedLibraryAsset) {
                return asset;
            }

            return {
                ...asset,
                tags: Array.from(new Set([...asset.tags, ...matchedLibraryAsset.autoTags, matchedLibraryAsset.semanticUsage, matchedLibraryAsset.root.toLowerCase()])),
                classificationSource: manual ? 'manual' : asset.classificationSource,
            };
        });
    }, []);

    const handleLibraryAssetAction = useCallback(async (asset: PublicAssetLibraryItem, mode: PublicAssetLibraryActionMode) => {
        if (mode === 'semantic' && asset.semanticUsage === 'dental-library') {
            setActiveToothLibraryAsset(asset);
            await showDesktopNotification('TlantiCAD Studio', `Dental library active: ${asset.name}`);
            return;
        }

        if (mode !== 'db' && mode !== 'semantic') {
            return;
        }

        try {
            const result = await importTlanticadAssetsFromAbsolutePaths(activeCase, databaseState, [asset.absolutePath]);
            if (!result) {
                return;
            }

            const nextAssets = mergeLibraryTagsIntoImportedAssets(result.assets, result.importedAssets, asset, mode === 'semantic');
            await persistClinicalAssets(nextAssets, result.caseFolderPath);
            await showDesktopNotification('TlantiCAD Studio', mode === 'semantic' ? `Semantic import applied: ${asset.name}` : `Library asset imported: ${asset.name}`);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo importar el asset de la librería al caso.');
        }
    }, [activeCase, databaseState, mergeLibraryTagsIntoImportedAssets, persistClinicalAssets]);

    const handleLibraryBatchAction = useCallback(async (assets: PublicAssetLibraryItem[], mode: PublicAssetLibraryActionMode) => {
        const libraryAsset = mode === 'semantic' ? assets.find((asset) => asset.semanticUsage === 'dental-library') : null;
        if (libraryAsset) {
            setActiveToothLibraryAsset(libraryAsset);
        }

        const importableAssets = assets.filter((asset) => mode !== 'semantic' || asset.semanticUsage !== 'dental-library');
        if (!importableAssets.length) {
            if (libraryAsset) {
                await showDesktopNotification('TlantiCAD Studio', `Dental library active: ${libraryAsset.name}`);
            }
            return;
        }

        try {
            const result = await importTlanticadAssetsFromAbsolutePaths(activeCase, databaseState, importableAssets.map((asset) => asset.absolutePath));
            if (!result) {
                return;
            }

            const nextAssets = mergeBatchLibraryTagsIntoImportedAssets(result.assets, result.importedAssets, importableAssets, mode === 'semantic');
            await persistClinicalAssets(nextAssets, result.caseFolderPath);
            await showDesktopNotification('TlantiCAD Studio', `${importableAssets.length} library assets processed in batch.`);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo completar el lote semántico de la librería.');
        }
    }, [activeCase, databaseState, mergeBatchLibraryTagsIntoImportedAssets, persistClinicalAssets]);

    const openClinicalAsset = useCallback(async (asset: NonNullable<TlantiCase['assets']>[number]) => {
        if (!activeCase.storagePath || !isTauriRuntime()) {
            return;
        }

        try {
            const absolutePath = await resolveTlanticadAssetPath(activeCase, asset);
            if (!absolutePath) {
                return;
            }
            await openPathInSystem(absolutePath);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo abrir el asset seleccionado.');
        }
    }, [activeCase.storagePath]);

    const replaceClinicalAsset = useCallback(async (asset: TlantiCaseAsset) => {
        try {
            if (isTauriRuntime()) {
                const result = await replaceTlanticadAssetFromDisk(activeCase, databaseState, asset);
                if (!result) {
                    return;
                }

                await persistClinicalAssets(result.assets, result.caseFolderPath);
                await showDesktopNotification('TlantiCAD Studio', `Asset reemplazado: ${result.asset.name}`);
                return;
            }

            setPendingAssetMutation({ mode: 'replace', asset });
            setGuidedImportRoles(null);
            setAssetFileAccept(DEFAULT_CLINICAL_IMPORT_ACCEPT);
            assetFileInputRef.current?.click();
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo reemplazar el asset clínico.');
        }
    }, [activeCase, databaseState, persistClinicalAssets]);

    const relinkClinicalAsset = useCallback(async (asset: TlantiCaseAsset) => {
        try {
            if (isTauriRuntime()) {
                const result = await relinkTlanticadAssetFromDisk(activeCase, databaseState, asset);
                if (!result) {
                    return;
                }

                await persistClinicalAssets(result.assets, result.caseFolderPath);
                await showDesktopNotification('TlantiCAD Studio', `Asset relinked: ${result.asset.name}`);
                return;
            }

            setPendingAssetMutation({ mode: 'relink', asset });
            setGuidedImportRoles(null);
            setAssetFileAccept(DEFAULT_CLINICAL_IMPORT_ACCEPT);
            assetFileInputRef.current?.click();
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo relinkear el asset clínico.');
        }
    }, [activeCase, databaseState, persistClinicalAssets]);

    const deleteClinicalAsset = useCallback(async (asset: TlantiCaseAsset) => {
        try {
            const result = await deleteTlanticadAsset(activeCase, asset);
            if (!result) {
                return;
            }

            await persistClinicalAssets(result.assets, result.caseFolderPath);
            await showDesktopNotification('TlantiCAD Studio', `Asset eliminado: ${asset.name}`);
        } catch (error) {
            console.error(error);
            await showDesktopNotification('TlantiCAD Studio', 'No se pudo eliminar el asset clínico.');
        }
    }, [activeCase, persistClinicalAssets]);

    const updateClinicalAssetMetadata = useCallback(async (assetId: string, patch: Partial<TlantiCaseAsset>) => {
        const nextAssets = (activeCase.assets ?? []).map((asset) => (asset.id === assetId ? { ...asset, ...patch } : asset));
        await persistClinicalAssets(nextAssets);
    }, [activeCase.assets, persistClinicalAssets]);

    const launchCadWorkspace = useCallback((moduleId = 'cad', label = t.design, options: { force?: boolean } = {}) => {
        const resolvedModuleId = normalizeWorkloadModuleTarget(moduleId);
        // V143 — when invoked from the import gate "Saltar e ir al módulo" branch
        // the user already acknowledged they understand the missing requirements.
        // The blocking toast was preventing the navigation entirely (BUG-004 root
        // cause). Honour `options.force` to bypass the soft gate while still
        // surfacing the warning notification.
        if (resolvedModuleId === 'cad' && !canLaunchCadModule) {
            if (!options.force) {
                toast(actionAvailability.cadBlockedReason ?? 'Completa los requisitos clínicos antes de abrir CAD.', 'info');
                void showDesktopNotification(
                    'TlantiCAD Studio',
                    actionAvailability.cadBlockedReason ?? 'Completa los requisitos clínicos antes de abrir CAD.',
                );
                if (!primaryScanReady && primaryScanChecklistItem) {
                    guideClinicalImport(primaryScanChecklistItem.roles);
                }
                return;
            }
            // Forced launch — still warn so the technician knows what's missing.
            toast(actionAvailability.cadBlockedReason ?? 'Abriendo CAD con prerequisitos pendientes.', 'info');
        }

        if (resolvedModuleId === 'dicom') {
            markActiveCasePipeline({ scan: true });
        } else if (resolvedModuleId === 'model-creator') {
            markActiveCasePipeline({ model: true });
        } else if (resolvedModuleId === 'fab') {
            markActiveCasePipeline({ manufacture: true });
        } else {
            markActiveCasePipeline({ design: true });
        }

        updateActiveCase((current) => ({
            lastOpenedModule: resolvedModuleId,
            moduleTarget: normalizeWorkloadModuleTarget(current.moduleTarget ?? resolvedModuleId),
            moduleId: resolvedModuleId,
            workloadStatus: getWorkloadStatus(current),
        }));

        if (preferences.openModulesInNewWindow) {
            void openWorkspaceWindow({
                workspace: 'tlanticad',
                caseId: activeCase.id,
                module: resolvedModuleId,
                title: `${label} · ${activeCase.caseNumber}`,
            });
            return;
        }

        onEnter({ caseId: activeCase.id, moduleId: resolvedModuleId });
    }, [activeCase.caseNumber, activeCase.id, actionAvailability.cadBlockedReason, canLaunchCadModule, guideClinicalImport, markActiveCasePipeline, onEnter, preferences.openModulesInNewWindow, primaryScanChecklistItem, primaryScanReady, t.design, toast, updateActiveCase]);

    /**
     * Module-opening choke point. Before actually entering the module, we
     * show the import-gate dialog that asks the clinician for DICOM + 3D
     * models. The user can skip explicitly (useful when data is already on
     * disk) or import first; either way the real launch happens via
     * `launchCadWorkspace`.
     */
    const openCadWorkspace = useCallback((moduleId = 'cad', label = t.design) => {
        const resolvedModuleId = normalizeWorkloadModuleTarget(moduleId);
        if (hasCadWorkspaceData) {
            launchCadWorkspace(resolvedModuleId, label, { force: true });
            return;
        }

        setPendingModuleGate({ moduleId: resolvedModuleId, label });
    }, [hasCadWorkspaceData, launchCadWorkspace, t.design]);

    const openCaseModuleFromBrowser = useCallback((caseItem: TlantiCase, moduleId: string, label: string) => {
        const resolvedModuleId = normalizeWorkloadModuleTarget(moduleId);
        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: caseItem.id,
            cases: prev.cases.map((item) => (
                item.id === caseItem.id
                    ? {
                        ...item,
                        moduleId: resolvedModuleId,
                        moduleTarget: resolvedModuleId,
                        lastOpenedModule: resolvedModuleId,
                        lastOpenedAt: new Date().toISOString(),
                        workloadStatus: getWorkloadStatus(item),
                    }
                    : item
            )),
        }));
        setIsCaseBrowserOpen(false);

        if (preferences.openModulesInNewWindow) {
            void openWorkspaceWindow({
                workspace: 'tlanticad',
                caseId: caseItem.id,
                module: resolvedModuleId,
                title: `${label} · ${caseItem.caseNumber}`,
            });
            return;
        }

        onEnter({ caseId: caseItem.id, moduleId: resolvedModuleId });
    }, [onEnter, persistDatabaseState, preferences.openModulesInNewWindow]);

    const importRequiredAssetsFromCaseBrowser = useCallback((caseItem: TlantiCase, roles: TlantiCaseAsset['role'][]) => {
        const requiredRoles = roles.length ? roles : getMissingRequiredAssets(caseItem);
        persistDatabaseState((prev) => ({
            ...prev,
            activeCaseId: caseItem.id,
            cases: prev.cases.map((item) => (
                item.id === caseItem.id
                    ? { ...item, lastOpenedAt: new Date().toISOString(), workloadStatus: getWorkloadStatus(item) }
                    : item
            )),
        }));
        setIsCaseBrowserOpen(false);

        if (caseItem.id === activeCase.id) {
            void importClinicalAssets(requiredRoles);
            return;
        }

        setGuidedImportRoles(requiredRoles);
        setAssetFileAccept(resolveClinicalImportAccept(requiredRoles));
        toast('Case activated. Use Import next asset to attach the required records.', 'info');
    }, [activeCase.id, importClinicalAssets, persistDatabaseState, toast]);

    const intakeCards = useMemo(() => [
        {
            id: 'dicom-files',
            title: 'Nuevo estudio DICOM',
            description: 'Importa `.dcm`, `.ima`, `.dicom` o `.zip` y entra directo al review clínico dentro del caso activo.',
            badge: launcherDicomRecommendation.title,
            actionLabel: 'Abrir archivos / ZIP',
            secondaryLabel: 'Abrir carpeta',
            onPrimary: () => void openLauncherDicomPicker(),
            onSecondary: () => void openLauncherDicomFolderPicker(),
            icon: Scan,
        },
        {
            id: 'existing-case',
            title: 'Abrir caso existente',
            description: 'Carga un `.tlanticad`, `case.json` o una carpeta previamente guardada y retoma el trabajo sin pasar por otro launcher.',
            badge: 'Workspace',
            actionLabel: 'Abrir caso',
            secondaryLabel: 'Crear caso vacío',
            onPrimary: () => void openCasePicker(),
            onSecondary: () => setIsCreateCaseOpen(true),
            icon: FolderOpen,
        },
        {
            id: 'library-case',
            title: 'Preparar caso desde biblioteca',
            description: 'Explora `public/library`, importa assets base y deja el caso listo para CAD, implantes o guía quirúrgica.',
            badge: activeCase.assets?.length ? `${activeCase.assets.length} assets` : 'Asset library',
            actionLabel: 'Importar assets',
            secondaryLabel: 'Seguir en CAD',
            onPrimary: () => void importClinicalAssets(),
            onSecondary: () => openCadWorkspace('cad', t.design),
            icon: Boxes,
        },
    ], [activeCase.assets?.length, importClinicalAssets, launcherDicomRecommendation.title, openCadWorkspace, openCasePicker, openLauncherDicomFolderPicker, openLauncherDicomPicker, t.design]);

    const tlantiDbDockItems = useMemo(() => [
        {
            title: 'Cases',
            onClick: () => setIsCaseBrowserOpen(true),
            icon: <FolderOpen className="size-5" />,
        },
        {
            title: 'DICOM',
            onClick: () => void openLauncherDicomPicker(),
            icon: <Scan className="size-5" />,
        },
        {
            title: 'Assets',
            onClick: () => void importClinicalAssets(),
            icon: <Boxes className="size-5" />,
        },
        {
            title: 'CAD',
            onClick: () => openCadWorkspace('cad', t.design),
            icon: <Layers className="size-5" />,
        },
        {
            title: 'Settings',
            onClick: () => setIsSettingsOpen(true),
            icon: <Settings className="size-5" />,
        },
    ], [importClinicalAssets, openCadWorkspace, openLauncherDicomPicker, t.design]);

    return (
        <div className="tl-window relative flex h-dvh w-full flex-col overflow-hidden font-sans text-text-primary">
            {isCreateCaseOpen && (
                <Suspense fallback={<LazyPanelFallback label="Loading workload wizard…" />}>
                    <TlantiDbWorkloadWizard
                        open={isCreateCaseOpen}
                        defaultCaseName={createCaseDraft.caseName}
                        defaultClientName={createCaseDraft.clientName}
                        onOpenChange={(open) => {
                            if (!open) cancelCreateCaseWizard();
                        }}
                        onCreateWorkload={handleCreateWorkload}
                    />
                </Suspense>
            )}

            {/* Settings Modal */}
            {isSettingsOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-window-bg/72">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        className="tl-glass flex max-h-[88dvh] w-[min(94vw,40rem)] flex-col gap-6 overflow-y-auto rounded-lg p-6"
                    >
                        <div className="flex justify-between items-center border-b border-glass-border pb-4">
                            <h3 className="text-xl font-display text-text-display">Settings</h3>
                            <button aria-label="Close settings" onClick={() => setIsSettingsOpen(false)} className="text-text-secondary hover:text-text-primary">✕</button>
                        </div>

                        <SettingsErrorBoundary>
                        <div className="flex flex-col gap-4">
                            <SettingsErrorBoundary>
                                <Suspense fallback={<LazyPanelFallback label="Loading runtime settings…" />}>
                                    <SystemRuntimeSettingsPanel
                                        report={systemRuntimeReport}
                                        pythonBridge={pythonBridgeStatus}
                                        loading={systemRuntimeLoading}
                                        error={systemRuntimeError}
                                        profile={performanceProfile}
                                        onRefresh={() => { void refreshSystemRuntimeReport({ autoApply: performanceProfile.mode === 'auto' }); }}
                                        onToggleAutoMode={(checked) => {
                                            setPerformanceProfile((current) => ({
                                                ...current,
                                                mode: checked ? 'auto' : 'manual',
                                                source: checked ? current.source : 'manual',
                                            }));

                                            if (checked) {
                                                void refreshSystemRuntimeReport({ autoApply: true });
                                            }
                                        }}
                                        onApplyRecommended={() => {
                                            if (!systemRuntimeReport) {
                                                return;
                                            }

                                            applyRuntimeProfile(systemRuntimeReport, performanceProfile.mode);
                                        }}
                                    />
                                </Suspense>
                            </SettingsErrorBoundary>

                            <Suspense fallback={<LazyPanelFallback label="Loading backend integration catalog…" />}>
                                <BackendIntegrationPanel
                                    catalog={backendCatalog}
                                    geometryProbe={geometryProbe}
                                    loading={backendCatalogLoading}
                                />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Scanning Rust crates workspace…" />}>
                                <BackendWorkspacePanel
                                    catalog={backendWorkspaceCatalog}
                                    loading={backendWorkspaceLoading}
                                    error={backendWorkspaceError}
                                />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading DICOM roadmap…" />}>
                                <DicomDentalRoadmapPanel />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading DICOM execution board…" />}>
                                <DicomDentalExecutionPanel />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading toolkit integrations…" />}>
                                <PlatformToolkitPanel />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading quality + interoperability gate…" />}>
                                <ClinicalQualityInteropPanel
                                    activeCase={activeCase}
                                    state={databaseState}
                                    systemRuntimeReport={systemRuntimeReport}
                                    pythonBridge={pythonBridgeStatus}
                                    backendCatalog={backendCatalog}
                                    runtimeDiagnostics={runtimeDiagnostics}
                                    runtimeDiagnosticsLoading={runtimeDiagnosticsLoading}
                                    onRefreshRuntime={() => { void refreshSystemRuntimeReport({ autoApply: performanceProfile.mode === 'auto' }); }}
                                />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading collaboration + auto-planning…" />}>
                                <CollaborationAutoplanningPanel
                                    activeCase={activeCase}
                                    state={databaseState}
                                    diagnostics={runtimeDiagnostics}
                                    onPatchCollaboration={patchActiveCaseCollaboration}
                                />
                            </Suspense>

                            <Suspense fallback={<LazyPanelFallback label="Loading hybrid ops + precision readiness…" />}>
                                <HybridOpsPrecisionPanel
                                    activeCase={activeCase}
                                    state={databaseState}
                                    operations={activeCase.operations ?? {
                                        remoteJobs: [],
                                        kernelTransition: {
                                            preference: 'mesh-first',
                                            preferredKernel: 'auto',
                                            offlineFallback: true,
                                            geometryBenchmarkScore: null,
                                            lastPolicyUpdateAt: null,
                                        },
                                    }}
                                    runtimeReport={systemRuntimeReport}
                                    backendCatalog={backendCatalog}
                                    workspaceCatalog={backendWorkspaceCatalog}
                                    diagnostics={runtimeDiagnostics}
                                    onPatchOperations={patchActiveCaseOperations}
                                />
                            </Suspense>

                            <div className="flex flex-col gap-2">
                                <label className="text-xs font-mono uppercase tracking-widest text-text-secondary">Timezone</label>
                                <Select value={timeZone} onValueChange={setTimeZone}>
                                    <SelectTrigger className="w-full bg-surface-raised">
                                        <SelectValue placeholder="Select timezone" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {groupedTimeZones.map((group, groupIndex) => (
                                            <React.Fragment key={group.label}>
                                                <SelectGroup>
                                                    <SelectLabel>{group.label}</SelectLabel>
                                                    {group.zones.map((tz) => (
                                                        <SelectItem key={tz} value={tz}>{formatTimeZoneLabel(tz)}</SelectItem>
                                                    ))}
                                                </SelectGroup>
                                                {groupIndex < groupedTimeZones.length - 1 ? <SelectSeparator /> : null}
                                            </React.Fragment>
                                        ))}
                                    </SelectContent>
                                </Select>
                                <p className="text-xs text-text-secondary text-pretty">Ahora aparecen regiones de América, Sudamérica, Europa, África, Asia y Oceanía completas.</p>
                            </div>

                            <div className="grid gap-2">
                                <label className="text-xs font-mono uppercase tracking-widest text-text-secondary">Tooth numbering</label>
                                <Select value={numberingSystem} onValueChange={(value) => setNumberingSystem(value as 'FDI' | 'UNIVERSAL')}>
                                    <SelectTrigger className="w-full bg-surface-raised">
                                        <SelectValue placeholder="Select numbering" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="FDI">FDI</SelectItem>
                                        <SelectItem value="UNIVERSAL">Universal / International</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            <div className="grid gap-2">
                                <label className="text-xs font-mono uppercase tracking-widest text-text-secondary">Asset profile</label>
                                <Select value={assetProfile} onValueChange={(value) => setAssetProfile(value as 'clinical' | 'lab' | 'demo')}>
                                    <SelectTrigger className="w-full bg-surface-raised">
                                        <SelectValue placeholder="Select asset profile" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="clinical">Clinical</SelectItem>
                                        <SelectItem value="lab">Lab</SelectItem>
                                        <SelectItem value="demo">Demo</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            <div className="grid gap-2">
                                <label className="text-xs font-mono uppercase tracking-widest text-text-secondary">Operator alias</label>
                                <input title="Operator alias" placeholder="Designer Station" value={operatorAlias} onChange={(event) => setOperatorAlias(event.target.value)} className="rounded-md border border-border bg-surface-raised px-3 py-2 text-sm text-text-primary outline-none focus:border-text-primary" />
                            </div>

                            <div className="grid gap-3 rounded-lg border border-border p-4">
                                <div className="flex items-center justify-between gap-4">
                                    <div>
                                        <p className="text-sm text-text-primary">Open modules in native system window</p>
                                        <p className="text-xs text-text-secondary">Abre CAD como ventana real de macOS/Windows dentro de Tauri.</p>
                                    </div>
                                    <Switch checked={preferences.openModulesInNewWindow} onCheckedChange={(checked) => setPreferences({ openModulesInNewWindow: checked })} aria-label="Open modules in new window" />
                                </div>
                                <div className="flex items-center justify-between gap-4">
                                    <div>
                                        <p className="text-sm text-text-primary">Sync windows</p>
                                        <p className="text-xs text-text-secondary">Broadcast case changes to parallel workspaces in real time.</p>
                                    </div>
                                    <Switch checked={preferences.syncWindows} onCheckedChange={(checked) => setPreferences({ syncWindows: checked })} aria-label="Sync case data between windows" />
                                </div>
                                <div className="flex items-center justify-between gap-4">
                                    <div>
                                        <p className="text-sm text-text-primary">Interactive odontogram</p>
                                        <p className="text-xs text-text-secondary">Use imported SVG odontogram instead of the old mock arch.</p>
                                    </div>
                                    <Switch checked={preferences.useInteractiveOdontogram} onCheckedChange={(checked) => setPreferences({ useInteractiveOdontogram: checked })} aria-label="Use interactive odontogram" />
                                </div>
                            </div>

                            <div className="grid gap-3 rounded-lg border border-border p-4">
                                <div>
                                    <p className="text-xs font-mono uppercase tracking-widest text-text-secondary">Navigation sensitivity</p>
                                    <p className="mt-1 text-xs text-text-secondary">Personalize zoom, pan and rotation response for smoother navigation.</p>
                                </div>

                                {([
                                    ['zoom', 'Zoom'],
                                    ['pan', 'Pan'],
                                    ['rotation', 'Rotation'],
                                ] as const).map(([key, label]) => (
                                    <label key={key} className="grid gap-2">
                                        <div className="flex items-center justify-between gap-3 text-xs text-text-secondary">
                                            <span>{label}</span>
                                            <span className="font-mono text-text-primary">{navigationSensitivity[key].toFixed(2)}</span>
                                        </div>
                                        <input
                                            type="range"
                                            min="0.2"
                                            max="2"
                                            step="0.05"
                                            value={navigationSensitivity[key]}
                                            onChange={(event) => setNavigationSensitivity(key, Number(event.target.value))}
                                            className="w-full accent-white"
                                        />
                                    </label>
                                ))}
                            </div>

                            <div className="grid gap-3 rounded-lg border border-border p-4">
                                <div>
                                    <p className="text-xs font-mono uppercase tracking-widest text-text-secondary">About</p>
                                    <h4 className="mt-2 text-balance text-lg font-display text-text-display">{APP_ABOUT.productName}</h4>
                                    <p className="mt-1 text-sm text-text-secondary text-pretty">By {APP_ABOUT.studio} · 2026 · derechos reservados.</p>
                                </div>

                                <div className="flex flex-wrap gap-2">
                                    <button
                                        type="button"
                                        onClick={() => void copyDiagnostics()}
                                        aria-label="Copy diagnostics"
                                        className="rounded-2xl border border-border px-4 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
                                    >
                                        Copy diagnostics
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => void createCrashReport()}
                                        aria-label="Create crash report"
                                        className="rounded-2xl border border-border px-4 py-2 text-sm text-text-primary transition-colors hover:bg-surface-raised"
                                    >
                                        Create crash report
                                    </button>
                                </div>

                                <div className="grid gap-3 md:grid-cols-2">
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Version</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{APP_ABOUT.version}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Build UID</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums break-all">{APP_ABOUT.buildUid}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3 md:col-span-2">
                                        <p className="text-[11px] uppercase text-text-secondary">Bundle identifier</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display break-all">{APP_ABOUT.bundleIdentifier}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Case UUID</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums break-all">{activeCase.id}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Build number</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{APP_ABOUT.buildNumber}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Tauri</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{runtimeDiagnostics?.tauriVersion ?? '2.10.3'}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Vite / React</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{runtimeDiagnostics ? `${runtimeDiagnostics.viteVersion} · ${runtimeDiagnostics.reactVersion}` : '6.2.0 · 19.2.3'}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Sistema</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display text-pretty">{runtimeDiagnosticsLoading ? 'Cargando…' : runtimeDiagnostics?.platform ?? 'macOS'}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3">
                                        <p className="text-[11px] uppercase text-text-secondary">Arquitectura / host</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display text-pretty">{runtimeDiagnosticsLoading ? 'Cargando…' : `${runtimeDiagnostics?.architecture ?? 'unknown'} · ${runtimeDiagnostics?.hostname ?? 'unknown'}`}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3 md:col-span-2">
                                        <p className="text-[11px] uppercase text-text-secondary">Workspace local</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display break-all">{runtimeDiagnosticsLoading ? 'Cargando…' : runtimeDiagnostics?.workspaceLocation ?? workspaceLocation}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3 md:col-span-2">
                                        <p className="text-[11px] uppercase text-text-secondary">Caso activo</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display break-all">{runtimeDiagnosticsLoading ? 'Cargando…' : runtimeDiagnostics?.caseLocation ?? activeCase.storagePath ?? 'No local case path saved yet'}</p>
                                    </div>
                                    <div className="rounded-2xl border border-border bg-surface-raised px-3 py-3 md:col-span-2">
                                        <p className="text-[11px] uppercase text-text-secondary">Fecha y zona activa</p>
                                        <p className="mt-1 text-sm font-semibold text-text-display text-pretty">{aboutTimestamp}</p>
                                    </div>
                                </div>

                                <div className="tl-panel rounded-md px-3 py-3">
                                    <p className="text-sm text-text-primary">{APP_ABOUT.studio}</p>
                                    <p className="mt-1 text-xs text-text-secondary text-pretty">{APP_ABOUT.location}</p>
                                    <p className="mt-2 text-xs text-text-secondary text-pretty">{APP_ABOUT.copyright}</p>
                                </div>
                            </div>
                        </div>
                        </SettingsErrorBoundary>

                        <div className="flex justify-end pt-4 border-t border-glass-border">
                            <button
                                onClick={() => setIsSettingsOpen(false)}
                                className="tl-control-active rounded px-4 py-2 font-mono text-xs font-bold uppercase tracking-widest transition-colors"
                            >
                                Done
                            </button>
                        </div>
                    </motion.div>
                </div>
            )}

            {/* Hidden File Input */}
            <input type="file" ref={caseFileInputRef} onChange={handleCaseFileChange} className="hidden" accept=".json,application/json" title="Open TlantiCAD case file" />
            <input type="file" ref={assetFileInputRef} onChange={handleAssetFileChange} className="hidden" multiple accept={assetFileAccept} title="Import clinical assets" />
            <input type="file" ref={launcherDicomInputRef} onChange={handleLauncherDicomChange} className="hidden" multiple accept=".zip,.dcm,.dicom,.ima" title="Import DICOM files or ZIP studies" />
            <input type="file" ref={launcherDicomDirectoryInputRef} onChange={handleLauncherDicomDirectoryChange} className="hidden" multiple accept=".dcm,.dicom,.ima" title="Import DICOM folder" {...({ webkitdirectory: 'true', directory: 'true' } as Record<string, string>)} />

            <Suspense fallback={null}>
                <TlantiDbCaseBrowserSheet
                    open={isCaseBrowserOpen}
                    cases={databaseState.cases}
                    activeCaseId={databaseState.activeCaseId}
                    onOpenChange={setIsCaseBrowserOpen}
                    onSelectCase={(caseId) => {
                        activateCase(caseId);
                        setIsCaseBrowserOpen(false);
                    }}
                    onImportCase={() => void openCasePicker()}
                    onRevealCaseFolder={(caseItem) => void revealCaseFolder(caseItem)}
                    onOpenModule={openCaseModuleFromBrowser}
                    onImportRequiredAssets={importRequiredAssetsFromCaseBrowser}
                />
            </Suspense>

            <Sheet open={isIntakeSheetOpen} onOpenChange={setIsIntakeSheetOpen}>
                <SheetContent side="bottom" className="max-h-[82dvh] overflow-y-auto border-border bg-surface p-0 text-text-primary">
                    <SheetHeader className="border-b border-border">
                        <SheetTitle className="text-text-display">Clinical intake</SheetTitle>
                        <SheetDescription>Choose the fastest entry path for {activeCase.caseNumber}.</SheetDescription>
                    </SheetHeader>
                    <div className="grid gap-3 p-4 md:grid-cols-3">
                        {intakeCards.map((card) => {
                            const Icon = card.icon;
                            return (
                                <div key={card.id} className="flex min-h-0 flex-col rounded-md border border-border bg-card p-3">
                                    <div className="flex items-start gap-3">
                                        <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-surface text-text-display">
                                            <Icon className="size-5" />
                                        </div>
                                        <div className="min-w-0 flex-1">
                                            <div className="flex flex-wrap items-center justify-between gap-2">
                                                <p className="text-sm font-semibold text-text-display">{card.title}</p>
                                                <Badge className="border border-border bg-surface text-text-secondary">{card.badge}</Badge>
                                            </div>
                                            <p className="mt-1 text-xs text-text-secondary">{card.description}</p>
                                        </div>
                                    </div>

                                    <div className="mt-auto flex flex-wrap gap-2 pt-4">
                                        <button
                                            type="button"
                                            onClick={card.onPrimary}
                                            className="inline-flex items-center gap-2 rounded-md border border-text-display bg-text-display px-3 py-1.5 text-[11px] font-medium text-black transition-colors hover:bg-white"
                                        >
                                            {card.actionLabel}
                                            <ArrowRight className="size-3.5" />
                                        </button>
                                        <button
                                            type="button"
                                            onClick={card.onSecondary}
                                            className="rounded-md border border-border px-3 py-1.5 text-[11px] text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
                                        >
                                            {card.secondaryLabel}
                                        </button>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </SheetContent>
            </Sheet>

            <Sheet open={isWorkflowSheetOpen} onOpenChange={setIsWorkflowSheetOpen}>
                <SheetContent side="right" className="w-[min(92vw,44rem)] overflow-y-auto border-border bg-surface p-0 text-text-primary sm:max-w-none">
                    <SheetHeader className="border-b border-border">
                        <SheetTitle className="text-text-display">Tooth workflow</SheetTitle>
                        <SheetDescription>Selections, materials, bridge spans and occlusion for {activeCase.caseNumber}.</SheetDescription>
                    </SheetHeader>
                    <div className="grid gap-3 p-4">
                        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                            <div className="rounded-md border border-border bg-card px-3 py-2.5">
                                <p className="text-[11px] uppercase text-text-secondary">Restorations</p>
                                <p className="text-lg font-semibold text-text-display tabular-nums">{selectedToothSummaries.length}</p>
                            </div>
                            <div className="rounded-md border border-border bg-card px-3 py-2.5">
                                <p className="text-[11px] uppercase text-text-secondary">Antagonists</p>
                                <p className="text-lg font-semibold text-text-display tabular-nums">{antagonistTeeth.length}</p>
                            </div>
                            <div className="rounded-md border border-border bg-card px-3 py-2.5">
                                <p className="text-[11px] uppercase text-text-secondary">Connectors</p>
                                <p className="text-lg font-semibold text-text-display tabular-nums">{activeConnectorCount}</p>
                            </div>
                            <div className="rounded-md border border-border bg-card px-3 py-2.5">
                                <p className="text-[11px] uppercase text-text-secondary">Occlusion</p>
                                <p className="truncate text-sm font-semibold text-text-display">{activeCase.occlusionScanType}</p>
                            </div>
                        </div>

                        <div className="rounded-md border border-border bg-surface-raised p-3">
                            <div className="mb-3 flex items-center justify-between gap-3">
                                <p className="text-sm font-semibold text-text-display">Selected teeth</p>
                                <Badge className="border border-border bg-card text-text-primary">{selectedToothSummaries.length} teeth</Badge>
                            </div>
                            {selectedToothSummaries.length ? (
                                <div className="grid gap-2 sm:grid-cols-2">
                                    {selectedToothSummaries.map((item) => (
                                        <button
                                            key={item.toothNumber}
                                            type="button"
                                            onClick={() => {
                                                setActiveToothNumber(item.toothNumber);
                                                setIsToothSheetOpen(true);
                                            }}
                                            className="flex items-center justify-between rounded-md border border-border bg-card px-3 py-2.5 text-left transition-colors hover:bg-surface"
                                        >
                                            <div>
                                                <p className="text-sm font-semibold text-text-display">{item.numberingLegend}</p>
                                                <p className="text-xs text-text-secondary">{item.label}</p>
                                            </div>
                                            <div className="text-right text-xs text-text-secondary">
                                                <p>{item.material}</p>
                                                <p>{item.shade}</p>
                                            </div>
                                        </button>
                                    ))}
                                </div>
                            ) : (
                                <div className="rounded-md border border-dashed border-border px-4 py-5 text-sm text-text-secondary">
                                    Select a tooth to define restoration, material, shade and production method.
                                </div>
                            )}
                        </div>

                        <div className="rounded-md border border-border bg-surface-raised p-3">
                            <div className="mb-3 flex items-center justify-between gap-3">
                                <p className="text-sm font-semibold text-text-display">Bridge logic</p>
                                <Badge className="border border-border bg-card text-text-primary">{connectorCandidates.length} candidates</Badge>
                            </div>
                            {connectorCandidates.length ? (
                                <div className="grid gap-2">
                                    {connectorCandidates.map((connector) => (
                                        <button
                                            key={connector.key}
                                            type="button"
                                            onClick={() => toggleConnector(connector.fromTooth, connector.toTooth)}
                                            disabled={!connector.canConnect}
                                            className={clsx(
                                                'flex items-center justify-between rounded-md border px-3 py-2.5 text-left transition-colors',
                                                connector.canConnect
                                                    ? connector.active
                                                        ? 'border-[#d4a843] bg-card hover:bg-surface'
                                                        : 'border-border bg-card hover:bg-surface'
                                                    : 'cursor-not-allowed border-border bg-card opacity-60',
                                            )}
                                        >
                                            <div>
                                                <p className="text-sm font-semibold text-text-display">{connector.fromTooth} - {connector.toTooth}</p>
                                                <p className="text-xs text-text-secondary">{connector.reason}{connector.span.length ? ` · span ${connector.span.join(', ')}` : ''}</p>
                                            </div>
                                            <Badge className="border border-border bg-surface text-text-primary">
                                                {connector.canConnect ? (connector.active ? 'Connected' : connector.auto ? 'Auto' : 'Manual') : 'Blocked'}
                                            </Badge>
                                        </button>
                                    ))}
                                </div>
                            ) : (
                                <div className="rounded-md border border-dashed border-border px-4 py-5 text-sm text-text-secondary">
                                    No bridge connectors available yet. Use pontics or omit in bridge to create spans.
                                </div>
                            )}
                        </div>
                    </div>
                </SheetContent>
            </Sheet>

            {isPristineCase && !caseId ? (
                <DentalDbWelcomeLauncher
                    currentTime={currentTime}
                    language={language}
                    timeZone={timeZone}
                    recentCases={recentCases}
                    labQueueStats={labQueueStats}
                    onCreateCase={() => setIsCreateCaseOpen(true)}
                    onOpenCase={() => setIsCaseBrowserOpen(true)}
                    onActivateCase={activateCase}
                />
            ) : (
                <>
            <div className="flex min-h-0 flex-1 overflow-hidden bg-black text-text-primary">
                <motion.aside
                    initial={false}
                    animate={{ width: isLeftPanelOpen ? 380 : 0, opacity: isLeftPanelOpen ? 1 : 0 }}
                    className="flex shrink-0 flex-col overflow-hidden whitespace-nowrap border-r border-border bg-surface"
                >
                    <div className="flex h-full min-w-[380px] flex-col gap-6 overflow-y-auto p-6">
                        <div className="flex flex-col gap-4">
                            <div className="flex items-start justify-between gap-4">
                                <div className="min-w-0">
                                    <h1 className="text-2xl font-display tracking-tight text-text-display">Hola DENTAL DESIGNER</h1>
                                    <p className="mt-1 truncate text-[11px] font-mono uppercase tracking-widest text-text-secondary">{activeCase.caseNumber} · {caseStatusLabel}</p>
                                </div>
                            </div>

                            <div className="flex flex-col gap-1 text-[10px] font-mono uppercase tracking-widest text-text-secondary">
                                <span>{currentTime.toLocaleDateString(language, { weekday: 'short', year: 'numeric', month: 'short', day: 'numeric' })}</span>
                                <span>{currentTime.toLocaleTimeString(language, { timeZone })} · {timeZone}</span>
                            </div>

                            <div className="flex gap-2">
                                <button type="button" onClick={() => setIsCreateCaseOpen(true)} className="rounded border border-border-visible p-2 text-text-secondary transition-colors hover:border-text-primary hover:text-text-primary" title={t.new}><FilePlus size={16} strokeWidth={1.5} /></button>
                                <button type="button" onClick={() => setIsCaseBrowserOpen(true)} className="rounded border border-border-visible p-2 text-text-secondary transition-colors hover:border-text-primary hover:text-text-primary" title={t.load}><FolderOpen size={16} strokeWidth={1.5} /></button>
                                <button type="button" onClick={() => void saveWorkspaceSnapshot()} className="rounded border border-border-visible p-2 text-text-secondary transition-colors hover:border-text-primary hover:text-text-primary" title={t.save}><Copy size={16} strokeWidth={1.5} /></button>
                                <button type="button" onClick={handleDuplicateCase} className="rounded border border-border-visible p-2 text-text-secondary transition-colors hover:border-text-primary hover:text-text-primary" title={t.duplicate}><Copy size={16} strokeWidth={1.5} /></button>
                            </div>
                        </div>

                        <div className="text-xs font-mono uppercase tracking-widest text-accent">SELECT NEXT ACTION IN TOOLBAR</div>

                        <div className="flex flex-col gap-6">
                            <div className="flex gap-4">
                                <label className="group flex-1">
                                    <span className="mb-2 flex items-center gap-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary"><User size={12} strokeWidth={1.5} /> {t.client}</span>
                                    <input value={activeCase.clientName} onChange={(event) => setClientName(event.target.value)} className="w-full border-b border-border-visible bg-transparent pb-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display" />
                                </label>
                                <label className="group w-24">
                                    <span className="mb-2 block text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary">ID</span>
                                    <input value={activeCase.clientId} onChange={(event) => setClientId(event.target.value)} className="w-full border-b border-border-visible bg-transparent pb-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display" />
                                </label>
                            </div>

                            <label className="group">
                                <span className="mb-2 flex items-center gap-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary"><User size={12} strokeWidth={1.5} /> {t.name}</span>
                                <input value={activeCase.name} onChange={(event) => setProjectName(event.target.value)} className="w-full border-b border-border-visible bg-transparent pb-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display" />
                            </label>

                            <div className="flex gap-4">
                                <label className="group flex-1">
                                    <span className="mb-2 flex items-center gap-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary"><User size={12} strokeWidth={1.5} /> {t.technician}</span>
                                    <input value={activeCase.technicianName} onChange={(event) => setTechnicianName(event.target.value)} className="w-full border-b border-border-visible bg-transparent pb-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display" />
                                </label>
                                <label className="group w-24">
                                    <span className="mb-2 block text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary">ID</span>
                                    <input value={activeCase.technicianId} onChange={(event) => setTechnicianId(event.target.value)} className="w-full border-b border-border-visible bg-transparent pb-2 text-sm font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display" />
                                </label>
                            </div>
                        </div>

                        <div className="group flex min-h-[150px] flex-1 flex-col border border-border">
                            <div className="flex items-center justify-between border-b border-border p-3 transition-colors group-focus-within:border-text-primary">
                                <span className="flex items-center gap-2 text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary"><FilePlus size={12} strokeWidth={1.5} /> {t.notes}</span>
                                <button type="button" onClick={() => setIsWorkflowSheetOpen(true)} className="text-[11px] font-mono uppercase text-text-secondary hover:text-text-primary">Open</button>
                            </div>
                            <textarea value={activeCase.notes} onChange={(event) => setNotes(event.target.value)} className="h-full resize-none bg-transparent p-4 text-sm leading-relaxed text-text-secondary outline-none transition-colors focus:text-text-primary" />
                        </div>

                        <div className="flex h-[240px] flex-col border border-border">
                            <div className="flex border-b border-border text-[11px] font-mono uppercase tracking-widest">
                                <button type="button" onClick={() => setActiveSidebarPanel('assets')} className="border-b-2 border-text-display px-4 py-3 text-text-display">3D PREVIEW</button>
                                <button type="button" onClick={() => setIsIntakeSheetOpen(true)} className="px-4 py-3 text-text-secondary transition-colors hover:text-text-primary">IMAGES/DOCS</button>
                            </div>
                            <div className="relative flex flex-1 items-center justify-center bg-surface-raised">
                                <div className="flex h-32 w-24 items-center justify-center border border-border-visible">
                                    <Boxes className="text-text-disabled" size={32} strokeWidth={1} />
                                </div>
                                <div className="absolute inset-x-4 bottom-4 flex gap-2">
                                    <button type="button" onClick={() => void importClinicalAssets()} className="flex flex-1 items-center border border-border bg-surface px-3 py-2 text-[11px] font-mono uppercase tracking-widest text-text-primary">
                                        <span className="mr-2 size-2 bg-success" /> {activeCase.assets?.length ?? 0} ASSETS
                                    </button>
                                    <button type="button" onClick={() => setIsCaseBrowserOpen(true)} className="flex items-center gap-2 border border-border bg-surface px-3 py-2 text-[11px] font-mono uppercase tracking-widest text-text-primary transition-colors hover:border-text-primary">
                                        <FolderOpen size={12} strokeWidth={1.5} /> OPEN
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </motion.aside>

                <main className="relative flex min-w-0 flex-1 flex-col bg-black">
                    <button type="button" onClick={() => setIsLeftPanelOpen(!isLeftPanelOpen)} className="absolute left-4 top-8 z-20 rounded-lg border border-border bg-surface p-2 text-text-secondary transition-colors hover:text-text-primary" aria-label="Toggle case panel">
                        <Layers size={16} />
                    </button>

                    <div className="flex items-center justify-between gap-6 p-8 pl-16">
                        <h2 className="truncate text-4xl font-display tracking-tight text-text-display">{t.indication_materials}</h2>
                        <div className="flex items-center gap-4 border border-border px-4 py-2">
                            <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">{t.patient}</span>
                            <Switch checked={activeCase.multiDie} onCheckedChange={(checked) => updateActiveCase({ multiDie: checked })} aria-label="Toggle multi-die mode" />
                            <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">{t.multidie}</span>
                        </div>
                    </div>

                    <div id="db-odontogram" className="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden px-8">
                        <div className="flex h-full min-h-[24rem] w-full max-w-[58rem] items-center justify-center">
                            <Suspense fallback={<LazyPanelFallback label="Loading interactive odontogram…" />}>
                                <InteractiveOdontogram
                                    toothMap={activeCase.toothMap}
                                    numberingSystem={numberingSystem}
                                    connectors={connectorCandidates}
                                    onToothClick={toggleTooth}
                                    onToothHoverChange={setHoveredToothNumber}
                                    className="max-h-full"
                                />
                            </Suspense>
                        </div>

                        <div className="pointer-events-none absolute left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col gap-3">
                            <div className="flex items-center gap-3">
                                <div className="size-4 bg-accent" />
                                <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Antagonist</span>
                            </div>
                            <div className="flex items-center gap-3">
                                <div className="size-4 bg-text-display" />
                                <span className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Anatomic Coping</span>
                            </div>
                            <div className="mt-4 text-center text-[11px] font-mono uppercase tracking-widest text-text-disabled">SHIFT+CLICK FOR ANTAGONIST</div>
                        </div>

                        {hoveredToothNumber ? (
                            <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} className="absolute bottom-4 left-1/2 -translate-x-1/2 border border-border bg-surface/90 px-4 py-3 text-center">
                                <p className="text-[11px] font-mono uppercase text-text-secondary">Hovered tooth</p>
                                <p className="text-lg font-semibold text-text-display">{getToothNumberingLegend(hoveredToothNumber, numberingSystem)}</p>
                                <p className="text-xs text-text-secondary">{hoveredToothState?.selected ? getRestorationLabel(hoveredToothState.restorationType) : 'Available for selection'}</p>
                            </motion.div>
                        ) : null}
                    </div>

                    <div className="grid grid-cols-1 gap-8 p-8 md:grid-cols-2">
                        <label className="group min-w-0">
                            <span className="mb-2 block text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary">COLORS</span>
                            <select
                                value={activeCase.materialShade}
                                onChange={(event) => {
                                    const value = event.target.value;
                                    updateActiveCase((current) => {
                                        const nextToothMap = { ...current.toothMap };
                                        Object.entries(nextToothMap).forEach(([key, tooth]) => {
                                            if (tooth?.selected && !tooth.shade) {
                                                nextToothMap[key] = { ...tooth, shade: value };
                                            }
                                        });
                                        return { materialShade: value, toothMap: nextToothMap };
                                    });
                                }}
                                className="w-full appearance-none border-b border-border-visible bg-transparent pb-2 text-base font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display"
                            >
                                {['A1', 'A2', 'A3', 'B1', 'C1'].map((shade) => <option key={shade} value={shade} className="bg-surface text-text-primary">{shade}</option>)}
                            </select>
                        </label>
                        <label className="group min-w-0">
                            <span className="mb-2 block text-[11px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-focus-within:text-text-primary">OCCLUSION SCAN</span>
                            <select value={activeCase.occlusionScanType} onChange={(event) => updateActiveCase({ occlusionScanType: event.target.value })} className="w-full appearance-none border-b border-border-visible bg-transparent pb-2 text-base font-mono text-text-primary outline-none transition-colors focus:border-text-primary focus:text-text-display">
                                <option value="two_models" className="bg-surface text-text-primary">Two models in occlusion</option>
                                <option value="single_model" className="bg-surface text-text-primary">Single model</option>
                                <option value="bite_registration" className="bg-surface text-text-primary">Bite registration</option>
                            </select>
                        </label>
                    </div>

                    <ClinicalOrderFlowPanel
                        activeCase={activeCase}
                        onImportMissingAssets={(roles) => void importClinicalAssets(roles)}
                        onOpenWorkflow={() => setIsWorkflowSheetOpen(true)}
                    />
                </main>

                <aside className="relative z-10 flex w-[280px] shrink-0 flex-col border-l border-border bg-surface">
                    <div className="flex items-center justify-between border-b border-border p-4">
                        <h2 className="text-xl font-display tracking-tight text-text-display">{t.actions}</h2>
                        <div className="flex items-center gap-3">
                            <button type="button" onClick={() => setLanguage(language === 'en' ? 'es' : 'en')} className="text-[10px] font-mono uppercase tracking-widest text-text-secondary transition-colors hover:text-text-primary">{language}</button>
                            <button type="button" onClick={() => setThemeMode(themeMode === 'dark' ? 'light' : 'dark')} className="text-text-secondary transition-colors hover:text-text-primary" title="Toggle Theme">{themeMode === 'dark' ? <Sun size={14} strokeWidth={1.5} /> : <Moon size={14} strokeWidth={1.5} />}</button>
                            <button type="button" onClick={() => setIsSettingsOpen(true)} className="text-text-secondary transition-colors hover:text-text-primary" title="Settings"><Settings size={14} strokeWidth={1.5} /></button>
                        </div>
                    </div>

                    <motion.div className="flex flex-1 flex-col gap-1 overflow-y-auto p-4" initial="hidden" animate="show" variants={{ hidden: { opacity: 0 }, show: { opacity: 1, transition: { staggerChildren: 0.04 } } }}>
                        <motion.button variants={{ hidden: { opacity: 0, y: 10 }, show: { opacity: 1, y: 0 } }} type="button" onClick={() => openCadWorkspace('cad', t.design)} className="mb-2 flex items-center justify-center gap-3 rounded-lg border border-border bg-text-display p-3 text-left text-black shadow-[0_0_15px_rgba(255,255,255,0.12)] transition-transform hover:scale-[1.02] active:scale-[0.98]">
                            <span className="text-[12px] font-mono font-bold uppercase tracking-widest">{t.design}</span>
                        </motion.button>

                        {TLANTIDB_WORKSPACE_MODULES.map((item) => {
                            const Icon = item.icon;
                            return (
                                <motion.button
                                    key={item.module}
                                    variants={{ hidden: { opacity: 0, x: -10 }, show: { opacity: 1, x: 0 } }}
                                    type="button"
                                    onClick={() => openCadWorkspace(item.module, item.label)}
                                    className="group relative flex items-center gap-4 overflow-hidden rounded-lg border border-transparent p-3 text-left transition-all duration-200 hover:border-border hover:bg-surface-raised"
                                >
                                    <Icon size={18} className="relative z-10 text-text-secondary transition-colors group-hover:text-text-display" strokeWidth={1.5} />
                                    <span className="relative z-10 text-[12px] font-mono uppercase tracking-widest text-text-secondary transition-colors group-hover:text-text-primary">{item.label}</span>
                                </motion.button>
                            );
                        })}

                        <div className="my-2 h-px bg-border" />
                        <button type="button" onClick={() => void copyActiveCaseReference()} className="flex items-center gap-4 rounded-lg p-3 text-left transition-colors hover:bg-surface-raised"><Share2 size={18} className="text-text-secondary" /><span className="text-[12px] font-mono uppercase tracking-widest text-text-secondary">{t.share}</span></button>
                        <button type="button" onClick={handleDuplicateCase} className="flex items-center gap-4 rounded-lg p-3 text-left transition-colors hover:bg-surface-raised"><Copy size={18} className="text-text-secondary" /><span className="text-[12px] font-mono uppercase tracking-widest text-text-secondary">{t.copy}</span></button>
                        <button type="button" onClick={() => void printActiveCase()} className="flex items-center gap-4 rounded-lg p-3 text-left transition-colors hover:bg-surface-raised"><Printer size={18} className="text-text-secondary" /><span className="text-[12px] font-mono uppercase tracking-widest text-text-secondary">{t.print}</span></button>
                    </motion.div>

                    <div className="flex items-center justify-between border-t border-border bg-surface p-4">
                        <div className="text-xl font-display tracking-tight text-text-display">TLANTI<span className="text-text-secondary">DB</span></div>
                        <div className="text-[10px] font-mono uppercase tracking-widest text-text-disabled">v{APP_ABOUT.version}</div>
                    </div>
                </aside>
            </div>
                </>
            )}

            <Suspense fallback={null}>
                <ToothWorkflowSheet
                    open={isToothSheetOpen}
                    toothNumber={activeToothNumber}
                    numberingSystem={numberingSystem}
                    toothState={activeToothState}
                    libraryAsset={activeToothLibraryAsset}
                    onOpenChange={setIsToothSheetOpen}
                    onPatchTooth={applyToothPatch}
                    onApplyLibraryDefaults={(toothNumber) => {
                        if (!activeToothLibraryAsset) {
                            return;
                        }
                        applyToothPatch(toothNumber, inferDentalLibraryDefaults(activeToothLibraryAsset).patch);
                    }}
                    onClearTooth={clearTooth}
                />
            </Suspense>

            <ModuleImportGateDialog
                open={pendingModuleGate !== null}
                moduleId={pendingModuleGate?.moduleId ?? 'cad'}
                moduleLabel={pendingModuleGate?.label ?? t.design}
                caseNumber={activeCase.caseNumber}
                caseName={activeCase.name}
                summary={moduleImportGateSummary}
                onImportDicomFiles={() => void openLauncherDicomPicker()}
                onImportDicomFolder={() => void openLauncherDicomFolderPicker()}
                onImportModels={() => void importClinicalAssets(['prep-scan', 'antagonist-scan', 'bite-registration'])}
                onDropFiles={(files) => void importDroppedAssets(files)}
                onContinue={() => {
                    const launch = pendingModuleGate;
                    setPendingModuleGate(null);
                    // V143 — explicit skip from the gate must always navigate, even
                    // if the soft prerequisites are not met (BUG-004).
                    if (launch) launchCadWorkspace(launch.moduleId, launch.label, { force: true });
                }}
                onCancel={() => setPendingModuleGate(null)}
            />

            <ToothWorkDefinitionDialog
                open={isToothWorkDialogOpen}
                toothNumber={activeToothNumber}
                numberingSystem={numberingSystem}
                toothState={activeToothState}
                onCancel={() => setIsToothWorkDialogOpen(false)}
                onClear={(toothNumber) => {
                    clearTooth(toothNumber);
                    setIsToothWorkDialogOpen(false);
                }}
                onConfirm={(toothNumber, patch) => {
                    const productionMethod: DentalProductionMethod =
                        patch.productionMethod === '3-axis-milling' ? 'inhouse-milling' :
                        patch.productionMethod === '5-axis-laser-3dprint' ? 'five-axis-milling' :
                        'outsourced-center';
                    const implantMode: DentalImplantMode =
                        patch.implantOption === 'custom-abutment' ? 'custom-abutment' :
                        patch.implantOption === 'screw-retained' ? 'screw-retained' :
                        patch.implantOption === 'on-stock-abutment' ? 'stock-abutment' :
                        'none';
                    applyToothPatch(toothNumber, {
                        restorationType: patch.legacyRestorationType,
                        material: patch.material,
                        shade: patch.shade,
                        productionMethod,
                        implantMode,
                        // V174 — keep legacy booleans in sync for older readers but the
                        // structured `additionalScans` is the new source of truth.
                        usePreOpModel: patch.preOpModel,
                        useExtraGingivaScan: patch.extraGingivaScan,
                        additionalScans: {
                            preOpModel: patch.preOpModel,
                            extraGingiva: patch.extraGingivaScan,
                            substructureScan: patch.substructureScan,
                            waxup: patch.waxup,
                        },
                        minimalThicknessMm: patch.minimalThicknessMm,
                        cementGapMm: patch.cementGapMm,
                        workTypeId: patch.workTypeId,
                        workTimeMinutes: patch.workTimeMinutes,
                        biteSplintMode: patch.biteSplintMode,
                        biteSplintAntagonistScan: patch.biteSplintAntagonistScan,
                    });
                    setIsToothWorkDialogOpen(false);
                }}
            />

        </div>
    );
};

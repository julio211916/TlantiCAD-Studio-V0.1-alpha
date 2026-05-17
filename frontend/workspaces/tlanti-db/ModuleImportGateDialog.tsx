/**
 * ModuleImportGateDialog — shown before opening any CAD / DICOM / Implant
 * module when the active case has no primary data yet.
 *
 * Asks the clinician to bring in:
 *   1. DICOM/CBCT → folder of files, ZIP archive, or individual .dcm/.ima
 *   2. 3D models (optional for DICOM-only work) → .stl, .obj, .ply, .glb
 *
 * The dialog never starts imports itself; it just collects intent and
 * fires the parent's loaders (`onImportDicomFiles`, `onImportDicomFolder`,
 * `onImportAssets`). The parent keeps existing progress / toast handling.
 *
 * Clicking "Skip and continue" opens the module without any data — useful
 * for exploratory sessions or when assets are already on disk.
 */

import React, { useCallback, useMemo, useRef } from 'react';
import {
    ArrowRight,
    FolderOpen,
    HelpCircle,
    Layers,
    Scan,
    UploadCloud,
    X,
} from 'lucide-react';

import { Button } from '@/components/ui/button';

export interface ModuleImportGateSummary {
    dicomAssetCount: number;
    modelAssetCount: number;
    hasAnyData: boolean;
}

interface ModuleImportGateDialogProps {
    open: boolean;
    moduleId: string;
    moduleLabel: string;
    caseNumber: string;
    caseName: string;
    summary: ModuleImportGateSummary;
    onImportDicomFiles: () => void;
    onImportDicomFolder: () => void;
    onImportModels: () => void;
    onDropFiles: (files: File[]) => void;
    onContinue: () => void;
    onCancel: () => void;
}

const DICOM_ACCEPT = '.dcm,.DCM,.ima,.IMA,.dicom,.DICOM,.zip,.ZIP';
const MODEL_ACCEPT = '.stl,.STL,.obj,.OBJ,.ply,.PLY,.glb,.GLB,.gltf,.GLTF';

export function ModuleImportGateDialog({
    open,
    moduleId,
    moduleLabel,
    caseNumber,
    caseName,
    summary,
    onImportDicomFiles,
    onImportDicomFolder,
    onImportModels,
    onDropFiles,
    onContinue,
    onCancel,
}: ModuleImportGateDialogProps) {
    const dicomInputRef = useRef<HTMLInputElement>(null);
    const modelInputRef = useRef<HTMLInputElement>(null);
    const imageInputRef = useRef<HTMLInputElement>(null);
    const documentInputRef = useRef<HTMLInputElement>(null);

    const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'copy';
    }, []);

    const handleDrop = useCallback(
        (e: React.DragEvent<HTMLDivElement>) => {
            e.preventDefault();
            const files = Array.from(e.dataTransfer.files ?? []);
            if (files.length > 0) onDropFiles(files);
        },
        [onDropFiles],
    );

    const handleLocalFiles = useCallback(
        (list: FileList | null) => {
            if (!list || list.length === 0) return;
            onDropFiles(Array.from(list));
        },
        [onDropFiles],
    );

    const moduleHint = useMemo(() => {
        switch (moduleId) {
            case 'dicom':
                return 'Para revisar escaneos CBCT o tomografías. Acepta carpeta completa, ZIP o archivos .dcm sueltos.';
            case 'implant':
            case 'guide':
                return 'Importa CBCT + escaneos de arcada para planificación protética-quirúrgica.';
            case 'model-creator':
            case 'partials':
                return 'Importa modelos digitales (.stl/.obj/.ply) del maxilar y antagonista para generar el modelo.';
            case 'splint':
                return 'Modelos superior e inferior + registro oclusal para diseñar la férula.';
            case 'orthocad':
            case 'smile-design':
                return 'Foto + modelo 3D para diseño de sonrisa / ortodoncia digital.';
            case 'fab':
                return 'Los STL/OBJ definitivos se cargan aquí para preparar manufactura.';
            default:
                return 'Importa los datos clínicos necesarios para este módulo antes de comenzar.';
        }
    }, [moduleId]);

    if (!open) return null;

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="module-import-gate-title"
            className="fixed inset-0 z-[120] flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
            onClick={(event) => {
                if (event.target === event.currentTarget) onCancel();
            }}
        >
            <div className="relative flex w-[min(46rem,100%)] max-h-[92vh] flex-col overflow-hidden rounded-2xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-start gap-3 border-b border-border px-5 py-4">
                    <div className="flex size-10 items-center justify-center rounded-xl border border-border bg-surface-sunken">
                        <Layers className="size-5 text-sky-300" aria-hidden />
                    </div>
                    <div className="flex-1">
                        <p className="text-[11px] font-mono uppercase tracking-wider text-text-secondary">
                            {caseNumber} · Abrir módulo
                        </p>
                        <h2
                            id="module-import-gate-title"
                            className="text-balance text-xl font-semibold text-text-display"
                        >
                            {moduleLabel}
                        </h2>
                        <p className="mt-1 text-sm text-text-secondary">{moduleHint}</p>
                    </div>
                    <button
                        type="button"
                        onClick={onCancel}
                        aria-label="Cerrar sin abrir módulo"
                        className="text-text-secondary hover:text-text-primary"
                    >
                        <X className="size-5" />
                    </button>
                </header>

                <div
                    className="flex flex-col gap-4 overflow-y-auto p-5"
                    onDragOver={handleDragOver}
                    onDrop={handleDrop}
                >
                    <div className="flex items-center gap-2 rounded-md border border-dashed border-border bg-surface-sunken/60 px-3 py-2 text-xs text-text-secondary">
                        <UploadCloud className="size-4 shrink-0" aria-hidden />
                        <span>
                            Puedes arrastrar archivos (DCM, ZIP, STL, OBJ, PLY…) sobre esta ventana.
                        </span>
                    </div>

                    <section className="rounded-xl border border-border bg-surface-sunken/40 p-4">
                        <header className="flex items-center gap-2">
                            <Scan className="size-4 text-sky-300" aria-hidden />
                            <h3 className="text-sm font-semibold text-text-primary">
                                DICOM / CBCT
                            </h3>
                            <span className="ml-auto rounded-full bg-surface-raised px-2 py-0.5 text-[10px] uppercase tracking-wider text-text-secondary">
                                {summary.dicomAssetCount > 0
                                    ? `${summary.dicomAssetCount} ya importados`
                                    : 'Sin datos todavía'}
                            </span>
                        </header>
                        <p className="mt-1 text-xs text-text-secondary">
                            Carpeta completa, ZIP, o archivos `.dcm` / `.ima` individuales.
                        </p>
                        <div className="mt-3 flex flex-wrap gap-2">
                            <Button
                                type="button"
                                variant="secondary"
                                size="sm"
                                onClick={() => {
                                    onImportDicomFiles();
                                    dicomInputRef.current?.click();
                                }}
                            >
                                <UploadCloud className="size-4" />
                                <span className="ml-2">Archivos / ZIP</span>
                            </Button>
                            <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                onClick={onImportDicomFolder}
                            >
                                <FolderOpen className="size-4" />
                                <span className="ml-2">Carpeta DICOM</span>
                            </Button>
                            <input
                                ref={dicomInputRef}
                                type="file"
                                multiple
                                accept={DICOM_ACCEPT}
                                className="hidden"
                                onChange={(e) => {
                                    handleLocalFiles(e.currentTarget.files);
                                    e.currentTarget.value = '';
                                }}
                            />
                        </div>
                    </section>

                    <section className="rounded-xl border border-border bg-surface-sunken/40 p-4">
                        <header className="flex items-center gap-2">
                            <Layers className="size-4 text-emerald-300" aria-hidden />
                            <h3 className="text-sm font-semibold text-text-primary">
                                Modelos 3D
                            </h3>
                            <span className="ml-auto rounded-full bg-surface-raised px-2 py-0.5 text-[10px] uppercase tracking-wider text-text-secondary">
                                {summary.modelAssetCount > 0
                                    ? `${summary.modelAssetCount} ya importados`
                                    : 'Sin datos todavía'}
                            </span>
                        </header>
                        <p className="mt-1 text-xs text-text-secondary">
                            STL, OBJ, PLY, GLB. Maxilar, mandíbula, antagonista, mordida, bite.
                        </p>
                        <div className="mt-3 flex flex-wrap gap-2">
                            <Button
                                type="button"
                                variant="secondary"
                                size="sm"
                                onClick={() => {
                                    onImportModels();
                                    modelInputRef.current?.click();
                                }}
                            >
                                <UploadCloud className="size-4" />
                                <span className="ml-2">Modelos STL / OBJ</span>
                            </Button>
                            <input
                                ref={modelInputRef}
                                type="file"
                                multiple
                                accept={MODEL_ACCEPT}
                                className="hidden"
                                onChange={(e) => {
                                    handleLocalFiles(e.currentTarget.files);
                                    e.currentTarget.value = '';
                                }}
                            />
                        </div>
                    </section>

                    {/* V47 — RealGUIDE: dedicated Images + Documents import buttons */}
                    <section className="rounded-md border border-border bg-surface-sunken px-4 py-3">
                        <header className="flex items-center gap-2">
                            <h3 className="text-sm font-semibold text-text-primary">
                                Imágenes 2D
                            </h3>
                        </header>
                        <p className="mt-1 text-xs text-text-secondary">
                            JPG / PNG / BMP — fotos del paciente, panorex 2D, referencias de
                            color.
                        </p>
                        <div className="mt-3 flex flex-wrap gap-2">
                            <Button
                                type="button"
                                variant="secondary"
                                size="sm"
                                onClick={() => imageInputRef.current?.click()}
                            >
                                <UploadCloud className="size-4" />
                                <span className="ml-2">Importar imágenes 2D</span>
                            </Button>
                            <input
                                ref={imageInputRef}
                                type="file"
                                multiple
                                accept="image/jpeg,image/png,image/bmp,image/webp"
                                className="hidden"
                                onChange={(e) => {
                                    handleLocalFiles(e.currentTarget.files);
                                    e.currentTarget.value = '';
                                }}
                            />
                        </div>
                    </section>

                    <section className="rounded-md border border-border bg-surface-sunken px-4 py-3">
                        <header className="flex items-center gap-2">
                            <h3 className="text-sm font-semibold text-text-primary">
                                Documentos
                            </h3>
                        </header>
                        <p className="mt-1 text-xs text-text-secondary">
                            PDF, recetas, notas clínicas, informes.
                        </p>
                        <div className="mt-3 flex flex-wrap gap-2">
                            <Button
                                type="button"
                                variant="secondary"
                                size="sm"
                                onClick={() => documentInputRef.current?.click()}
                            >
                                <UploadCloud className="size-4" />
                                <span className="ml-2">Importar documentos</span>
                            </Button>
                            <input
                                ref={documentInputRef}
                                type="file"
                                multiple
                                accept="application/pdf,text/plain,.doc,.docx,.txt,.rtf"
                                className="hidden"
                                onChange={(e) => {
                                    handleLocalFiles(e.currentTarget.files);
                                    e.currentTarget.value = '';
                                }}
                            />
                        </div>
                    </section>

                    <div className="flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/5 px-3 py-2 text-[11px] text-amber-100">
                        <HelpCircle className="size-3.5 shrink-0" aria-hidden />
                        <span>
                            Si ya importaste los datos desde el Workspace, puedes continuar al módulo.
                        </span>
                    </div>
                </div>

                <footer className="flex items-center justify-between gap-2 border-t border-border px-5 py-3">
                    <Button type="button" variant="ghost" size="sm" onClick={onCancel}>
                        Cancelar
                    </Button>
                    <Button
                        type="button"
                        variant="default"
                        size="sm"
                        onClick={onContinue}
                    >
                        <span>
                            {summary.hasAnyData ? 'Continuar al módulo' : 'Saltar e ir al módulo'}
                        </span>
                        <ArrowRight className="ml-2 size-4" />
                    </Button>
                </footer>
            </div>
        </div>
    );
}

import React from 'react';

import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import { DicomMetadata, ThemeMode } from '@/types';

import { TlantiOhifRenderMode, TlantiOhifRenderStatus } from './DicomViewerRenderStatus';

interface TlantiOhifStudyPanelProps {
    metadata?: DicomMetadata;
    renderMode: TlantiOhifRenderMode;
    themeMode: ThemeMode;
}

const studyRows = (metadata?: DicomMetadata) => [
    { label: 'Paciente', value: metadata?.patientName || 'No identificado' },
    { label: 'Modalidad', value: metadata?.modality || 'DICOM' },
    { label: 'Serie', value: metadata?.seriesDescription || 'Serie local' },
    { label: 'Fecha', value: metadata?.studyDate || '—' },
    { label: 'Slices', value: String(metadata?.sliceCount ?? 1) },
    { label: 'Resolución', value: metadata?.dimensions || 'Stack local' },
];

export function TlantiOhifStudyPanel({ metadata, renderMode, themeMode }: TlantiOhifStudyPanelProps) {
    const warnings = metadata?.warnings ?? [];

    return (
        <aside
            className={cn(
                'flex h-full min-h-0 flex-col overflow-hidden rounded-[1.35rem] border shadow-xl',
                themeMode === 'dark' ? 'border-border bg-surface-raised/95 text-text-primary' : 'border-border bg-surface/95 text-text-primary',
            )}
        >
            <div className="flex items-start justify-between gap-3 px-4 py-4">
                <div>
                    <p className="text-[11px] text-text-secondary tabular-nums">Contexto clínico</p>
                    <h3 className="mt-1 text-balance text-sm font-semibold text-text-display">Resumen del estudio</h3>
                </div>
                <TlantiOhifRenderStatus mode={renderMode} />
            </div>

            <Separator />

            <div className="grid grid-cols-2 gap-x-3 gap-y-4 px-4 py-4 text-sm">
                {studyRows(metadata).map((row) => (
                    <div key={row.label} className="min-w-0 space-y-1">
                        <p className="text-[11px] text-text-secondary">{row.label}</p>
                        <p className="truncate text-text-primary tabular-nums">{row.value}</p>
                    </div>
                ))}
            </div>

            {warnings.length ? (
                <>
                    <Separator />
                    <div className="space-y-3 px-4 py-4">
                        <div className="flex items-center justify-between gap-2">
                            <p className="text-[11px] text-text-secondary">Avisos del estudio</p>
                            <Badge variant="outline" className="rounded-full border-amber-400/30 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-300">
                                {warnings.length}
                            </Badge>
                        </div>
                        <div className="space-y-2">
                            {warnings.map((warning) => (
                                <div key={warning} className="rounded-2xl border border-border bg-card px-3 py-2 text-sm text-text-secondary text-pretty">
                                    {warning}
                                </div>
                            ))}
                        </div>
                    </div>
                </>
            ) : null}

            <div className="mt-auto px-4 pb-4">
                <div className="rounded-2xl border border-border bg-card px-3 py-3">
                    <p className="text-[11px] text-text-secondary">Ruta activa</p>
                    <p className="mt-1 text-sm text-pretty text-text-primary">
                        Revisión radiológica limpia para navegar slices, ajustar DICOM y validar el estudio antes del planning CAD.
                    </p>
                </div>
            </div>
        </aside>
    );
}
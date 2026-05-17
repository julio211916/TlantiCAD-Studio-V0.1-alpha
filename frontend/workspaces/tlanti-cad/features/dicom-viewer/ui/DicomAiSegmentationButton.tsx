/**
 * DicomAiSegmentationButton — compact trigger for the segmentation flow.
 *
 * Reads the study capability (volumetric + slice count) from props and
 * renders a disabled tooltip reason when the study isn't suitable.
 */

import React from 'react';
import { Brain, Loader2 } from 'lucide-react';

import { Button } from '@/components/ui/button';

import type { DicomStudy } from '../domain/dicom-study';
import { resolveAvailableModes } from '../domain/viewer-mode';

interface DicomAiSegmentationButtonProps {
    study: DicomStudy | null;
    isRunning: boolean;
    onTrigger: (study: DicomStudy) => void;
}

function resolveDisabledReason(study: DicomStudy | null): string | null {
    if (!study) return 'Import a DICOM study first.';
    const primary =
        study.series.find((series) => series.isVolumetric && series.instanceCount >= 16) ??
        study.series[0];
    if (!primary) return 'Active study has no series.';
    const modes = resolveAvailableModes({
        sliceCount: primary.instanceCount,
        isVolumetric: primary.isVolumetric,
        modality: primary.modality,
    });
    const aiMode = modes.find((m) => m.mode === 'ai-report');
    return aiMode?.available ? null : aiMode?.disabledReason ?? 'AI not available for this study.';
}

export function DicomAiSegmentationButton({
    study,
    isRunning,
    onTrigger,
}: DicomAiSegmentationButtonProps) {
    const disabledReason = resolveDisabledReason(study);
    const disabled = disabledReason !== null || isRunning;

    return (
        <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={disabled}
            title={disabledReason ?? 'Run AI segmentation on the active series'}
            onClick={() => {
                if (study && !disabled) onTrigger(study);
            }}
        >
            {isRunning ? (
                <Loader2 className="size-4 animate-spin" aria-hidden />
            ) : (
                <Brain className="size-4" aria-hidden />
            )}
            <span className="ml-2">AI Segmentation</span>
        </Button>
    );
}

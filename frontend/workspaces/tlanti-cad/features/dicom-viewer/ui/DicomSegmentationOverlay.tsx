/**
 * DicomSegmentationOverlay — surfaces progress, stdout and result of a
 * running segmentation job. Rendered inside the DICOM viewer viewport as
 * a dismissable drawer; does NOT occupy the full canvas.
 *
 * No cornerstone imports here. Purely presentational; the parent owns
 * the use-case and passes snapshots in.
 */

import React, { useEffect, useRef } from 'react';
import { X, Brain, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-react';

import { Button } from '@/components/ui/button';

import type { SegmentationJob } from '../application/segment-study.use-case';

interface DicomSegmentationOverlayProps {
    job: SegmentationJob | null;
    onDismiss: () => void;
    onCancel: (jobId: string) => void;
}

function formatProgress(progress: number): string {
    return `${Math.round(progress * 100)}%`;
}

function StatusIcon({ status }: { status: SegmentationJob['status'] }) {
    switch (status) {
        case 'succeeded':
            return <CheckCircle2 className="size-4 text-emerald-400" aria-hidden />;
        case 'failed':
        case 'cancelled':
            return <AlertTriangle className="size-4 text-red-400" aria-hidden />;
        case 'running':
        case 'queued':
        default:
            return <Loader2 className="size-4 animate-spin text-sky-400" aria-hidden />;
    }
}

export function DicomSegmentationOverlay({
    job,
    onDismiss,
    onCancel,
}: DicomSegmentationOverlayProps) {
    const logRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (logRef.current) {
            logRef.current.scrollTop = logRef.current.scrollHeight;
        }
    }, [job?.stdoutTail.length, job?.stderrTail.length]);

    if (!job) return null;

    const running = job.status === 'running' || job.status === 'queued';
    const failed = job.status === 'failed' || job.status === 'cancelled';

    return (
        <div
            role="dialog"
            aria-modal="false"
            aria-labelledby="segmentation-overlay-title"
            className="pointer-events-auto absolute right-4 top-4 z-30 flex w-[min(22rem,90vw)] flex-col gap-3 rounded-lg border border-border bg-surface-raised/95 p-4 shadow-xl backdrop-blur"
        >
            <header className="flex items-center gap-2">
                <Brain className="size-4 text-sky-400" aria-hidden />
                <h3
                    id="segmentation-overlay-title"
                    className="flex-1 text-sm font-semibold text-text-primary"
                >
                    AI Segmentation
                </h3>
                <button
                    type="button"
                    onClick={onDismiss}
                    aria-label="Dismiss segmentation overlay"
                    className="text-text-secondary hover:text-text-primary"
                >
                    <X className="size-4" />
                </button>
            </header>

            <div className="flex items-center gap-2 text-xs text-text-secondary">
                <StatusIcon status={job.status} />
                <span className="capitalize">{job.status}</span>
                <span aria-hidden>•</span>
                <span>{formatProgress(job.progress)}</span>
            </div>

            <div
                className="h-1.5 overflow-hidden rounded-full bg-surface-sunken"
                role="progressbar"
                aria-valuenow={Math.round(job.progress * 100)}
                aria-valuemin={0}
                aria-valuemax={100}
            >
                <div
                    className={
                        failed
                            ? 'h-full bg-red-400/70'
                            : job.status === 'succeeded'
                              ? 'h-full bg-emerald-400/80'
                              : 'h-full bg-sky-400/70'
                    }
                    style={{ width: `${Math.max(2, Math.round(job.progress * 100))}%` }}
                />
            </div>

            {job.error ? (
                <p className="rounded-md border border-red-500/40 bg-red-500/10 px-2 py-1.5 text-xs text-red-100">
                    {job.error}
                </p>
            ) : null}

            <div
                ref={logRef}
                className="max-h-36 overflow-y-auto rounded-md border border-border bg-surface-sunken px-2 py-1.5 font-mono text-[0.6875rem] leading-snug text-text-secondary"
            >
                {job.stdoutTail.length === 0 && job.stderrTail.length === 0 ? (
                    <p className="italic">Waiting for backend output…</p>
                ) : (
                    <>
                        {job.stdoutTail.map((line, i) => (
                            <div key={`out-${i}`}>{line}</div>
                        ))}
                        {job.stderrTail.map((line, i) => (
                            <div key={`err-${i}`} className="text-red-200/80">
                                {line}
                            </div>
                        ))}
                    </>
                )}
            </div>

            <footer className="flex gap-2">
                {running ? (
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => onCancel(job.jobId)}
                    >
                        Cancel
                    </Button>
                ) : null}
                <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    onClick={onDismiss}
                    className="ml-auto"
                >
                    Close
                </Button>
            </footer>
        </div>
    );
}

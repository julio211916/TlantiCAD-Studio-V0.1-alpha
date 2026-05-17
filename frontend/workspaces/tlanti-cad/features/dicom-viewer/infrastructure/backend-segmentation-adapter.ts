/**
 * SegmentationBackendAdapter — talks HTTP to the FastAPI backend.
 *
 * Backend contract (documented in vendor-integration.mdx):
 *   POST /clinical-vendors/jobs/launch { tool_id: 'AMASSS_CLI', cli_args: [...] }
 *        → { jobId, status, stdoutTail, stderrTail, ... }
 *   GET  /clinical-vendors/jobs/{job_id} → same DTO
 *
 * The adapter translates that to the domain `SegmentationJob` type.
 * This HTTP adapter is an explicit opt-in bridge for a local loopback
 * sidecar. Clinical launch and preview flow should prefer Tauri handles.
 */

import type {
    SegmentationBackendPort,
    SegmentationJob,
    SegmentationJobStatus,
} from '../application/segment-study.use-case';
import { backendUrl } from '@/lib/backend-config';

interface JobDTO {
    jobId: string;
    label: string;
    status: string;
    createdAt: number;
    startedAt: number | null;
    finishedAt: number | null;
    exitCode: number | null;
    error: string | null;
    outputPaths: string[];
    stdoutTail: string[];
    stderrTail: string[];
    metadata: Record<string, unknown>;
}

function normaliseStatus(raw: string): SegmentationJobStatus {
    switch (raw) {
        case 'queued':
        case 'running':
        case 'succeeded':
        case 'failed':
        case 'cancelled':
            return raw;
        default:
            return 'failed';
    }
}

function dtoToJob(dto: JobDTO): SegmentationJob {
    const segmentationUrl =
        dto.outputPaths.find((path) => path.endsWith('.nii.gz')) ?? null;
    // Progress estimate from stdout lines — nnU-Net prints `Progress: x/y`
    let progress = 0;
    for (let i = dto.stdoutTail.length - 1; i >= 0; i -= 1) {
        const match = dto.stdoutTail[i].match(/Progress:\s*(\d+)\s*\/\s*(\d+)/i);
        if (match) {
            const current = Number.parseInt(match[1], 10);
            const total = Number.parseInt(match[2], 10);
            if (Number.isFinite(current) && Number.isFinite(total) && total > 0) {
                progress = Math.min(1, current / total);
            }
            break;
        }
    }
    if (dto.status === 'succeeded') progress = 1;

    return {
        jobId: dto.jobId,
        status: normaliseStatus(dto.status),
        progress,
        error: dto.error,
        labels: [], // Filled by a subsequent /dicom/segmentation/stats call (V16.1)
        segmentationUrl,
        stdoutTail: [...dto.stdoutTail],
        stderrTail: [...dto.stderrTail],
    };
}

export function createBackendSegmentationAdapter(options?: {
    enabled?: boolean;
    fetcher?: typeof fetch;
}): SegmentationBackendPort {
    const fetcher = options?.fetcher ?? fetch;

    function assertCapability() {
        if (!options?.enabled) {
            throw new Error('Local HTTP segmentation adapter is disabled. Use the Tauri DICOM job adapter for clinical preview.');
        }
    }

    async function postJson<T>(path: string, body: unknown): Promise<T> {
        assertCapability();
        const response = await fetcher(backendUrl(path), {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!response.ok) {
            const detail = await response.text().catch(() => response.statusText);
            throw new Error(`POST ${path} failed: ${response.status} ${detail}`);
        }
        return (await response.json()) as T;
    }

    async function getJson<T>(path: string): Promise<T | null> {
        assertCapability();
        const response = await fetcher(backendUrl(path));
        if (response.status === 404) return null;
        if (!response.ok) {
            const detail = await response.text().catch(() => response.statusText);
            throw new Error(`GET ${path} failed: ${response.status} ${detail}`);
        }
        return (await response.json()) as T;
    }

    return {
        async launch({ studyInstanceUID, seriesInstanceUID, vendor }) {
            // For V15/V16 the only wired vendor is AMASSS_CLI; nnU-Net
            // direct-import path lands in V16.1.
            const toolId = vendor === 'amass' ? 'AMASSS_CLI' : 'AMASSS_CLI';
            const dto = await postJson<JobDTO>('/clinical-vendors/jobs/launch', {
                vendor: 'slicer_automated_dental_tools',
                tool_id: toolId,
                cli_args: [
                    '--study-uid',
                    studyInstanceUID,
                    '--series-uid',
                    seriesInstanceUID,
                ],
            });
            return dtoToJob(dto);
        },

        async poll(jobId) {
            const dto = await getJson<JobDTO>(`/clinical-vendors/jobs/${jobId}`);
            return dto ? dtoToJob(dto) : null;
        },

        async cancel(jobId) {
            await postJson(`/clinical-vendors/jobs/${encodeURIComponent(jobId)}/cancel`, {});
        },
    };
}

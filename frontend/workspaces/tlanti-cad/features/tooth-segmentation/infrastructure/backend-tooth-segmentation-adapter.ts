/**
 * HTTP adapter for the crown segmentation port. Talks to the same
 * `/clinical-vendors/jobs/*` endpoints the DICOM segmentation adapter uses,
 * but targets the `tooth_group_network` vendor.
 */

import type {
    CrownSegJob,
    CrownSegJobStatus,
    CrownSegmentationLaunchArgs,
    ToothSegmentationPort,
} from '../application/tooth-segmentation-port';
import { BACKEND_ORIGIN } from '../../../lib/backend-config';

const DEFAULT_BACKEND_ORIGIN = BACKEND_ORIGIN;

interface JobDTO {
    jobId: string;
    status: string;
    stdoutTail: string[];
    stderrTail: string[];
    error: string | null;
    outputPaths: string[];
    metadata: Record<string, unknown>;
}

function normaliseStatus(raw: string): CrownSegJobStatus {
    return raw === 'queued' ||
        raw === 'running' ||
        raw === 'succeeded' ||
        raw === 'failed' ||
        raw === 'cancelled'
        ? raw
        : 'failed';
}

/** TGN prints lines like `tooth 23: done (conf=0.88)` — parse for progress. */
function parseProgress(stdout: string[]): { progress: number; teeth: number[] } {
    const teeth = new Set<number>();
    for (const line of stdout) {
        const m = line.match(/tooth\s+(\d{2})/i);
        if (m) {
            const fdi = Number.parseInt(m[1], 10);
            if (Number.isFinite(fdi)) teeth.add(fdi);
        }
    }
    // Heuristic: a full arch has up to 16 teeth. Clamp to 1.0 on success.
    const progress = Math.min(1, teeth.size / 16);
    return { progress, teeth: Array.from(teeth).sort((a, b) => a - b) };
}

function dtoToJob(dto: JobDTO): CrownSegJob {
    const { progress, teeth } = parseProgress(dto.stdoutTail);
    return {
        jobId: dto.jobId,
        status: normaliseStatus(dto.status),
        progress: dto.status === 'succeeded' ? 1 : progress,
        segmentedTeeth: teeth,
        error: dto.error,
        stdoutTail: [...dto.stdoutTail],
        stderrTail: [...dto.stderrTail],
    };
}

export function createBackendToothSegmentationAdapter(options?: {
    origin?: string;
    fetcher?: typeof fetch;
}): ToothSegmentationPort {
    const origin = options?.origin ?? DEFAULT_BACKEND_ORIGIN;
    const fetcher = options?.fetcher ?? fetch;

    async function post<T>(path: string, body: unknown): Promise<T> {
        const res = await fetcher(`${origin}${path}`, {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            const detail = await res.text().catch(() => res.statusText);
            throw new Error(`POST ${path} failed: ${res.status} ${detail}`);
        }
        return (await res.json()) as T;
    }

    async function get<T>(path: string): Promise<T | null> {
        const res = await fetcher(`${origin}${path}`);
        if (res.status === 404) return null;
        if (!res.ok) {
            const detail = await res.text().catch(() => res.statusText);
            throw new Error(`GET ${path} failed: ${res.status} ${detail}`);
        }
        return (await res.json()) as T;
    }

    return {
        async launch({ jaw, extractGingiva, keepSegmented, skipTeeth, scanRef }: CrownSegmentationLaunchArgs) {
            const cliArgs: string[] = [
                '--arch',
                jaw === 'maxilla' ? 'upper' : 'lower',
            ];
            if (extractGingiva) cliArgs.push('--extract-gingiva');
            if (keepSegmented) cliArgs.push('--keep-segmented');
            if (skipTeeth && skipTeeth.length > 0) {
                cliArgs.push('--skip-teeth', skipTeeth.join(','));
            }
            if (scanRef.kind === 'mesh') {
                cliArgs.push('--input', scanRef.path);
            } else {
                cliArgs.push('--study-uid', scanRef.uid);
            }

            const dto = await post<JobDTO>('/clinical-vendors/jobs/launch', {
                vendor: 'tooth_group_network',
                tool_id: 'TGN_INFERENCE',
                cli_args: cliArgs,
            });
            return dtoToJob(dto);
        },

        async poll(jobId) {
            const dto = await get<JobDTO>(`/clinical-vendors/jobs/${jobId}`);
            return dto ? dtoToJob(dto) : null;
        },

        async cancel(_jobId) {
            // Backend cancel not implemented yet (shared V15 limitation).
        },
    };
}

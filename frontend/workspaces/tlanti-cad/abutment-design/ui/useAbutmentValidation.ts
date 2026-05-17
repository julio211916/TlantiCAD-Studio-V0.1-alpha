/**
 * useAbutmentValidation — V218.
 *
 * Debounced live-validation against /cad/abutment/validate. Returns
 * the latest issues + a derived `blockNext` flag (any error severity).
 * Skips when no outputPath is available — the abutment must have been
 * generated at least once before validation kicks in.
 */

import { useEffect, useRef, useState } from 'react';

import {
    createBackendAbutmentAdapter,
} from '../infrastructure/backend-abutment-adapter';
import type {
    AbutmentPort,
    AbutmentValidationIssue,
} from '../application/abutment-port';

export interface UseAbutmentValidationInput {
    outputPath: string | null;
    minThicknessMm: number;
    angulatedScrewChannelDeg: number;
    implantLibraryMaxDeg?: number;
    /** Debounce delay (ms). Default 320ms. */
    debounceMs?: number;
    /** Inject a custom port (tests). */
    port?: AbutmentPort;
}

export interface UseAbutmentValidationResult {
    issues: AbutmentValidationIssue[];
    isValidating: boolean;
    /** True when at least one issue has severity=error. Block 'Next'. */
    blockNext: boolean;
    error: string | null;
    /** Last backend label ("trimesh" / "mock"). */
    backend: string | null;
}

export function useAbutmentValidation(
    input: UseAbutmentValidationInput,
): UseAbutmentValidationResult {
    const portRef = useRef<AbutmentPort | null>(null);
    if (portRef.current === null) {
        portRef.current = input.port ?? createBackendAbutmentAdapter();
    }

    const [issues, setIssues] = useState<AbutmentValidationIssue[]>([]);
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [backend, setBackend] = useState<string | null>(null);
    const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        if (!input.outputPath) {
            setIssues([]);
            setError(null);
            return;
        }
        if (debounceRef.current) clearTimeout(debounceRef.current);
        const delay = input.debounceMs ?? 320;
        debounceRef.current = setTimeout(() => {
            let cancelled = false;
            setIsValidating(true);
            setError(null);
            portRef
                .current!.validate({
                    outputPath: input.outputPath!,
                    minThicknessMm: input.minThicknessMm,
                    angulatedScrewChannelDeg: input.angulatedScrewChannelDeg,
                    implantLibraryMaxDeg: input.implantLibraryMaxDeg,
                })
                .then((res) => {
                    if (cancelled) return;
                    setIssues(res.issues);
                    setBackend(res.backend);
                    setIsValidating(false);
                })
                .catch((err) => {
                    if (cancelled) return;
                    setError(err instanceof Error ? err.message : String(err));
                    setIsValidating(false);
                });
            return () => {
                cancelled = true;
            };
        }, delay);
        return () => {
            if (debounceRef.current) clearTimeout(debounceRef.current);
        };
    }, [
        input.outputPath,
        input.minThicknessMm,
        input.angulatedScrewChannelDeg,
        input.implantLibraryMaxDeg,
        input.debounceMs,
    ]);

    return {
        issues,
        isValidating,
        blockNext: issues.some((i) => i.severity === 'error'),
        error,
        backend,
    };
}

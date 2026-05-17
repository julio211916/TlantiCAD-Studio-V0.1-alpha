/**
 * Stateful hook that wraps the Margin Line port's 3 operations
 * (detect / correct / repair) and exposes React-friendly flags.
 */

import { useCallback, useRef, useState } from 'react';

import type {
    MarginCorrectInput,
    MarginDetectInput,
    MarginLine,
    MarginRepairInput,
} from '../domain/margin-line';
import type { MarginDetectionPort } from '../application/margin-detection-port';

interface UseMarginDetectionResult {
    margin: MarginLine | null;
    isBusy: boolean;
    error: string | null;
    detect: (input: MarginDetectInput) => Promise<void>;
    correct: (input: MarginCorrectInput) => Promise<void>;
    repair: (input: MarginRepairInput) => Promise<void>;
    clear: () => void;
}

export function useMarginDetection(
    portFactory: () => MarginDetectionPort,
): UseMarginDetectionResult {
    const portRef = useRef<MarginDetectionPort | null>(null);
    if (portRef.current === null) portRef.current = portFactory();

    const [margin, setMargin] = useState<MarginLine | null>(null);
    const [isBusy, setIsBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const runOp = useCallback(
        async <T>(op: () => Promise<MarginLine>): Promise<void> => {
            setIsBusy(true);
            setError(null);
            try {
                const result = await op();
                setMargin(result);
            } catch (err) {
                setError(err instanceof Error ? err.message : String(err));
            } finally {
                setIsBusy(false);
            }
        },
        [],
    );

    const detect = useCallback(
        (input: MarginDetectInput) => {
            if (!portRef.current) return Promise.resolve();
            return runOp(() => portRef.current!.detect(input));
        },
        [runOp],
    );

    const correct = useCallback(
        (input: MarginCorrectInput) => {
            if (!portRef.current) return Promise.resolve();
            return runOp(() => portRef.current!.correct(input));
        },
        [runOp],
    );

    const repair = useCallback(
        (input: MarginRepairInput) => {
            if (!portRef.current) return Promise.resolve();
            return runOp(() => portRef.current!.repair(input));
        },
        [runOp],
    );

    const clear = useCallback(() => {
        setMargin(null);
        setError(null);
    }, []);

    return { margin, isBusy, error, detect, correct, repair, clear };
}

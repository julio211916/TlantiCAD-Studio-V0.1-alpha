/**
 * useCommandPalette — subscribes to registry + exposes open/close, ranked
 * results, and the Cmd/Ctrl+K global hotkey.
 */

import { useCallback, useEffect, useMemo, useState } from 'react';

import { commandRegistry } from '../application/command-registry';
import {
    type CommandAction,
    type CommandMatch,
    rankActions,
} from '../domain/command-action';

export interface UseCommandPaletteResult {
    open: boolean;
    query: string;
    setQuery: (query: string) => void;
    results: CommandMatch[];
    recents: CommandAction[];
    openPalette: () => void;
    closePalette: () => void;
    runAction: (action: CommandAction) => Promise<void>;
}

export function useCommandPalette(): UseCommandPaletteResult {
    const [open, setOpen] = useState(false);
    const [query, setQuery] = useState('');
    const [tick, setTick] = useState(0);

    useEffect(() => {
        return commandRegistry.subscribe(() => setTick((value) => value + 1));
    }, []);

    useEffect(() => {
        const onKey = (event: KeyboardEvent) => {
            const isMeta = event.metaKey || event.ctrlKey;
            if (isMeta && event.key.toLowerCase() === 'k') {
                event.preventDefault();
                setOpen((value) => !value);
                setQuery('');
                return;
            }
            if (event.key === 'Escape' && open) {
                event.preventDefault();
                setOpen(false);
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open]);

    // All actions live through the registry; re-derive on tick.
    const actions = useMemo(() => commandRegistry.list(), [tick]);
    const results = useMemo(() => rankActions(query, actions), [actions, query]);
    const recents = useMemo(() => commandRegistry.recent(), [tick, open]);

    const runAction = useCallback(async (action: CommandAction) => {
        commandRegistry.markUsed(action.id);
        setOpen(false);
        setQuery('');
        await action.run();
    }, []);

    return {
        open,
        query,
        setQuery,
        results,
        recents,
        openPalette: () => setOpen(true),
        closePalette: () => setOpen(false),
        runAction,
    };
}

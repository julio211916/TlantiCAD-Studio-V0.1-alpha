/**
 * WorkspaceVisibilityTree — replicates exocad "Show/Hide" panel (imagen #48).
 * Color-coded rows, per-group toggle + transparent, Show all / Auto Hide.
 */

import React, { useCallback, useEffect, useState } from 'react';

import { AppIcon } from '../../app-icons';
import {
    autoHideOpposing,
    DEFAULT_VISIBILITY_TREE,
    showAll as showAllHelper,
    toggleGroupTransparent,
    toggleGroupVisibility,
    type VisibilityGroup,
} from '../domain/visibility-group';

interface WorkspaceVisibilityTreeProps {
    tree?: VisibilityGroup[];
    onChange?: (tree: VisibilityGroup[]) => void;
    /** When the camera is looking at the opposite jaw, Auto Hide kicks in. */
    cameraFromOpposite?: boolean;
}

export function WorkspaceVisibilityTree({
    tree: externalTree,
    onChange,
    cameraFromOpposite = false,
}: WorkspaceVisibilityTreeProps) {
    const [tree, setTree] = useState<VisibilityGroup[]>(externalTree ?? DEFAULT_VISIBILITY_TREE);
    const [autoHide, setAutoHide] = useState(false);
    const [tempShowAll, setTempShowAll] = useState(false);

    const update = useCallback(
        (next: VisibilityGroup[]) => {
            setTree(next);
            onChange?.(next);
        },
        [onChange],
    );

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.altKey && !tempShowAll) setTempShowAll(true);
        };
        const onKeyUp = (e: KeyboardEvent) => {
            if (!e.altKey) setTempShowAll(false);
        };
        window.addEventListener('keydown', onKeyDown);
        window.addEventListener('keyup', onKeyUp);
        return () => {
            window.removeEventListener('keydown', onKeyDown);
            window.removeEventListener('keyup', onKeyUp);
        };
    }, [tempShowAll]);

    useEffect(() => {
        if (autoHide) update(autoHideOpposing(tree, cameraFromOpposite));
        // Intentional: only run when flags change.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [autoHide, cameraFromOpposite]);

    const showing = tempShowAll ? tree.map((g) => ({ ...g, visible: true })) : tree;

    return (
        <aside
            role="region"
            aria-label="Show / Hide groups"
            className="pointer-events-auto flex w-56 flex-col gap-1 rounded-md border border-border bg-violet-950/80 p-2 text-slate-100 shadow-lg backdrop-blur"
            data-visual-qa="workspace-visibility-tree"
        >
            <header className="flex items-center gap-2 px-1 text-[10px] uppercase tracking-wider">
                <AppIcon name="workflow.show-hide-groups" size={14} aria-hidden />
                <span className="flex-1 font-semibold">Show / Hide</span>
                <button
                    type="button"
                    onClick={() => update(showAllHelper(tree))}
                    className="rounded bg-orange-500/20 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-orange-300"
                >
                    Show All
                </button>
            </header>

            <ul className="flex flex-col gap-0.5">
                {showing.map((g) => (
                    <li
                        key={g.id}
                        className="flex items-center gap-2 rounded px-2 py-1.5 text-[11px]"
                        style={{ background: g.color }}
                    >
                        <button
                            type="button"
                            aria-pressed={g.visible}
                            onClick={() => update(toggleGroupVisibility(tree, g.id))}
                            title={g.visible ? 'Hide group' : 'Show group'}
                        >
                            <AppIcon name={g.visible ? 'common.eye' : 'common.eye-off'} size={14} aria-hidden />
                        </button>
                        <span className="flex-1 truncate">{g.label}</span>
                        <button
                            type="button"
                            aria-pressed={g.transparent}
                            onClick={() => update(toggleGroupTransparent(tree, g.id))}
                            title="Toggle transparency"
                            className={[
                                'rounded border border-white/30 px-1.5 py-0.5 text-[9px] uppercase',
                                g.transparent ? 'bg-white/10 text-orange-200' : 'text-slate-200/70',
                            ].join(' ')}
                        >
                            T
                        </button>
                    </li>
                ))}
            </ul>

            <footer className="mt-1 flex items-center gap-2 px-1 text-[10px]">
                <label className="flex flex-1 cursor-pointer items-center gap-1">
                    <input
                        type="checkbox"
                        checked={autoHide}
                        onChange={(e) => setAutoHide(e.target.checked)}
                        className="accent-orange-400"
                    />
                    Auto Hide
                </label>
                <span className="text-slate-400">ALT = show all</span>
            </footer>
        </aside>
    );
}

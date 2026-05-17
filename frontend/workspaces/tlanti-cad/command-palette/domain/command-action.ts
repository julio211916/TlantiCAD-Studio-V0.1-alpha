/**
 * Command Palette domain (V109).
 *
 * Every feature in the app can register `CommandAction`s into the global
 * registry; the palette hotkey (Cmd/Ctrl+K) searches across every registered
 * entry via fuzzy match.
 */

export type CommandKind =
    | 'navigation'
    | 'wizard-step'
    | 'tool'
    | 'toggle'
    | 'case'
    | 'asset'
    | 'setting'
    | 'custom';

export interface CommandAction {
    id: string;
    label: string;
    kind: CommandKind;
    keywords?: string[];
    description?: string;
    hotkey?: string;
    group?: string;
    /** Icon name in the app-icons manifest, e.g. `workflow.merge-save`. */
    icon?: string;
    run(): void | Promise<void>;
    /** Optional predicate evaluated at search time; false hides the entry. */
    available?: () => boolean;
}

export interface CommandMatch {
    action: CommandAction;
    score: number;
    matchedIndices: readonly number[];
}

/**
 * Fuzzy match — subsequence with bonuses for: start-of-word, camel-case
 * boundary, contiguous runs, and exact match. Returns null when the query
 * characters don't appear in order.
 */
export function fuzzyScore(query: string, target: string): { score: number; matchedIndices: number[] } | null {
    if (!query) return { score: 0, matchedIndices: [] };
    const q = query.toLowerCase();
    const t = target.toLowerCase();
    const matched: number[] = [];
    let ti = 0;
    let qi = 0;
    let score = 0;
    let prevIndex = -2;
    while (qi < q.length && ti < t.length) {
        if (t[ti] === q[qi]) {
            matched.push(ti);
            // Contiguous run bonus.
            if (prevIndex === ti - 1) score += 5;
            // Start-of-word bonus (space or dash or underscore or camel-case).
            const prevChar = ti > 0 ? t[ti - 1] : ' ';
            const isBoundary =
                prevChar === ' ' ||
                prevChar === '-' ||
                prevChar === '_' ||
                prevChar === '/' ||
                prevChar === '.';
            if (isBoundary) score += 4;
            const origPrev = ti > 0 ? target[ti - 1] : ' ';
            const origCur = target[ti];
            if (origPrev.toLowerCase() === origPrev && origCur.toLowerCase() !== origCur) {
                score += 3; // camelCase boundary
            }
            prevIndex = ti;
            qi += 1;
        }
        ti += 1;
    }
    if (qi < q.length) return null;
    // Length penalty — shorter targets score higher.
    score -= Math.floor(t.length / 12);
    // Exact prefix bonus.
    if (t.startsWith(q)) score += 10;
    if (t === q) score += 20;
    return { score, matchedIndices: matched };
}

export function rankActions(
    query: string,
    actions: readonly CommandAction[],
    maxResults = 25,
): CommandMatch[] {
    const q = query.trim();
    const candidates: CommandMatch[] = [];
    for (const action of actions) {
        if (action.available && !action.available()) continue;
        if (q.length === 0) {
            candidates.push({ action, score: 0, matchedIndices: [] });
            continue;
        }
        // Best-of across label, keywords, description, group.
        const haystacks = [action.label, ...(action.keywords ?? []), action.description ?? '', action.group ?? ''];
        let best: { score: number; matchedIndices: number[] } | null = null;
        for (const h of haystacks) {
            const r = fuzzyScore(q, h);
            if (r && (best === null || r.score > best.score)) best = r;
        }
        if (best !== null) candidates.push({ action, score: best.score, matchedIndices: best.matchedIndices });
    }
    candidates.sort((a, b) => b.score - a.score);
    return candidates.slice(0, maxResults);
}

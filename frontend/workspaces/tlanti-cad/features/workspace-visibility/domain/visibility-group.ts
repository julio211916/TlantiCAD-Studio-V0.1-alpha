/**
 * Workspace visibility tree (V44) — mirrors exocad "Show/Hide" groups.
 */

export type VisibilityGroupId =
    | 'antagonists'
    | 'jaw-scans'
    | 'pre-op-scans'
    | 'min-thickness'
    | 'all-other-parts'
    | 'teeth'
    | 'hidden';

export interface VisibilityGroup {
    id: VisibilityGroupId;
    label: string;
    visible: boolean;
    transparent: boolean;
    /** Hex colour for the row highlight (matches exocad doc imagen #48). */
    color: string;
    /** True for groups that can be expanded to per-item control (Teeth, Hidden). */
    expandable: boolean;
    children?: VisibilityItem[];
}

export interface VisibilityItem {
    id: string;
    label: string;
    visible: boolean;
}

export const DEFAULT_VISIBILITY_TREE: VisibilityGroup[] = [
    { id: 'antagonists', label: 'Antagonist', visible: false, transparent: false, color: 'rgba(148,163,184,0.35)', expandable: false },
    { id: 'jaw-scans', label: 'Jaw scans', visible: true, transparent: false, color: 'rgba(217,119,6,0.45)', expandable: true, children: [] },
    { id: 'pre-op-scans', label: 'Pre-op scans', visible: false, transparent: false, color: 'rgba(20,184,166,0.45)', expandable: false },
    { id: 'min-thickness', label: 'Min. thickness', visible: false, transparent: false, color: 'rgba(239,68,68,0.55)', expandable: false },
    { id: 'all-other-parts', label: 'All other parts', visible: true, transparent: false, color: 'rgba(139,92,246,0.35)', expandable: true, children: [] },
    { id: 'teeth', label: 'Teeth', visible: true, transparent: false, color: 'rgba(139,92,246,0.25)', expandable: true, children: [] },
    { id: 'hidden', label: 'Hidden', visible: false, transparent: false, color: 'rgba(71,85,105,0.35)', expandable: true, children: [] },
];

export function toggleGroupVisibility(
    tree: VisibilityGroup[],
    id: VisibilityGroupId,
): VisibilityGroup[] {
    return tree.map((g) => (g.id === id ? { ...g, visible: !g.visible } : g));
}

export function toggleGroupTransparent(
    tree: VisibilityGroup[],
    id: VisibilityGroupId,
): VisibilityGroup[] {
    return tree.map((g) => (g.id === id ? { ...g, transparent: !g.transparent } : g));
}

export function showAll(tree: VisibilityGroup[]): VisibilityGroup[] {
    return tree.map((g) => ({ ...g, visible: true }));
}

export function autoHideOpposing(
    tree: VisibilityGroup[],
    cameraFromOpposite: boolean,
): VisibilityGroup[] {
    return tree.map((g) =>
        g.id === 'antagonists' ? { ...g, visible: !cameraFromOpposite } : g,
    );
}

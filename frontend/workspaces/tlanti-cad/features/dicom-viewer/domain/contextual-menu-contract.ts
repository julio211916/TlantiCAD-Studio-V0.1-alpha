/**
 * Contextual-menu contract shared by every clinical module.
 *
 * A `ContextMenuProvider` inspects a `ContextSource` and returns the menu
 * items the user can invoke. The UI layer wires this into a single
 * `<DicomContextMenu>` component built on `@radix-ui/react-dropdown-menu`.
 *
 * Rendering rules enforced by the UI component (not here):
 *  - Menu appears pegged to pointer position (never fullscreen).
 *  - ESC and click-outside close it.
 *  - Nested submenus are allowed but shallow (max 1 level).
 */

export type ClinicalModuleId =
    | 'dicom'
    | 'implant'
    | 'guide'       // Surgical guide
    | 'splint'
    | 'fab'         // Model creator / fabrication
    | 'smile'       // Smile design
    | 'ortho'       // Orthodontics (ceph, aligners)
    | 'cad';        // Restorative CAD

export type ContextObjectKind =
    | 'implant'
    | 'abutment'
    | 'sleeve'
    | 'scan-body'
    | 'nerve'
    | 'sinus'
    | 'segmentation'
    | 'measurement'
    | 'annotation';

export type ContextSource =
    | { kind: 'viewport'; moduleId: ClinicalModuleId }
    | { kind: 'series'; moduleId: 'dicom' }
    | { kind: 'object'; moduleId: ClinicalModuleId; objectKind: ContextObjectKind; objectId: string };

export interface ContextMenuItem {
    /** Stable id — used by command palette and telemetry. */
    id: string;
    label: string;
    /** Optional lucide/tabler icon key, resolved by the UI. */
    icon?: string;
    /** Keyboard shortcut hint, e.g. `⌘ + S` or `Ctrl + Shift + R`. */
    shortcut?: string;
    /** When set, the item is rendered disabled with the explanation. */
    disabledReason?: string | null;
    /** Visual hint: 'danger' renders in red, 'primary' bold. */
    tone?: 'default' | 'primary' | 'danger';
    /** Sub-items if this is a folder. */
    children?: ContextMenuItem[];
    /** Command palette ID to dispatch on click. */
    runsCommand: string;
    /** Separator flag — items before this render divider below. */
    separator?: boolean;
}

export interface ContextMenuProvider {
    /**
     * Returns the menu items for the given source, fully resolved (already
     * knows which items are disabled and why).
     */
    resolve(source: ContextSource): ContextMenuItem[];
}

export interface ContextMenuRegistry {
    /** Register a provider for a specific module. */
    register(moduleId: ClinicalModuleId, provider: ContextMenuProvider): void;
    /** Resolve provider for a module; returns null if none registered. */
    providerFor(moduleId: ClinicalModuleId): ContextMenuProvider | null;
}

/**
 * In-memory registry used by the app shell. One instance per window.
 */
export function createContextMenuRegistry(): ContextMenuRegistry {
    const providers = new Map<ClinicalModuleId, ContextMenuProvider>();
    return {
        register(moduleId, provider) {
            providers.set(moduleId, provider);
        },
        providerFor(moduleId) {
            return providers.get(moduleId) ?? null;
        },
    };
}

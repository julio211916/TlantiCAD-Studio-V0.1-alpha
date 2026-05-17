/**
 * Pure domain types for the dental asset library (exocad-style).
 *
 * A LibraryCatalog is organised as:
 *   category (implant | teeth | pontics | …)
 *     └─ group (manufacturer / preset family)
 *          └─ items (.sdfa, .stl, .xml, preview .png, …)
 */

export type LibraryItemKind =
    | 'sdfa'
    | 'stl'
    | 'off'
    | 'xml'
    | 'png'
    | 'svg'
    | 'jpg'
    | 'bin'
    | 'partInfo'
    | string;

export interface LibraryItem {
    name: string;
    /** Public-relative path (starts with `/library/...`) so it can be fetched or <img src>-ed. */
    relPath: string;
    kind: LibraryItemKind;
}

export interface LibraryGroup {
    name: string;
    previewPath: string | null;
    items: LibraryItem[];
}

export interface LibraryCategory {
    name: string;
    groups: LibraryGroup[];
}

export interface LibraryCatalog {
    generatedAt: string;
    categories: LibraryCategory[];
}

/** Canonical categories we expect to see under `public/library/`. */
export const KNOWN_CATEGORIES = [
    'implant',
    'teeth',
    'pontics',
    'attachments',
    'bar',
    'bolts',
    'bridgesplitter',
    'articulator',
    'artiregister',
    'modelcreator',
    'movementregister',
    'nesting',
    'production',
    'prosthetictoothpresets',
    'prosthetictoothsets',
    'rendereffects',
    'retentions',
    'smiledesign',
    'visualizers',
    'controls',
    'metadata',
    'gfx',
] as const;

export type KnownCategory = (typeof KNOWN_CATEGORIES)[number];

export function findGroup(
    catalog: LibraryCatalog,
    category: string,
    groupName: string,
): LibraryGroup | null {
    const cat = catalog.categories.find((c) => c.name === category);
    if (!cat) return null;
    return cat.groups.find((g) => g.name === groupName) ?? null;
}

export function findItemByKind(group: LibraryGroup, kind: LibraryItemKind): LibraryItem | null {
    return group.items.find((i) => i.kind === kind) ?? null;
}

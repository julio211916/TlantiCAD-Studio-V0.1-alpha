/**
 * Public surface of the dental-library feature.
 *
 * Composition root exposes a lazy adapter factory so consumers that never
 * open a library panel don't pay the fetch cost.
 */

export type {
    LibraryCatalog,
    LibraryCategory,
    LibraryGroup,
    LibraryItem,
    LibraryItemKind,
    KnownCategory,
} from './domain/library-item';
export { KNOWN_CATEGORIES, findGroup, findItemByKind } from './domain/library-item';

export type { LibraryCatalogPort } from './application/library-catalog-port';

export { createStaticLibraryCatalogAdapter } from './infrastructure/static-library-catalog-adapter';

export { LibraryPicker } from './ui/LibraryPicker';

/** Singleton factory — one adapter per app is fine since it caches. */
let _default: import('./application/library-catalog-port').LibraryCatalogPort | null = null;
export async function getDefaultLibraryCatalog() {
    if (_default) return _default;
    const mod = await import('./infrastructure/static-library-catalog-adapter');
    _default = mod.createStaticLibraryCatalogAdapter();
    return _default;
}

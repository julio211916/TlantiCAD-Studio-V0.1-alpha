/**
 * Port for loading the dental asset catalog. Adapters may read from a static
 * JSON bundled in `public/library/library.json`, from a Tauri filesystem scan,
 * or from a future backend endpoint.
 */

import type { LibraryCatalog, LibraryCategory, LibraryGroup } from '../domain/library-item';

export interface LibraryCatalogPort {
    load(): Promise<LibraryCatalog>;
    listCategories(): Promise<LibraryCategory[]>;
    listGroups(category: string): Promise<LibraryGroup[]>;
    /** Returns the raw bytes for an item — used by loaders/parsers (STL, SDFA). */
    fetchItem(relPath: string): Promise<Uint8Array>;
}

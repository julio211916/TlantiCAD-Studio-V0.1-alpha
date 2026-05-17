/**
 * Static adapter — loads `/library/library.json` (produced by
 * `scripts/build-library-index.ts`) and exposes it as a LibraryCatalogPort.
 *
 * This keeps 4k+ asset paths out of the JS bundle. The JSON is fetched once
 * and cached on the adapter instance.
 */

import type { LibraryCatalogPort } from '../application/library-catalog-port';
import type {
    LibraryCatalog,
    LibraryCategory,
    LibraryGroup,
} from '../domain/library-item';

interface RawGroup {
    previewPath: string | null;
    items: Array<{ name: string; relPath: string; kind: string }>;
}

interface RawIndex {
    generatedAt: string;
    categories: Record<string, { groups: Record<string, RawGroup> }>;
}

function normalise(raw: RawIndex): LibraryCatalog {
    const categories: LibraryCategory[] = Object.entries(raw.categories).map(
        ([catName, cat]) => ({
            name: catName,
            groups: Object.entries(cat.groups).map(
                ([groupName, g]): LibraryGroup => ({
                    name: groupName,
                    previewPath: g.previewPath,
                    items: g.items.map((it) => ({
                        name: it.name,
                        relPath: it.relPath,
                        kind: it.kind,
                    })),
                }),
            ),
        }),
    );
    return { generatedAt: raw.generatedAt, categories };
}

export function createStaticLibraryCatalogAdapter(
    indexUrl = '/library/library.json',
): LibraryCatalogPort {
    let cache: Promise<LibraryCatalog> | null = null;

    async function loadOnce(): Promise<LibraryCatalog> {
        if (cache) return cache;
        cache = fetch(indexUrl, { cache: 'force-cache' })
            .then(async (res) => {
                if (!res.ok) {
                    throw new Error(
                        `library.json not found (HTTP ${res.status}). ` +
                            `Run \`bun run tlanticad:library:index\` to generate it.`,
                    );
                }
                const raw = (await res.json()) as RawIndex;
                return normalise(raw);
            })
            .catch((err) => {
                cache = null;
                throw err;
            });
        return cache;
    }

    return {
        load: loadOnce,
        async listCategories() {
            return (await loadOnce()).categories;
        },
        async listGroups(category: string) {
            const catalog = await loadOnce();
            return catalog.categories.find((c) => c.name === category)?.groups ?? [];
        },
        async fetchItem(relPath: string) {
            const res = await fetch(relPath);
            if (!res.ok) {
                throw new Error(`Library asset fetch failed: ${relPath} (HTTP ${res.status})`);
            }
            return new Uint8Array(await res.arrayBuffer());
        },
    };
}

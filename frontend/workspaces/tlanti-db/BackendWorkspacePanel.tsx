import React from 'react';
import { Braces, FolderTree, Search, TerminalSquare } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { BackendWorkspaceCatalog } from '@/lib/backend-integrations';

interface BackendWorkspacePanelProps {
  catalog: BackendWorkspaceCatalog | null;
  loading: boolean;
  error: string | null;
}

function functionTone(kind: string) {
  if (kind.startsWith('tauri-')) {
    return 'text-emerald-300';
  }

  if (kind === 'async-fn') {
    return 'text-sky-300';
  }

  return 'text-text-secondary';
}

export function BackendWorkspacePanel({ catalog, loading, error }: BackendWorkspacePanelProps) {
  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <FolderTree className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary text-balance">Rust crates workspace explorer</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Lee `apps/desktop/src-tauri/crates`, detecta crates, cuenta funciones públicas y muestra entradas reales del código Rust disponibles para futuras integraciones frontend.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline">{loading ? 'Scanning…' : `${catalog?.crateCount ?? 0} crates`}</Badge>
          {catalog ? <Badge variant="outline">{catalog.publicFunctionCount} public fns</Badge> : null}
          {catalog ? <Badge variant="outline">{catalog.tauriCommandCount} Tauri commands</Badge> : null}
        </div>
      </div>

      {error ? <p className="mt-3 text-sm text-rose-300 text-pretty">{error}</p> : null}

      {!catalog && !loading ? (
        <div className="mt-4 rounded-2xl border border-dashed border-border px-4 py-5 text-sm text-text-secondary">
          No crate workspace report loaded yet.
        </div>
      ) : null}

      {catalog ? (
        <>
          <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <div className="rounded-2xl border border-border bg-surface px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Workspace root</p>
              <p className="mt-1 break-all text-xs text-text-primary">{catalog.workspaceRoot}</p>
            </div>
            <div className="rounded-2xl border border-border bg-surface px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Rust files</p>
              <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{catalog.rustFileCount}</p>
            </div>
            <div className="rounded-2xl border border-border bg-surface px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Public functions</p>
              <p className="mt-1 text-sm font-semibold text-text-display tabular-nums">{catalog.publicFunctionCount}</p>
            </div>
            <div className="rounded-2xl border border-border bg-surface px-3 py-3">
              <p className="text-[11px] uppercase text-text-secondary">Route</p>
              <p className="mt-1 text-xs font-mono text-text-primary">{catalog.route}</p>
            </div>
          </div>

          <ScrollArea className="mt-4 h-[28rem] pr-2">
            <div className="space-y-3">
              {catalog.crates.map((crateInfo) => (
                <div key={crateInfo.relativePath} className="rounded-2xl border border-border bg-surface px-4 py-4">
                  <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <TerminalSquare className="size-4 text-text-secondary" />
                        <p className="text-sm font-semibold text-text-display">{crateInfo.packageName}</p>
                        {crateInfo.packageName !== crateInfo.name ? <Badge variant="outline">{crateInfo.name}</Badge> : null}
                      </div>
                      <p className="mt-1 break-all text-xs text-text-secondary">{crateInfo.relativePath}</p>
                      {crateInfo.description ? <p className="mt-2 text-xs text-text-primary text-pretty">{crateInfo.description}</p> : null}
                    </div>

                    <div className="flex flex-wrap items-center gap-2">
                      <Badge variant="outline">{crateInfo.rustFileCount} files</Badge>
                      <Badge variant="outline">{crateInfo.publicFunctionCount} fns</Badge>
                      <Badge variant="outline">{crateInfo.tauriCommandCount} commands</Badge>
                    </div>
                  </div>

                  {crateInfo.topFunctions.length ? (
                    <div className="mt-4 rounded-2xl border border-border bg-surface-raised px-3 py-3">
                      <div className="mb-3 flex items-center gap-2">
                        <Braces className="size-4 text-text-secondary" />
                        <p className="text-[11px] uppercase text-text-secondary">Detected function surface</p>
                      </div>
                      <div className="space-y-2">
                        {crateInfo.topFunctions.map((fn) => (
                          <div key={`${crateInfo.relativePath}:${fn.file}:${fn.line}:${fn.name}`} className="rounded-xl border border-border bg-card px-3 py-3">
                            <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
                              <div>
                                <div className="flex items-center gap-2">
                                  <p className="font-mono text-xs text-text-display">{fn.name}</p>
                                  <span className={`text-[11px] uppercase ${functionTone(fn.kind)}`}>{fn.kind}</span>
                                </div>
                                <p className="mt-1 break-all font-mono text-[11px] text-text-secondary">{fn.signature}</p>
                              </div>
                              <div className="flex items-center gap-2 text-[11px] text-text-secondary">
                                <Search className="size-3.5" />
                                <span className="break-all">{fn.file}:{fn.line}</span>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : null}
                </div>
              ))}
            </div>
          </ScrollArea>
        </>
      ) : null}
    </div>
  );
}

import React, { useMemo, useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { useReactTable, getCoreRowModel, flexRender, createColumnHelper } from '@tanstack/react-table';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Boxes, DatabaseZap, LockKeyhole, PackageCheck } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  compressToolkitPayload,
  decryptToolkitPayload,
  decompressToolkitPayload,
  encryptToolkitPayload,
  loadBackendWorkspaceCatalog,
  loadSystemRuntimeReport,
  type WorkspaceCrateInfo,
} from '@/lib/backend-integrations';
import { toolkitPayloadSchema, type ToolkitPayloadForm } from '@/lib/toolkit-stack.schemas';
import { useToolkitStackStore } from '@/stores/toolkit-stack-store';

const columnHelper = createColumnHelper<WorkspaceCrateInfo>();

export function PlatformToolkitPanel() {
  const payload = useToolkitStackStore((state) => state.payload);
  const passphrase = useToolkitStackStore((state) => state.passphrase);
  const setDraft = useToolkitStackStore((state) => state.setDraft);

  const [compressedBase64, setCompressedBase64] = useState('');
  const [encryptedCiphertext, setEncryptedCiphertext] = useState('');
  const [encryptedNonce, setEncryptedNonce] = useState('');
  const [roundtripPlaintext, setRoundtripPlaintext] = useState('');
  const [toolkitError, setToolkitError] = useState<string | null>(null);

  const form = useForm<ToolkitPayloadForm>({
    resolver: zodResolver(toolkitPayloadSchema),
    defaultValues: { payload, passphrase },
  });

  const runtimeQuery = useQuery({
    queryKey: ['toolkit', 'runtime'],
    queryFn: loadSystemRuntimeReport,
  });

  const workspaceQuery = useQuery({
    queryKey: ['toolkit', 'workspace'],
    queryFn: loadBackendWorkspaceCatalog,
  });

  const compressMutation = useMutation({ mutationFn: compressToolkitPayload });
  const decompressMutation = useMutation({ mutationFn: decompressToolkitPayload });
  const encryptMutation = useMutation({ mutationFn: ({ payload, passphrase }: ToolkitPayloadForm) => encryptToolkitPayload(payload, passphrase) });
  const decryptMutation = useMutation({ mutationFn: ({ ciphertext, nonce, passphrase }: { ciphertext: string; nonce: string; passphrase: string }) => decryptToolkitPayload(ciphertext, nonce, passphrase) });

  const columns = useMemo(() => [
    columnHelper.accessor('packageName', {
      header: 'Crate',
      cell: (info) => <span className="font-mono text-xs text-text-display">{info.getValue()}</span>,
    }),
    columnHelper.accessor('rustFileCount', { header: 'Files' }),
    columnHelper.accessor('publicFunctionCount', { header: 'Public fns' }),
    columnHelper.accessor('tauriCommandCount', { header: 'Commands' }),
  ], []);

  const table = useReactTable({
    data: workspaceQuery.data?.crates ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  const submitCompression = form.handleSubmit(async (values) => {
    try {
      setToolkitError(null);
      setDraft({ payload: values.payload, passphrase: values.passphrase, lastAction: 'compressed' });
      const nextCompressed = await compressMutation.mutateAsync(values.payload);
      setCompressedBase64(nextCompressed);
      const nextEncrypted = await encryptMutation.mutateAsync(values);
      setEncryptedCiphertext(nextEncrypted.ciphertextBase64);
      setEncryptedNonce(nextEncrypted.nonceBase64);
    } catch (error) {
      setToolkitError(error instanceof Error ? error.message : 'Compression failed.');
    }
  });

  const runRoundtrip = form.handleSubmit(async (values) => {
    try {
      setToolkitError(null);
      setDraft({ payload: values.payload, passphrase: values.passphrase, lastAction: 'decrypted' });
      const nextCompressed = compressedBase64 || await compressMutation.mutateAsync(values.payload);
      const nextPlain = await decompressMutation.mutateAsync(nextCompressed);
      const encrypted = encryptedCiphertext && encryptedNonce
        ? { ciphertextBase64: encryptedCiphertext, nonceBase64: encryptedNonce }
        : await encryptMutation.mutateAsync(values);
      const nextDecrypted = await decryptMutation.mutateAsync({
        ciphertext: encrypted.ciphertextBase64,
        nonce: encrypted.nonceBase64,
        passphrase: values.passphrase,
      });
      setCompressedBase64(nextCompressed);
      setRoundtripPlaintext(`${nextPlain}\n\n--- decrypted ---\n${nextDecrypted}`);
    } catch (error) {
      setToolkitError(error instanceof Error ? error.message : 'Roundtrip failed.');
    }
  });

  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <PackageCheck className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary">Installed platform toolkit</h4>
          </div>
          <p className="mt-2 text-sm text-text-secondary text-pretty">
            Zustand, Zod, React Hook Form, React Query y TanStack Table ya están montados; además se exponen utilidades Tauri para compresión LZMA y cifrado AES.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">React Query</Badge>
          <Badge variant="outline">Zustand</Badge>
          <Badge variant="outline">Zod / RHF</Badge>
          <Badge variant="outline">TanStack Table</Badge>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">GPU profile</p>
          <p className="mt-1 text-sm font-semibold text-text-display">{runtimeQuery.data?.system.capabilities.recommendedRenderQuality ?? 'loading…'}</p>
          <p className="mt-1 text-xs text-text-secondary">{runtimeQuery.data?.system.gpus[0]?.backend ?? 'No backend yet'}</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Workspace crates</p>
          <p className="mt-1 text-sm font-semibold text-text-display">{workspaceQuery.data?.crateCount ?? 0}</p>
          <p className="mt-1 text-xs text-text-secondary">{workspaceQuery.data?.publicFunctionCount ?? 0} public functions</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Compression</p>
          <p className="mt-1 text-sm font-semibold text-text-display">LZMA base64</p>
          <p className="mt-1 text-xs text-text-secondary">Desktop command ready</p>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Crypto</p>
          <p className="mt-1 text-sm font-semibold text-text-display">AES-256-GCM</p>
          <p className="mt-1 text-xs text-text-secondary">SHA-256 key derivation</p>
        </div>
      </div>

      <form className="mt-4 grid gap-3" onSubmit={submitCompression}>
        <div className="grid gap-2">
          <label className="text-xs uppercase text-text-secondary">Payload</label>
          <textarea
            className="min-h-28 rounded-md border border-input bg-background px-3 py-2 text-sm text-text-primary"
            {...form.register('payload')}
          />
          {form.formState.errors.payload ? <p className="text-xs text-rose-300">{form.formState.errors.payload.message}</p> : null}
        </div>

        <div className="grid gap-2">
          <label className="text-xs uppercase text-text-secondary">Passphrase</label>
          <Input type="password" {...form.register('passphrase')} />
          {form.formState.errors.passphrase ? <p className="text-xs text-rose-300">{form.formState.errors.passphrase.message}</p> : null}
        </div>

        <div className="flex flex-wrap gap-2">
          <Button type="submit" variant="secondary">
            <Boxes className="mr-2 size-4" />
            Compress + encrypt
          </Button>
          <Button type="button" variant="outline" onClick={runRoundtrip}>
            <LockKeyhole className="mr-2 size-4" />
            Verify roundtrip
          </Button>
          <Button type="button" variant="outline" onClick={() => { void runtimeQuery.refetch(); void workspaceQuery.refetch(); }}>
            <DatabaseZap className="mr-2 size-4" />
            Refresh toolkit data
          </Button>
        </div>
      </form>

      {toolkitError ? <p className="mt-3 text-sm text-rose-300">{toolkitError}</p> : null}

      <div className="mt-4 grid gap-3 xl:grid-cols-2">
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Compressed payload (base64)</p>
          <pre className="mt-2 overflow-x-auto whitespace-pre-wrap break-all text-[11px] text-text-primary">{compressedBase64 || 'No compressed payload yet.'}</pre>
        </div>
        <div className="rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Encrypted payload</p>
          <pre className="mt-2 overflow-x-auto whitespace-pre-wrap break-all text-[11px] text-text-primary">{encryptedCiphertext ? `nonce=${encryptedNonce}\n\n${encryptedCiphertext}` : 'No encrypted payload yet.'}</pre>
        </div>
      </div>

      {roundtripPlaintext ? (
        <div className="mt-4 rounded-2xl border border-border bg-surface px-3 py-3">
          <p className="text-[11px] uppercase text-text-secondary">Roundtrip output</p>
          <pre className="mt-2 whitespace-pre-wrap break-words text-[11px] text-text-primary">{roundtripPlaintext}</pre>
        </div>
      ) : null}

      <div className="mt-4 overflow-hidden rounded-2xl border border-border">
        <table className="min-w-full divide-y divide-border text-left text-sm">
          <thead className="bg-surface-raised">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <th key={header.id} className="px-3 py-2 text-xs uppercase text-text-secondary">
                    {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody className="divide-y divide-border bg-card">
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} className="px-3 py-2 text-text-primary">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
            {!table.getRowModel().rows.length ? (
              <tr>
                <td colSpan={4} className="px-3 py-4 text-sm text-text-secondary">No crates available yet.</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </div>
  );
}

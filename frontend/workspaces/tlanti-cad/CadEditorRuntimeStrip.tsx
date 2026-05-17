import React from 'react';
import { Bot, Box, Cpu, Layers, Loader2, MonitorCog } from 'lucide-react';

import { cn } from '@/lib/utils';
import type { CadEditorRuntimeStatus, CadRuntimeLayer, CadRuntimeStatus } from '@/lib/cad-editor-runtime';

interface CadEditorRuntimeStripProps {
  status: CadEditorRuntimeStatus | null;
  loading: boolean;
  error: string | null;
  className?: string;
}

const layerIcon: Record<CadRuntimeLayer, React.ComponentType<{ className?: string }>> = {
  react: MonitorCog,
  three: Box,
  tauri: Layers,
  rust: Cpu,
  python: Bot,
};

const statusClass: Record<CadRuntimeStatus, string> = {
  ready: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-100',
  planned: 'border-amber-400/30 bg-amber-400/10 text-amber-100',
  disabled: 'border-white/10 bg-white/5 text-white/45',
};

export function CadEditorRuntimeStrip({ status, loading, error, className }: CadEditorRuntimeStripProps) {
  const points = status?.extensionPoints ?? [];

  return (
    <div
      className={cn(
        'pointer-events-auto flex max-w-[min(94vw,72rem)] flex-wrap items-center gap-2 rounded-md border border-white/10 bg-black/70 px-3 py-2 text-xs text-white shadow-xl backdrop-blur-xl',
        className,
      )}
      data-visual-qa-cad-runtime="true"
    >
      <div className="flex items-center gap-2 pr-2">
        {loading ? <Loader2 className="size-3.5 animate-spin text-cyan-200" /> : <Cpu className="size-3.5 text-cyan-200" />}
        <span className="font-mono uppercase tracking-[0.18em] text-white/60">Editor runtime</span>
      </div>

      {error ? (
        <span className="rounded border border-red-400/30 bg-red-500/10 px-2 py-1 text-red-100">
          {error}
        </span>
      ) : null}

      {points.map((point) => {
        const Icon = layerIcon[point.layer];
        return (
          <span
            key={point.id}
            className={cn(
              'inline-flex items-center gap-1.5 rounded border px-2 py-1',
              statusClass[point.status],
            )}
            title={point.notes}
          >
            <Icon className="size-3.5" />
            <span>{point.label}</span>
          </span>
        );
      })}

      {status ? (
        <span className="ml-auto rounded border border-white/10 px-2 py-1 font-mono uppercase tracking-[0.14em] text-white/45">
          {status.route}
        </span>
      ) : null}
    </div>
  );
}

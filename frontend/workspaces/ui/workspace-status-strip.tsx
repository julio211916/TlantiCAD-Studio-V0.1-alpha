import React from 'react';
import type { WorkspaceStatusItem } from '@/lib/workspace-shell';
import { cn } from '@/lib/utils';

export function WorkspaceStatusStrip({ items }: { items: WorkspaceStatusItem[] }) {
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-1.5">
      {items.map((item) => (
        <span
          key={item.id}
          className={cn(
            'rounded-full border border-white/10 bg-black/20 px-2 py-0.5 text-[10px] text-text-secondary',
            item.tone === 'success' && 'border-emerald-400/30 text-emerald-200',
            item.tone === 'warning' && 'border-amber-400/30 text-amber-200',
            item.tone === 'danger' && 'border-red-400/30 text-red-200',
          )}
        >
          {item.label}{item.value !== undefined ? `: ${String(item.value)}` : ''}
        </span>
      ))}
    </div>
  );
}

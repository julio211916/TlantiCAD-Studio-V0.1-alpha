import React from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface AssetLibraryPanelMessageProps {
  actionLabel?: string;
  onAction?: () => void;
  title: string;
  tone?: 'default' | 'dashed';
  description?: string;
  className?: string;
}

export function AssetLibraryPanelMessage({
  actionLabel,
  className,
  description,
  onAction,
  title,
  tone = 'default',
}: AssetLibraryPanelMessageProps) {
  return (
    <div
      className={cn(
        'rounded-2xl border bg-surface px-4 py-5 text-sm animate-in fade-in-0 duration-200',
        tone === 'dashed' ? 'border-dashed border-border text-text-secondary' : 'border-border text-text-secondary',
        className,
      )}
    >
      <p className="text-text-primary text-pretty">{title}</p>
      {description ? <p className="mt-1 text-pretty">{description}</p> : null}
      {actionLabel && onAction ? (
        <Button variant="outline" size="sm" className="mt-3" onClick={onAction}>
          {actionLabel}
        </Button>
      ) : null}
    </div>
  );
}
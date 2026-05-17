import React from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface AssetLibraryUsageCardProps {
  accentClassName?: string;
  description: string;
  disabled?: boolean;
  onAction: () => void;
  pending?: boolean;
  title: string;
}

export function AssetLibraryUsageCard({
  accentClassName,
  description,
  disabled,
  onAction,
  pending,
  title,
}: AssetLibraryUsageCardProps) {
  return (
    <div className="rounded-2xl border border-border bg-card p-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className={cn('text-sm text-text-primary text-balance', accentClassName)}>{title}</p>
          <p className="mt-1 text-sm text-text-secondary text-pretty">{description}</p>
        </div>
        <Button size="sm" onClick={onAction} disabled={disabled || pending}>
          {pending ? 'Working…' : title}
        </Button>
      </div>
    </div>
  );
}
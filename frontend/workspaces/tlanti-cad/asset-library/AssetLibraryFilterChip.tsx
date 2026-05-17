import React from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface AssetLibraryFilterChipProps {
  active: boolean;
  label: string;
  onClick: () => void;
}

export function AssetLibraryFilterChip({ active, label, onClick }: AssetLibraryFilterChipProps) {
  return (
    <Button
      type="button"
      variant={active ? 'secondary' : 'outline'}
      size="sm"
      aria-pressed={active}
      onClick={onClick}
      className={cn(
        'h-8 rounded-full px-3 text-xs',
        active ? 'border-text-primary bg-surface text-text-primary' : 'text-text-secondary hover:text-text-primary',
      )}
    >
      {label}
    </Button>
  );
}
import React from 'react';
import { FolderTree, Image as ImageIcon, Music4, Shapes } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { PublicAssetLibraryItem, PublicAssetLibrarySurface } from '@/lib/public-asset-library';
import { cn } from '@/lib/utils';

interface AssetLibraryItemRowProps {
  asset: PublicAssetLibraryItem;
  isBatchSelected: boolean;
  isSelected: boolean;
  onSelect: () => void;
  onToggleBatch: () => void;
  surface: PublicAssetLibrarySurface;
}

function getKindIcon(item: PublicAssetLibraryItem) {
  switch (item.kind) {
    case 'image':
    case 'vector':
      return ImageIcon;
    case 'audio':
      return Music4;
    case 'model':
      return Shapes;
    default:
      return FolderTree;
  }
}

export function AssetLibraryItemRow({ asset, isBatchSelected, isSelected, onSelect, onToggleBatch, surface }: AssetLibraryItemRowProps) {
  const Icon = getKindIcon(asset);
  const canUse = surface === 'cad' ? asset.supportedInCad : asset.supportedInDb;

  return (
    <div
      className={cn(
        'flex items-start gap-3 rounded-2xl border px-3 py-3 transition-colors hover:bg-card',
        isSelected ? 'border-text-primary bg-card' : 'border-border bg-transparent',
      )}
    >
      <Button
        type="button"
        variant={isBatchSelected ? 'secondary' : 'outline'}
        size="icon"
        aria-label={`Select ${asset.label} for batch action`}
        onClick={(event) => {
          event.stopPropagation();
          onToggleBatch();
        }}
        className={cn('mt-0.5 size-5 rounded-full', isBatchSelected ? 'border-text-primary bg-surface text-text-primary' : 'text-text-secondary')}
      >
        <span className="text-[10px] leading-none">{isBatchSelected ? '✓' : ''}</span>
      </Button>

      <button type="button" onClick={onSelect} className="flex min-w-0 flex-1 items-start gap-3 text-left">
        <div className="flex size-9 items-center justify-center rounded-xl border border-border bg-card text-text-secondary">
          <Icon className="size-4" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm text-text-display">{asset.label}</p>
          <p className="truncate text-xs text-text-secondary">{asset.relativePath}</p>
          <div className="mt-2 flex flex-wrap gap-2">
            <Badge className="border border-border bg-card text-text-primary">{asset.extension || 'file'}</Badge>
            <Badge className="border border-border bg-card text-text-primary">{asset.kind}</Badge>
            <Badge className={cn('border border-border', canUse ? 'bg-surface text-text-primary' : 'bg-card text-text-secondary')}>
              {canUse ? `Ready for ${surface.toUpperCase()}` : 'Reference only'}
            </Badge>
          </div>
        </div>
      </button>
    </div>
  );
}
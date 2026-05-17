import React, { lazy, Suspense } from 'react';

import { AssetLibraryPanelMessage } from '@/components/asset-library/AssetLibraryPanelMessage';
import type { PublicAssetLibraryItem } from '@/lib/public-asset-library';

const PublicAssetModelPreview = lazy(() => import('./PublicAssetModelPreview').then((module) => ({ default: module.PublicAssetModelPreview })));

export function LazyPublicAssetModelPreview({ asset }: { asset: PublicAssetLibraryItem }) {
  return (
    <Suspense
      fallback={(
        <AssetLibraryPanelMessage
          title="Preparing 3D preview"
          description="The model viewer loads only when needed to keep the workspace lighter."
        />
      )}
    >
      <PublicAssetModelPreview asset={asset} />
    </Suspense>
  );
}
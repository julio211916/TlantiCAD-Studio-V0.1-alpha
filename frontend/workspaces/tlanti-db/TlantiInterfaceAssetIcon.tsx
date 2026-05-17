import React, { useState } from 'react';
import clsx from 'clsx';

import { getTlantiInterfaceAsset, type TlantiInterfaceAssetKey } from '@/lib/tlanticad-interface-assets';

interface TlantiInterfaceAssetIconProps {
  assetKey: TlantiInterfaceAssetKey;
  className?: string;
  imageClassName?: string;
  fallbackClassName?: string;
}

export function TlantiInterfaceAssetIcon({
  assetKey,
  className,
  imageClassName,
  fallbackClassName,
}: TlantiInterfaceAssetIconProps) {
  const [failed, setFailed] = useState(false);
  const asset = getTlantiInterfaceAsset(assetKey);
  const FallbackIcon = asset.fallbackIcon;

  return (
    <span className={clsx('inline-flex size-5 shrink-0 items-center justify-center overflow-hidden', className)}>
      {failed ? (
        <FallbackIcon className={clsx('size-4 text-text-secondary', fallbackClassName)} />
      ) : (
        <img
          src={asset.src}
          alt=""
          width={20}
          height={20}
          loading="lazy"
          decoding="async"
          className={clsx('size-5 object-contain', imageClassName)}
          onError={() => setFailed(true)}
        />
      )}
    </span>
  );
}

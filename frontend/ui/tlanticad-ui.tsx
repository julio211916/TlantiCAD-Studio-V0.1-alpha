import type { ComponentProps } from 'react';

import { AppIcon } from '@/features/app-icons/AppIcon';

export function Icon(props: ComponentProps<typeof AppIcon>) {
  return <AppIcon {...props} />;
}

export { AppIcon };

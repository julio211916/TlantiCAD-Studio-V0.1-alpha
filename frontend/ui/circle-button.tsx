import * as React from 'react';

import { cn } from '@/lib/utils';

export interface CircleButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: React.ComponentType<{ className?: string }>;
}

const CircleButton = React.forwardRef<HTMLButtonElement, CircleButtonProps>(
  ({ className, icon: Icon, type = 'button', ...props }, ref) => (
    <button
      ref={ref}
      type={type}
      className={cn(
        'flex size-8 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/70 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
        className,
      )}
      {...props}
    >
      <Icon className="size-4 text-muted-foreground" />
    </button>
  ),
);
CircleButton.displayName = 'CircleButton';

export { CircleButton };

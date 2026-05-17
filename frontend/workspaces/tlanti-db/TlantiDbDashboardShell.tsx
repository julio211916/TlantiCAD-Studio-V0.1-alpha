import React from 'react';

import { Card } from '@/components/ui/card';
import { cn } from '@/lib/utils';

interface TlantiDbDashboardShellProps {
  header: React.ReactNode;
  left: React.ReactNode;
  center: React.ReactNode;
  right: React.ReactNode;
}

export function TlantiDbDashboardShell({ header, left, center, right }: TlantiDbDashboardShellProps) {
  return (
    <div className="flex h-dvh min-h-0 flex-col overflow-hidden bg-black text-text-primary">
      <div className="shrink-0">{header}</div>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-hidden p-2 lg:grid-cols-[clamp(19rem,24vw,23rem)_minmax(0,1fr)] xl:grid-cols-[clamp(19rem,22vw,23rem)_minmax(0,1fr)_clamp(18rem,20vw,22rem)]">
        <DashboardRegion className="min-h-[18rem] lg:min-h-0">{left}</DashboardRegion>
        <DashboardRegion className="min-h-[32rem] lg:min-h-0">{center}</DashboardRegion>
        <DashboardRegion className="min-h-[18rem] xl:min-h-0">{right}</DashboardRegion>
      </div>
    </div>
  );
}

export function DashboardRegion({ className, children }: { className?: string; children: React.ReactNode }) {
  return (
    <Card className={cn('min-h-0 overflow-hidden rounded-md border-border bg-surface/95 p-0 shadow-none', className)}>
      {children}
    </Card>
  );
}

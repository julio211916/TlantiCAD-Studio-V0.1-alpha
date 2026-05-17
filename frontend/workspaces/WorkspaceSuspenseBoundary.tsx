import React, { Suspense } from 'react';

import { ThemeMode } from '@/types';

import { TlantiWorkspacePreloader } from './TlantiWorkspacePreloader';

interface WorkspaceSuspenseBoundaryProps {
  subtitle: string;
  themeMode: ThemeMode;
  title: string;
  children: React.ReactNode;
}

export function WorkspaceSuspenseBoundary({ children, subtitle, themeMode, title }: WorkspaceSuspenseBoundaryProps) {
  return (
    <Suspense
      fallback={(
        <TlantiWorkspacePreloader
          visible
          title={title}
          subtitle={subtitle}
          themeMode={themeMode}
        />
      )}
    >
      {children}
    </Suspense>
  );
}

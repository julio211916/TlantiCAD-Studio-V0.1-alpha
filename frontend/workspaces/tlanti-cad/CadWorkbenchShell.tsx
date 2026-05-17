import React from 'react';

export interface CadWorkbenchShellProps {
  topDock?: React.ReactNode;
  rails?: React.ReactNode;
  advancedSummary?: React.ReactNode;
  activeOverlays?: React.ReactNode;
  floatingDock?: React.ReactNode;
  bottomStatus?: React.ReactNode;
  copilot?: React.ReactNode;
}

export function CadWorkbenchShell({
  topDock,
  rails,
  advancedSummary,
  activeOverlays,
  floatingDock,
  bottomStatus,
  copilot,
}: CadWorkbenchShellProps) {
  return (
    <>
      {topDock}
      {rails}
      {advancedSummary}
      {activeOverlays}
      {floatingDock}
      {bottomStatus}
      {copilot}
    </>
  );
}

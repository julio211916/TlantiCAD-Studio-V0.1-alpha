import React from 'react';
import { Activity, Box, FileText, Layers3, MoreHorizontal, PanelRightOpen, ScanSearch } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import type { DicomWorkspaceView, ThemeMode } from '@/types';

interface TlantiOhifActionMenuProps {
  activeView: DicomWorkspaceView;
  onSelectView: (view: DicomWorkspaceView) => void;
  onToggleInspector: () => void;
  onOpenDetails: () => void;
  onShowImportHint: () => void;
  themeMode: ThemeMode;
}

const viewItems = [
  { id: 'review' as const, label: 'Review', icon: Layers3 },
  { id: 'volume' as const, label: 'Volume', icon: Box },
  { id: 'ai' as const, label: 'AI analysis', icon: Activity },
  { id: 'report' as const, label: 'Report', icon: FileText },
];

export function TlantiOhifActionMenu({
  activeView,
  onSelectView,
  onToggleInspector,
  onOpenDetails,
  onShowImportHint,
}: TlantiOhifActionMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button type="button" variant="outline" size="icon" className="rounded-2xl border-border-visible bg-card/90 backdrop-blur-xl" aria-label="Abrir acciones DICOM">
          <MoreHorizontal size={16} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel>Viewer actions</DropdownMenuLabel>
        {viewItems.map(({ id, label, icon: Icon }) => (
          <DropdownMenuItem key={id} onClick={() => onSelectView(id)}>
            <Icon size={15} />
            <span>{label}</span>
            {activeView === id ? <DropdownMenuShortcut>active</DropdownMenuShortcut> : null}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={onToggleInspector}>
          <PanelRightOpen size={15} />
          <span>Toggle clinical panel</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={onOpenDetails}>
          <Layers3 size={15} />
          <span>Open details sheet</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={onShowImportHint}>
          <ScanSearch size={15} />
          <span>Import ZIP / folder hint</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

import React from 'react';
import type { LucideIcon } from 'lucide-react';
import { ChevronLeft, ChevronRight, LayoutPanelTop, PanelRightOpen } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import type { DicomWorkspaceView, ThemeMode } from '@/types';

interface DockItem {
  id: DicomWorkspaceView;
  label: string;
  icon: LucideIcon;
  badge?: string;
}

interface TlantiOhifClinicalDockProps {
  activeView: DicomWorkspaceView;
  items: DockItem[];
  isCollapsed: boolean;
  isInspectorOpen: boolean;
  onSelectView: (view: DicomWorkspaceView) => void;
  onToggleCollapsed: () => void;
  onToggleInspector: () => void;
  themeMode: ThemeMode;
}

export function TlantiOhifClinicalDock({
  activeView,
  items,
  isCollapsed,
  isInspectorOpen,
  onSelectView,
  onToggleCollapsed,
  onToggleInspector,
  themeMode,
}: TlantiOhifClinicalDockProps) {
  return (
    <div className="pointer-events-auto absolute left-4 top-1/2 z-20 -translate-y-1/2 sm:left-6">
      <div
        className={cn(
          'flex flex-col gap-2 rounded-[1.5rem] border p-2 shadow-xl backdrop-blur-xl transition-all',
          themeMode === 'dark'
            ? 'border-border bg-surface-raised/94 text-text-primary'
            : 'border-border bg-surface/94 text-text-primary',
        )}
      >
        <div className="flex items-center justify-between gap-2 px-1 pt-1">
          {!isCollapsed ? (
            <div>
              <p className="text-[11px] uppercase text-text-secondary">DICOM workflow</p>
              <p className="text-xs text-text-secondary">Panels ocultables</p>
            </div>
          ) : null}
          <Button type="button" variant="ghost" size="icon" onClick={onToggleCollapsed} aria-label={isCollapsed ? 'Expandir dock' : 'Colapsar dock'}>
            {isCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
          </Button>
        </div>

        <div className="flex flex-col gap-1">
          {items.map(({ id, label, icon: Icon, badge }) => {
            const active = activeView === id;
            return (
              <button
                key={id}
                type="button"
                onClick={() => onSelectView(id)}
                className={cn(
                  'flex items-center gap-3 rounded-2xl border px-3 py-2 text-left transition-colors',
                  active
                    ? 'border-[#FA93FA] bg-[#FA93FA]/15 text-text-primary'
                    : 'border-transparent text-text-secondary hover:border-border hover:bg-card hover:text-text-primary',
                  isCollapsed && 'justify-center px-2.5',
                )}
              >
                <Icon size={16} className={active ? 'text-[#FA93FA]' : ''} />
                {!isCollapsed ? (
                  <span className="min-w-0 flex-1 truncate text-sm">{label}</span>
                ) : null}
                {!isCollapsed && badge ? <Badge variant="outline" className="rounded-full border-border-visible bg-card text-[10px] text-text-secondary">{badge}</Badge> : null}
              </button>
            );
          })}
        </div>

        <div className="h-px bg-border" />

        <button
          type="button"
          onClick={onToggleInspector}
          className={cn(
            'flex items-center gap-3 rounded-2xl border px-3 py-2 text-left transition-colors',
            isInspectorOpen
              ? 'border-border-visible bg-card text-text-primary'
              : 'border-transparent text-text-secondary hover:border-border hover:bg-card hover:text-text-primary',
            isCollapsed && 'justify-center px-2.5',
          )}
        >
          {isInspectorOpen ? <PanelRightOpen size={16} /> : <LayoutPanelTop size={16} />}
          {!isCollapsed ? <span className="text-sm">{isInspectorOpen ? 'Ocultar panel clínico' : 'Mostrar panel clínico'}</span> : null}
        </button>
      </div>
    </div>
  );
}

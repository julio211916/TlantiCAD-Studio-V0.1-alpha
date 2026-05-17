import React from 'react';
import {
  Boxes,
  Camera,
  Copy,
  FileCode2,
  FileDown,
  FolderOpen,
  FolderSearch,
  LayoutDashboard,
  Plus,
  Printer,
  Save,
  Settings2,
  Share2,
  Sparkles,
} from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import type { TlantiDbSidebarPanelId } from '@/components/tlantidb/TlantiDbSidebar';

interface TlantiDbWorkspaceHeaderProps {
  activeCaseNumber: string;
  activeCaseName: string;
  assetCount: number;
  caseStatusLabel: string;
  caseStatusTone: 'neutral' | 'warning' | 'success' | 'info' | 'accent';
  leftPanelOpen: boolean;
  activePanel: TlantiDbSidebarPanelId;
  onToggleSidebar: () => void;
  onPanelChange: (panel: TlantiDbSidebarPanelId) => void;
  onCreateCase: () => void;
  onOpenCase: () => void;
  onOpenSettings: () => void;
  onSnapshot?: () => void;
  onCopyReference?: () => void;
  onPrint?: () => void;
  onExport?: () => void;
  onGenerateInteropXml?: () => void;
  onRevealFolder?: () => void;
  onShare?: () => void;
  canRevealFolder?: boolean;
}

const panelItems = [
  { id: 'case', label: 'Workspace', icon: LayoutDashboard },
  { id: 'assets', label: 'Assets', icon: Boxes },
  { id: 'settings', label: 'Settings', icon: Settings2 },
] satisfies Array<{
  id: TlantiDbSidebarPanelId;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}>;

export function TlantiDbWorkspaceHeader({
  activeCaseNumber,
  activeCaseName,
  assetCount,
  caseStatusLabel,
  caseStatusTone,
  leftPanelOpen,
  activePanel,
  onToggleSidebar,
  onPanelChange,
  onCreateCase,
  onOpenCase,
  onOpenSettings,
  onSnapshot,
  onCopyReference,
  onPrint,
  onExport,
  onGenerateInteropXml,
  onRevealFolder,
  onShare,
  canRevealFolder = true,
}: TlantiDbWorkspaceHeaderProps) {
  const statusClassName = {
    neutral: 'border-glass-border bg-control-bg text-text-primary',
    warning: 'border-warning/50 bg-warning/10 text-warning',
    success: 'border-success/50 bg-success/10 text-success-foreground',
    info: 'border-interactive/50 bg-interactive/10 text-interactive',
    accent: 'border-accent/50 bg-accent-subtle text-accent',
  }[caseStatusTone];

  const handlePanelSelect = (panel: TlantiDbSidebarPanelId) => {
    if (leftPanelOpen && activePanel === panel) {
      onToggleSidebar();
      return;
    }

    onPanelChange(panel);
  };

  return (
    <header className="border-b border-glass-border bg-window-bg/95 px-3 py-2">
      <div className="flex min-h-11 items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <img src="/logoo.svg" alt="TlantiCAD Studio" width={32} height={32} className="size-8 shrink-0 rounded-md object-contain" />
          <div className="min-w-0">
            <div className="flex min-w-0 items-center gap-2">
              <p className="truncate text-sm font-semibold leading-tight text-text-display">TlantiCAD Studio</p>
              <Badge className="hidden border border-glass-border bg-control-bg text-text-primary sm:inline-flex">{activeCaseNumber}</Badge>
              <Badge className={`hidden border sm:inline-flex ${statusClassName}`}>{caseStatusLabel}</Badge>
            </div>
            <p className="truncate text-[11px] text-text-secondary">
              {activeCaseName} · {assetCount} archivo{assetCount === 1 ? '' : 's'}
            </p>
          </div>
        </div>

        <nav className="hidden min-w-0 items-center gap-1 md:flex">
          {panelItems.map((item) => {
            const Icon = item.icon;
            const isActive = leftPanelOpen && activePanel === item.id;

            return (
              <button
                key={item.id}
                type="button"
                aria-label={`${isActive ? 'Ocultar' : 'Abrir'} panel ${item.label}`}
                aria-pressed={isActive}
                onClick={() => handlePanelSelect(item.id)}
                className={isActive
                  ? 'tl-control-active inline-flex h-8 items-center gap-2 rounded-md px-3 text-xs font-medium'
                  : 'inline-flex h-8 items-center gap-2 rounded-md px-3 text-xs font-medium text-text-secondary transition-colors hover:bg-control-bg-hover hover:text-text-primary'}
              >
                <Icon className="size-3.5" />
                {item.label}
              </button>
            );
          })}
        </nav>

        <div className="flex shrink-0 items-center gap-1">
          <button type="button" onClick={onCreateCase} className="inline-flex h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover">
            <Plus className="size-3.5" />
            New
          </button>
          <button type="button" onClick={onOpenCase} className="inline-flex h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover">
            <FolderOpen className="size-3.5" />
            Open
          </button>
          {onSnapshot ? (
            <button type="button" onClick={onSnapshot} className="hidden h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover lg:inline-flex">
              <Save className="size-3.5" />
              Save
            </button>
          ) : null}
          {onCopyReference ? (
            <button type="button" onClick={onCopyReference} className="hidden h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover xl:inline-flex">
              <Copy className="size-3.5" />
              Duplicate
            </button>
          ) : null}
          {onShare ? (
            <button type="button" onClick={onShare} className="hidden h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover xl:inline-flex">
              <Share2 className="size-3.5" />
              Share
            </button>
          ) : null}

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button type="button" aria-label="Abrir menú rápido del Workspace" className="inline-flex h-8 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-text-primary transition-colors hover:bg-control-bg-hover">
                <Sparkles className="size-3.5" />
                More
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-60">
              {onSnapshot && (
                <DropdownMenuItem onClick={onSnapshot}>
                  <Camera className="mr-2 size-4" />
                  Guardar snapshot
                </DropdownMenuItem>
              )}
              {onCopyReference && (
                <DropdownMenuItem onClick={onCopyReference}>
                  <Copy className="mr-2 size-4" />
                  Copiar referencia del caso
                </DropdownMenuItem>
              )}
              {onShare && (
                <DropdownMenuItem onClick={onShare}>
                  <Share2 className="mr-2 size-4" />
                  Compartir caso
                </DropdownMenuItem>
              )}
              <DropdownMenuSeparator />
              {onExport && (
                <DropdownMenuItem onClick={onExport}>
                  <FileDown className="mr-2 size-4" />
                  Exportar paquete del caso
                </DropdownMenuItem>
              )}
              {onGenerateInteropXml && (
                <DropdownMenuItem onClick={onGenerateInteropXml}>
                  <FileCode2 className="mr-2 size-4" />
                  Generar XML interop
                </DropdownMenuItem>
              )}
              {onPrint && (
                <DropdownMenuItem onClick={onPrint}>
                  <Printer className="mr-2 size-4" />
                  Imprimir caso
                </DropdownMenuItem>
              )}
              {onRevealFolder && (
                <DropdownMenuItem onClick={onRevealFolder} disabled={!canRevealFolder}>
                  <FolderSearch className="mr-2 size-4" />
                  Mostrar carpeta
                </DropdownMenuItem>
              )}
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={onOpenSettings}>
                <Settings2 className="mr-2 size-4" />
                Configuración del Workspace
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </header>
  );
}

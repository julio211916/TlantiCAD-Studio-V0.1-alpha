import React from 'react';
import {
  Archive,
  BookOpen,
  ChevronDown,
  Download,
  FolderOpen,
  Layers3,
  MoonStar,
  ScanSearch,
  Settings2,
  Smile,
  Syringe,
  Target,
  Upload,
} from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface DentalCadLogoMenuProps {
  activeCaseNumber?: string;
  activeModuleLabel: string;
  onBackToDb?: () => void;
  onImport: () => void;
  onExport: () => void;
  onOpenAssetLibrary: () => void;
  onOpenSmileWorkflow: () => void;
  onOpenImplant: () => void;
  onOpenGuide: () => void;
  onOpenSplint: () => void;
  onOpenLayers: () => void;
  onOpenCommandPalette: () => void;
}

export function DentalCadLogoMenu({
  activeCaseNumber,
  activeModuleLabel,
  onBackToDb,
  onImport,
  onExport,
  onOpenAssetLibrary,
  onOpenSmileWorkflow,
  onOpenImplant,
  onOpenGuide,
  onOpenSplint,
  onOpenLayers,
  onOpenCommandPalette,
}: DentalCadLogoMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          aria-label="TlantiCAD Studio workspace menu"
          className="group inline-flex h-11 max-w-[13.5rem] items-center gap-2 overflow-hidden rounded-md border border-border/80 bg-card/80 px-3 text-left shadow-sm backdrop-blur-xl transition-colors hover:bg-surface-raised"
        >
          <img src="/logoo.svg" alt="TlantiCAD Studio" className="size-7 object-contain" />
          <div className="min-w-0 flex-1">
            <p className="text-[10px] font-mono uppercase tracking-[0.18em] text-text-secondary">TlantiCAD Studio</p>
            <p className="truncate text-xs font-medium text-text-primary">{activeCaseNumber ?? 'CAD design'}</p>
          </div>
          <ChevronDown className="size-4 text-text-secondary transition-transform group-hover:text-text-primary" />
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="start" sideOffset={10} className="w-72 rounded-md border-border/80 bg-surface/96 backdrop-blur-xl">
        <DropdownMenuLabel className="px-3 py-3 font-normal">
          <div className="flex items-start gap-3">
            <img src="/logoo.svg" alt="TlantiCAD Studio" className="size-10 object-contain" />
            <div className="min-w-0 flex-1">
              <p className="text-sm font-semibold text-text-primary">{activeCaseNumber ?? 'CAD design'}</p>
              <p className="mt-0.5 text-xs text-text-secondary">Módulo activo: {activeModuleLabel}</p>
              <div className="mt-2 flex flex-wrap gap-2">
                <Badge variant="outline">Dental CAD</Badge>
                <Badge variant="outline">Local AI</Badge>
              </div>
            </div>
          </div>
        </DropdownMenuLabel>

        <DropdownMenuSeparator />

        <DropdownMenuGroup>
          <DropdownMenuItem onClick={onImport}>
            <Upload className="mr-2 size-4" />
            Import files
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onOpenAssetLibrary}>
            <FolderOpen className="mr-2 size-4" />
            Open asset library
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExport}>
            <Download className="mr-2 size-4" />
            Export scene
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onOpenCommandPalette}>
            <Archive className="mr-2 size-4" />
            Command palette
          </DropdownMenuItem>
        </DropdownMenuGroup>

        <DropdownMenuSeparator />

        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <BookOpen className="mr-2 size-4" />
            Clinical modules
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-64 rounded-md border-border/80 bg-surface/96 backdrop-blur-xl">
            <DropdownMenuItem onClick={onOpenSmileWorkflow}>
              <Smile className="mr-2 size-4" />
              Smile / Waxup workflow
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onOpenImplant}>
              <Syringe className="mr-2 size-4" />
              Implant planning
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onOpenGuide}>
              <Target className="mr-2 size-4" />
              Surgical guide
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onOpenSplint}>
              <MoonStar className="mr-2 size-4" />
              Splint workflow
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onOpenLayers}>
              <Layers3 className="mr-2 size-4" />
              Layers & groups
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onImport}>
              <ScanSearch className="mr-2 size-4" />
              DICOM intake
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSeparator />

        {onBackToDb ? (
          <DropdownMenuItem onClick={onBackToDb}>
            <Settings2 className="mr-2 size-4" />
            Back to Workspace
          </DropdownMenuItem>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

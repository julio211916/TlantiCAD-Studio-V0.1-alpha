import React from 'react';
import {
  Braces,
  Layers3,
  MoonStar,
  Moon,
  Search,
  ScanFace,
  Smile,
  Sparkles,
  Sun,
  Undo2,
  Redo2,
  Syringe,
  Target,
  ActivitySquare,
} from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { WorkspaceStatusStrip } from '@/components/ui/workspace-status-strip';
import type { WorkspaceModuleDefinition, WorkspaceStatusItem } from '@/lib/workspace-shell';
import type { ThemeMode, ToolMode } from '@/types';
import {
  listCadProductModules,
  type TlantiCadProductModuleDefinition,
  type TlantiCadProductModuleId,
} from '@/core';
import { cn } from '@/lib/utils';
import { DentalCadLogoMenu } from './DentalCadLogoMenu';
import {
  DENTAL_CAD_SHELL_ACTIONS,
  type DentalCadShellActionId,
} from '../config/dental-cad-shell';

interface DentalCadShellBarProps {
  activeCaseNumber?: string;
  activeCaseName?: string;
  activeModule: WorkspaceModuleDefinition;
  activeProductModule: TlantiCadProductModuleDefinition;
  caseStatusItems: WorkspaceStatusItem[];
  themeMode: ThemeMode;
  filesCount: number;
  isCompact: boolean;
  isExpertMode: boolean;
  activeSelectionSummary: string;
  activeToolLabel: string;
  snapLabel: string;
  transformSpace: string;
  canUndo: boolean;
  canRedo: boolean;
  moduleToolset: DentalCadShellActionId[];
  activeModuleAction?: DentalCadShellActionId | null;
  onBackToDb?: () => void;
  onImport: () => void;
  onExport: () => void;
  onOpenAssetLibrary: () => void;
  onOpenLayers: () => void;
  onOpenProperties: () => void;
  propertiesDisabled: boolean;
  onOpenCommandPalette: () => void;
  onOpenSmileWorkflow: () => void;
  onOpenImplant: () => void;
  onOpenGuide: () => void;
  onOpenSplint: () => void;
  onOpenOdontogram: () => void;
  onToggleTheme: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onToggleExpertMode: () => void;
  onActivateCadCore: () => void;
  onActivateDicomReview: () => void;
  onSelectProductModule: (moduleId: TlantiCadProductModuleId) => void;
  onSetTool: (tool: ToolMode) => void;
  onRunModuleAction: (actionId: DentalCadShellActionId) => void;
}

const productModules = listCadProductModules();

const productModuleIcons: Record<TlantiCadProductModuleId, React.ComponentType<{ className?: string; size?: number }>> = {
  'tlanticad-crown': ActivitySquare,
  'tlanticad-implant': Syringe,
  'tlanticad-bridge': Layers3,
  'tlanticad-waxup': Smile,
  'tlanticad-freeform': Sparkles,
  'tlanticad-abutment': Target,
  'tlanticad-model': Layers3,
  'tlanticad-bar': Braces,
  'tlanticad-telescope': ScanFace,
  'tlanticad-bite-splint': MoonStar,
};

export function DentalCadShellBar(props: DentalCadShellBarProps) {
  return (
    <div
      data-visual-qa-top-dock="true"
      className={cn('absolute inset-x-0 top-0 z-20 flex justify-center px-3 pt-3 sm:px-5', props.isCompact && 'px-2 pt-2')}
    >
      <div className={cn('pointer-events-auto w-[min(1180px,calc(100vw-96px))] space-y-2', props.isCompact && 'w-full')}>
        <header className="cad-top-dock">
          <div className="flex min-w-0 items-center gap-2">
            <DentalCadLogoMenu
              activeCaseNumber={props.activeCaseNumber}
              activeModuleLabel={props.activeModule.label}
              onBackToDb={props.onBackToDb}
              onImport={props.onImport}
              onExport={props.onExport}
              onOpenAssetLibrary={props.onOpenAssetLibrary}
              onOpenSmileWorkflow={props.onOpenSmileWorkflow}
              onOpenImplant={props.onOpenImplant}
              onOpenGuide={props.onOpenGuide}
              onOpenSplint={props.onOpenSplint}
              onOpenLayers={props.onOpenLayers}
              onOpenCommandPalette={props.onOpenCommandPalette}
            />
            <div className="hidden min-w-0 border-l border-white/10 pl-3 md:block">
              <p className="max-w-[14rem] truncate text-xs font-semibold text-text-display">{props.activeCaseName ?? 'CAD design'}</p>
              <p className="max-w-[14rem] truncate text-[10px] text-text-secondary">{props.activeCaseNumber ?? 'Clinical case'} · {props.filesCount} assets</p>
            </div>
          </div>

          <div className="cad-top-scroll">
            {productModules.map((item) => {
              const Icon = productModuleIcons[item.id];
              const isActive = props.activeProductModule.id === item.id;

              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => props.onSelectProductModule(item.id)}
                  className={cn('cad-top-icon-button', isActive && 'cad-top-icon-button-active')}
                  title={`${item.label}: ${item.purpose}`}
                  aria-label={item.label}
                >
                  <Icon className="size-4" />
                </button>
              );
            })}
          </div>

          <div className="flex shrink-0 items-center gap-1">
            {props.moduleToolset.slice(0, props.isCompact ? 3 : 6).map((actionId) => {
              const action = DENTAL_CAD_SHELL_ACTIONS[actionId];
              const Icon = action.icon;
              const isActive = props.activeModuleAction === actionId;

              return (
                <button
                  key={action.id}
                  type="button"
                  onClick={() => props.onRunModuleAction(action.id)}
                  className={cn('cad-top-icon-button', isActive && 'cad-top-icon-button-accent')}
                  title={action.description}
                  aria-label={action.label}
                >
                  <Icon className="size-4" />
                </button>
              );
            })}
            <button
              type="button"
              onClick={props.onUndo}
              disabled={!props.canUndo}
              className={cn('cad-top-icon-button', !props.canUndo && 'cursor-not-allowed opacity-35')}
              title="Undo"
              aria-label="Undo"
            >
              <Undo2 className="size-4" />
            </button>
            <button
              type="button"
              onClick={props.onRedo}
              disabled={!props.canRedo}
              className={cn('cad-top-icon-button', !props.canRedo && 'cursor-not-allowed opacity-35')}
              title="Redo"
              aria-label="Redo"
            >
              <Redo2 className="size-4" />
            </button>
            <button
              type="button"
              onClick={props.onOpenCommandPalette}
              className="cad-top-icon-button"
              title="Command palette"
              aria-label="Command palette"
            >
              <Search className="size-4" />
            </button>
            <button
              type="button"
              onClick={props.onToggleExpertMode}
              className={cn('cad-top-icon-button hidden md:inline-flex', props.isExpertMode && 'cad-top-icon-button-success')}
              title={props.isExpertMode ? 'Expert mode' : 'Assistant mode'}
              aria-label="Toggle CAD mode"
            >
              {props.isExpertMode ? 'EX' : 'AS'}
            </button>
            <button
              type="button"
              onClick={props.onToggleTheme}
              className="cad-top-icon-button"
              title="Toggle theme"
              aria-label="Toggle theme"
            >
              {props.themeMode === 'dark' ? <Sun className="size-4" /> : <Moon className="size-4" />}
            </button>
          </div>
        </header>

        <div className={cn('pointer-events-auto rounded-md border border-white/8 bg-[#101215]/84 px-3 py-1.5 shadow-[0_12px_28px_rgba(0,0,0,0.16)] backdrop-blur-xl', props.isCompact ? 'space-y-2' : 'flex items-center justify-between gap-3')}>
          <div className="min-w-0 flex-1">
            <WorkspaceStatusStrip items={props.caseStatusItems} />
          </div>
          <div className={cn('flex flex-wrap items-center gap-2 text-[11px] text-text-secondary', props.isCompact ? '' : 'justify-end')}>
            <Badge variant="outline">tool · {props.activeToolLabel}</Badge>
            <Badge variant="outline">{props.activeSelectionSummary}</Badge>
            <Badge variant="outline">{props.snapLabel}</Badge>
            <Badge variant="outline" className="capitalize">{props.transformSpace}</Badge>
          </div>
        </div>
      </div>
    </div>
  );
}

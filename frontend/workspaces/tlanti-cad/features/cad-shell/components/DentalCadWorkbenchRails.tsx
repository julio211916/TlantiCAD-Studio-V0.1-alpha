import React from 'react';
import {
  Grid3X3,
  Settings2,
  ShieldCheck,
  Sparkles,
} from 'lucide-react';

import type { ToolMode } from '@/types';
import { cn } from '@/lib/utils';
import { AppIcon, type AppIconName } from '@/features/app-icons';
import {
  DENTAL_CAD_SHELL_ACTIONS,
  type DentalCadShellActionId,
} from '../config/dental-cad-shell';
import type {
  TlantiCadModuleRoadmapDefinition,
  TlantiCadModuleWorkflowPhase,
  TlantiCadProductModuleDefinition,
} from '@/core';

interface DentalCadWorkbenchRailsProps {
  activeTool: ToolMode;
  activeProductModule: TlantiCadProductModuleDefinition;
  activeRoadmap: TlantiCadModuleRoadmapDefinition;
  activeWorkflowPhase: TlantiCadModuleWorkflowPhase;
  activeModuleAction?: DentalCadShellActionId | null;
  moduleToolset: DentalCadShellActionId[];
  isCompact: boolean;
  isExpertMode: boolean;
  hasSelection: boolean;
  onSetTool: (tool: ToolMode) => void;
  onRunModuleAction: (actionId: DentalCadShellActionId) => void;
  onImport: () => void;
  onOpenLayers: () => void;
  onOpenAssetLibrary: () => void;
  onOpenCommandPalette: () => void;
  onOpenToolsWorkflows: () => void;
  onOpenProperties: () => void;
  onOpenOdontogram: () => void;
}

type RailAction = {
  id: string;
  label: string;
  icon: React.ComponentType<{ className?: string; size?: number }>;
  run: () => void;
  active?: boolean;
  disabled?: boolean;
  tone?: 'default' | 'danger' | 'accent' | 'success';
};

function railIcon(name: AppIconName): RailAction['icon'] {
  const RailSemanticIcon = ({ className, size = 20 }: { className?: string; size?: number }) => (
    <AppIcon name={name} className={className} size={size} aria-hidden />
  );

  RailSemanticIcon.displayName = `RailSemanticIcon(${name})`;
  return RailSemanticIcon;
}

function RailButton({ action }: { action: RailAction }) {
  const Icon = action.icon;

  return (
    <button
      type="button"
      onClick={action.run}
      disabled={action.disabled}
      className={cn(
        'group relative flex size-12 items-center justify-center rounded-[0.9rem] border border-transparent text-[var(--cad-rail-muted)] transition-colors',
        'hover:border-white/10 hover:bg-white/8 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#7c5cff]',
        action.active && 'border-white/12 bg-white/12 text-white shadow-[inset_3px_0_0_var(--cad-rail-accent)]',
        action.tone === 'accent' && 'text-[#8fb4ff]',
        action.tone === 'success' && 'text-[#9ee7bd]',
        action.tone === 'danger' && 'text-[#ff7777]',
        action.disabled && 'cursor-not-allowed opacity-35 hover:border-transparent hover:bg-transparent',
      )}
      aria-label={action.label}
      title={action.label}
    >
      <Icon className="size-5" />
      <span className="pointer-events-none absolute left-[calc(100%+0.6rem)] top-1/2 z-50 hidden -translate-y-1/2 whitespace-nowrap rounded-md border border-white/10 bg-[#17191d] px-2 py-1 text-[11px] text-white shadow-xl group-hover:block">
        {action.label}
      </span>
    </button>
  );
}

function RailGroup({ actions }: { actions: RailAction[] }) {
  return (
    <div className="flex flex-col gap-1">
      {actions.map((action) => (
        <RailButton key={action.id} action={action} />
      ))}
    </div>
  );
}

export function DentalCadWorkbenchRails({
  activeTool,
  activeProductModule,
  activeRoadmap,
  activeWorkflowPhase,
  activeModuleAction,
  moduleToolset,
  isCompact,
  isExpertMode,
  hasSelection,
  onSetTool,
  onRunModuleAction,
  onImport,
  onOpenLayers,
  onOpenAssetLibrary,
  onOpenCommandPalette,
  onOpenToolsWorkflows,
  onOpenProperties,
  onOpenOdontogram,
}: DentalCadWorkbenchRailsProps) {
  if (isCompact) {
    return null;
  }

  const workspaceActions: RailAction[] = [
    { id: 'import', label: 'Import STL/DICOM', icon: railIcon('action.import-asset'), run: onImport, tone: 'accent' },
    { id: 'files', label: 'Files and layers', icon: railIcon('module.asset-vault'), run: onOpenLayers },
    { id: 'library', label: 'Dental asset library', icon: railIcon('tool.choose-library-tooth'), run: onOpenAssetLibrary },
    { id: 'odontogram', label: 'Tooth context', icon: railIcon('tool.tooth-axes'), run: onOpenOdontogram },
  ];

  const viewportActions: RailAction[] = [
    { id: 'render', label: 'Render mode', icon: railIcon('tool.screenshot'), run: onOpenCommandPalette },
    { id: 'grid', label: 'Grid and voxel view', icon: Grid3X3, run: () => onSetTool('VOXELIZE'), active: activeTool === 'VOXELIZE' },
    { id: 'properties', label: 'Properties inspector', icon: Settings2, run: onOpenProperties, disabled: !hasSelection },
    {
      id: 'workflow',
      label: `${activeProductModule.label}: ${activeWorkflowPhase.label} · ${activeWorkflowPhase.logicOwner} · ${activeRoadmap.differentiators[0]}`,
      icon: ShieldCheck,
      run: onOpenToolsWorkflows,
      tone: 'success',
    },
  ];

  const productActions: RailAction[] = moduleToolset.map((actionId) => {
    const action = DENTAL_CAD_SHELL_ACTIONS[actionId];
    return {
      id: action.id,
      label: `${action.label} - ${action.description}`,
      icon: action.icon,
      run: () => onRunModuleAction(action.id),
      active: activeModuleAction === action.id,
      tone: action.id === 'manufacturing-export' || action.id === 'splint-export' ? 'success' : action.id === 'boolean' ? 'danger' : action.id === 'sculpt' ? 'accent' : 'default',
    };
  });

  const assistActions: RailAction[] = [
    { id: 'ai', label: isExpertMode ? 'AI tools expert' : 'AI assisted tools', icon: Sparkles, run: onOpenCommandPalette, tone: 'success' },
  ];

  return (
    <>
      <aside className="cad-sculpt-rail cad-sculpt-rail-left" aria-label="Workspace tools">
        <RailGroup actions={workspaceActions} />
        <div className="h-px w-8 bg-white/10" />
        <RailGroup actions={viewportActions} />
      </aside>

      <aside className="cad-sculpt-rail cad-sculpt-rail-right" aria-label="Dental edit tools">
        <RailGroup actions={productActions.slice(0, 6)} />
        <div className="h-px w-8 bg-white/10" />
        <RailGroup actions={[...productActions.slice(6), ...assistActions]} />
      </aside>
    </>
  );
}

import type { CadCommandPanelsProps } from '../CadCommandPanels'
import type { CadOverlaysProps } from '../CadOverlays'
import type { CadWorkbenchShellProps } from '../CadWorkbenchShell'
import type { CanvasSceneProps } from '../CanvasScene'

export interface CadWorkbenchViewModels {
  canvas: CanvasSceneProps
  overlays: CadOverlaysProps
  commands: CadCommandPanelsProps
  primaryShell: Pick<CadWorkbenchShellProps, 'topDock' | 'rails' | 'advancedSummary' | 'activeOverlays'>
  secondaryShell: Pick<CadWorkbenchShellProps, 'floatingDock' | 'bottomStatus' | 'copilot'>
}

export function defineCanvasSceneViewModel(viewModel: CanvasSceneProps): CanvasSceneProps {
  return viewModel
}

export function defineCadOverlaysViewModel(viewModel: CadOverlaysProps): CadOverlaysProps {
  return viewModel
}

export function defineCadCommandPanelsViewModel(viewModel: CadCommandPanelsProps): CadCommandPanelsProps {
  return viewModel
}

export function definePrimaryShellViewModel(
  viewModel: CadWorkbenchViewModels['primaryShell'],
): CadWorkbenchViewModels['primaryShell'] {
  return viewModel
}

export function defineSecondaryShellViewModel(
  viewModel: CadWorkbenchViewModels['secondaryShell'],
): CadWorkbenchViewModels['secondaryShell'] {
  return viewModel
}

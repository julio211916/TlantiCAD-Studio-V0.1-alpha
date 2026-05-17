import { create } from 'zustand';

import type { TlantiDbSidebarPanelId } from '@/components/tlantidb/TlantiDbSidebar';

interface TlantiDbUiState {
  activeSidebarPanel: TlantiDbSidebarPanelId;
  hoveredToothNumber: string | null;
  isLeftPanelOpen: boolean;
  selectedModuleId: string | null;
  setActiveSidebarPanel: (panel: TlantiDbSidebarPanelId) => void;
  setHoveredToothNumber: (toothNumber: string | null) => void;
  setIsLeftPanelOpen: (open: boolean | ((current: boolean) => boolean)) => void;
  setSelectedModuleId: (moduleId: string | null) => void;
}

export const useTlantiDbUiStore = create<TlantiDbUiState>((set) => ({
  activeSidebarPanel: 'case',
  hoveredToothNumber: null,
  isLeftPanelOpen: true,
  selectedModuleId: null,
  setActiveSidebarPanel: (activeSidebarPanel) => set({ activeSidebarPanel }),
  setHoveredToothNumber: (hoveredToothNumber) => set({ hoveredToothNumber }),
  setIsLeftPanelOpen: (open) => set((state) => ({
    isLeftPanelOpen: typeof open === 'function' ? open(state.isLeftPanelOpen) : open,
  })),
  setSelectedModuleId: (selectedModuleId) => set({ selectedModuleId }),
}));

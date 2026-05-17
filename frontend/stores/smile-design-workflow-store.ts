import { create } from 'zustand';

import { SMILE_DESIGN_PLAYBOOKS, type SmileDesignPlaybookId } from '@/lib/smile-design-playbooks';

interface SmileDesignWorkflowState {
  selectedPlaybookId: SmileDesignPlaybookId;
  currentStageByPlaybook: Record<string, number>;
  completedTaskIds: Record<string, boolean>;
  notesByPlaybook: Record<string, string>;
  selectPlaybook: (id: SmileDesignPlaybookId) => void;
  goToStage: (id: SmileDesignPlaybookId, stageIndex: number) => void;
  nextStage: (id: SmileDesignPlaybookId) => void;
  previousStage: (id: SmileDesignPlaybookId) => void;
  toggleTask: (taskId: string, checked: boolean) => void;
  setNotes: (id: SmileDesignPlaybookId, value: string) => void;
  resetPlaybook: (id: SmileDesignPlaybookId) => void;
}

function getMaxStageIndex(id: SmileDesignPlaybookId) {
  return Math.max(0, (SMILE_DESIGN_PLAYBOOKS.find((item) => item.id === id)?.stages.length ?? 1) - 1);
}

export const useSmileDesignWorkflowStore = create<SmileDesignWorkflowState>((set, get) => ({
  selectedPlaybookId: 'ai-mockup',
  currentStageByPlaybook: {},
  completedTaskIds: {},
  notesByPlaybook: {},
  selectPlaybook: (id) => set({ selectedPlaybookId: id }),
  goToStage: (id, stageIndex) => set((state) => ({
    currentStageByPlaybook: {
      ...state.currentStageByPlaybook,
      [id]: Math.min(Math.max(0, stageIndex), getMaxStageIndex(id)),
    },
  })),
  nextStage: (id) => {
    const current = get().currentStageByPlaybook[id] ?? 0;
    get().goToStage(id, current + 1);
  },
  previousStage: (id) => {
    const current = get().currentStageByPlaybook[id] ?? 0;
    get().goToStage(id, current - 1);
  },
  toggleTask: (taskId, checked) => set((state) => ({
    completedTaskIds: { ...state.completedTaskIds, [taskId]: checked },
  })),
  setNotes: (id, value) => set((state) => ({
    notesByPlaybook: { ...state.notesByPlaybook, [id]: value },
  })),
  resetPlaybook: (id) => set((state) => ({
    currentStageByPlaybook: { ...state.currentStageByPlaybook, [id]: 0 },
    notesByPlaybook: { ...state.notesByPlaybook, [id]: '' },
  })),
}));

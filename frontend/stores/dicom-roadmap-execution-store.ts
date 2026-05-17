import { create } from 'zustand';

import type { DicomExecutionBriefForm } from '@/lib/dicom-roadmap-execution.schemas';

interface DicomRoadmapExecutionState {
  brief: DicomExecutionBriefForm;
  taskStates: Record<string, boolean>;
  setBrief: (brief: DicomExecutionBriefForm) => void;
  toggleTask: (taskId: string, checked: boolean) => void;
  resetPhase: () => void;
}

const DEFAULT_BRIEF: DicomExecutionBriefForm = {
  owner: 'TlantiCAD',
  validationMode: 'clinical',
  evidenceGoal: 'Validar DICOM local con preview, segmentacion y gate offline.',
};

export const useDicomRoadmapExecutionStore = create<DicomRoadmapExecutionState>((set) => ({
  brief: DEFAULT_BRIEF,
  taskStates: {},
  setBrief: (brief) => set({ brief }),
  toggleTask: (taskId, checked) => set((state) => ({
    taskStates: { ...state.taskStates, [taskId]: checked },
  })),
  resetPhase: () => set({ brief: DEFAULT_BRIEF, taskStates: {} }),
}));

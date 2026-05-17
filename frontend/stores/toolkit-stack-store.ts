import { create } from 'zustand';

interface ToolkitStackState {
  payload: string;
  passphrase: string;
  lastAction: string | null;
  setDraft: (draft: Partial<Pick<ToolkitStackState, 'payload' | 'passphrase' | 'lastAction'>>) => void;
}

export const useToolkitStackStore = create<ToolkitStackState>((set) => ({
  payload: JSON.stringify({ app: 'TlantiCAD Studio', runtime: 'local-tauri-fastapi-next' }, null, 2),
  passphrase: 'tlanticad-local',
  lastAction: null,
  setDraft: (draft) => set((state) => ({ ...state, ...draft })),
}));

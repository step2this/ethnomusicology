import { create } from 'zustand';

interface GenerationState {
  isGenerating: boolean;
  isArranging: boolean;
  error: string | null;
  setGenerating: (v: boolean) => void;
  setArranging: (v: boolean) => void;
  setError: (e: string | null) => void;
  reset: () => void;
}

export const useGenerationStore = create<GenerationState>((set) => ({
  isGenerating: false,
  isArranging: false,
  error: null,
  setGenerating: (v) => set({ isGenerating: v }),
  setArranging: (v) => set({ isArranging: v }),
  setError: (e) => set({ error: e }),
  reset: () => set({ isGenerating: false, isArranging: false, error: null }),
}));

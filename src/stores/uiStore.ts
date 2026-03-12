// ContextPaste — UI Store (Zustand)

import { create } from "zustand";
import type { WorkflowChain } from "../lib/types";

type View = "quick-paste" | "history" | "settings";

interface UIState {
  currentView: View;
  overlayVisible: boolean;
  searchQuery: string;
  selectedIndex: number;
  activeChain: WorkflowChain | null;

  setView: (view: View) => void;
  showOverlay: () => void;
  hideOverlay: () => void;
  toggleOverlay: () => void;
  setSearchQuery: (query: string) => void;
  setSelectedIndex: (index: number) => void;
  moveSelection: (delta: number, maxItems: number) => void;
  setActiveChain: (chain: WorkflowChain) => void;
  clearChain: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  currentView: "quick-paste",
  overlayVisible: false,
  searchQuery: "",
  selectedIndex: 0,
  activeChain: null,

  setView: (view) => set({ currentView: view }),

  showOverlay: () =>
    set({ overlayVisible: true, searchQuery: "", selectedIndex: 0 }),

  hideOverlay: () => set({ overlayVisible: false }),

  toggleOverlay: () =>
    set((state) =>
      state.overlayVisible
        ? { overlayVisible: false }
        : { overlayVisible: true, searchQuery: "", selectedIndex: 0 },
    ),

  setSearchQuery: (query) => set({ searchQuery: query, selectedIndex: 0 }),

  setSelectedIndex: (index) => set({ selectedIndex: index }),

  moveSelection: (delta, maxItems) =>
    set((state) => {
      const next = state.selectedIndex + delta;
      if (next < 0) return { selectedIndex: maxItems - 1 };
      if (next >= maxItems) return { selectedIndex: 0 };
      return { selectedIndex: next };
    }),

  setActiveChain: (chain) => set({ activeChain: chain }),

  clearChain: () => set({ activeChain: null }),
}));

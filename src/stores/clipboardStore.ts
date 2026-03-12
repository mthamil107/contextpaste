// ContextPaste — Clipboard Store (Zustand)

import { create } from "zustand";
import type { ClipItem, RankedItem } from "../lib/types";
import {
  getRecentItems,
  deleteItem as deleteItemApi,
  togglePin as togglePinApi,
  toggleStar as toggleStarApi,
  pasteItem as pasteItemApi,
  clearHistory as clearHistoryApi,
  getPredictions,
} from "../lib/tauri";

interface ClipboardState {
  items: ClipItem[];
  predictions: RankedItem[];
  loading: boolean;
  error: string | null;

  // Actions
  fetchItems: (limit?: number, offset?: number) => Promise<void>;
  fetchPredictions: (limit?: number) => Promise<void>;
  addItem: (item: ClipItem) => void;
  removeItem: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  toggleStar: (id: string) => Promise<void>;
  pasteItem: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
}

export const useClipboardStore = create<ClipboardState>((set) => ({
  items: [],
  predictions: [],
  loading: false,
  error: null,

  fetchItems: async (limit = 50, offset = 0) => {
    set({ loading: true, error: null });
    try {
      const items = await getRecentItems(limit, offset);
      set({ items, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchPredictions: async (limit = 8) => {
    try {
      const predictions = await getPredictions(limit);
      set({ predictions });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addItem: (item: ClipItem) => {
    set((state) => ({
      items: [item, ...state.items.filter((i) => i.id !== item.id)],
    }));
  },

  removeItem: async (id: string) => {
    try {
      await deleteItemApi(id);
      set((state) => ({
        items: state.items.filter((i) => i.id !== id),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  togglePin: async (id: string) => {
    try {
      await togglePinApi(id);
      set((state) => ({
        items: state.items.map((i) =>
          i.id === id ? { ...i, isPinned: !i.isPinned } : i,
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  toggleStar: async (id: string) => {
    try {
      await toggleStarApi(id);
      set((state) => ({
        items: state.items.map((i) =>
          i.id === id ? { ...i, isStarred: !i.isStarred } : i,
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  pasteItem: async (id: string) => {
    try {
      await pasteItemApi(id);
      set((state) => ({
        items: state.items.map((i) =>
          i.id === id ? { ...i, pasteCount: i.pasteCount + 1 } : i,
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearHistory: async () => {
    try {
      await clearHistoryApi();
      set((state) => ({
        items: state.items.filter((i) => i.isPinned),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

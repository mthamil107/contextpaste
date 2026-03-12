// ContextPaste — Settings Store (Zustand)

import { create } from "zustand";
import type { AppSettings } from "../lib/types";
import {
  getAllSettings,
  updateSetting as updateSettingApi,
} from "../lib/tauri";
import { DEFAULTS } from "../lib/constants";

interface SettingsState {
  settings: AppSettings;
  loaded: boolean;

  fetchSettings: () => Promise<void>;
  updateSetting: (key: string, value: string) => Promise<void>;
}

const DEFAULT_SETTINGS: AppSettings = {
  maxHistoryItems: DEFAULTS.MAX_HISTORY_ITEMS,
  credentialAutoExpireMinutes: DEFAULTS.CREDENTIAL_AUTO_EXPIRE_MINUTES,
  hotkeyQuickPaste: DEFAULTS.HOTKEY_QUICK_PASTE,
  hotkeyHistory: DEFAULTS.HOTKEY_HISTORY,
  theme: "system",
  showSourceContext: true,
  showTypeBadges: true,
  enablePredictions: true,
  enableWorkflowChains: false,
  enableSemanticSearch: false,
  aiProvider: "local",
  ignoredApps: [],
  startupOnLogin: false,
  overlayPosition: "cursor",
  overlayMaxItems: DEFAULTS.OVERLAY_MAX_ITEMS,
  dedupEnabled: true,
  dedupWindowSeconds: DEFAULTS.DEDUP_WINDOW_SECONDS,
};

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,

  fetchSettings: async () => {
    try {
      const raw = await getAllSettings();
      const merged = { ...DEFAULT_SETTINGS };

      // Parse each setting from the raw key-value map
      if (raw.maxHistoryItems)
        merged.maxHistoryItems = parseInt(raw.maxHistoryItems, 10);
      if (raw.credentialAutoExpireMinutes)
        merged.credentialAutoExpireMinutes = parseInt(
          raw.credentialAutoExpireMinutes,
          10,
        );
      if (raw.hotkeyQuickPaste)
        merged.hotkeyQuickPaste = raw.hotkeyQuickPaste;
      if (raw.hotkeyHistory) merged.hotkeyHistory = raw.hotkeyHistory;
      if (raw.theme)
        merged.theme = raw.theme as AppSettings["theme"];
      if (raw.showSourceContext)
        merged.showSourceContext = raw.showSourceContext === "true";
      if (raw.showTypeBadges)
        merged.showTypeBadges = raw.showTypeBadges === "true";
      if (raw.enablePredictions)
        merged.enablePredictions = raw.enablePredictions === "true";
      if (raw.overlayMaxItems)
        merged.overlayMaxItems = parseInt(raw.overlayMaxItems, 10);
      if (raw.overlayPosition)
        merged.overlayPosition =
          raw.overlayPosition as AppSettings["overlayPosition"];
      if (raw.dedupEnabled)
        merged.dedupEnabled = raw.dedupEnabled === "true";

      set({ settings: merged, loaded: true });
    } catch (e) {
      console.error("Failed to load settings:", e);
      set({ loaded: true });
    }
  },

  updateSetting: async (key: string, value: string) => {
    try {
      await updateSettingApi(key, value);
      set((state) => ({
        settings: { ...state.settings, [key]: value },
      }));
    } catch (e) {
      console.error("Failed to update setting:", e);
    }
  },
}));

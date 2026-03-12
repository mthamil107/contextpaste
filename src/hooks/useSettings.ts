// ContextPaste — Settings Hook

import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import { onSettingsChanged } from "../lib/tauri";

export function useSettings() {
  const { settings, loaded, fetchSettings, updateSetting } = useSettingsStore();

  useEffect(() => {
    fetchSettings();

    const unlisten = onSettingsChanged(({ key, value }) => {
      useSettingsStore.setState((state) => ({
        settings: { ...state.settings, [key]: value },
      }));
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return { settings, loaded, updateSetting };
}

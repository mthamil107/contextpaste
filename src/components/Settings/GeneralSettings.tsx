// ContextPaste — General Settings

import { useSettings } from "../../hooks/useSettings";

export function GeneralSettings() {
  const { settings, updateSetting } = useSettings();

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">General</h3>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Max history items</span>
        <input
          type="number"
          value={settings.maxHistoryItems}
          onChange={(e) => updateSetting("maxHistoryItems", e.target.value)}
          data-testid="setting-max-history-items"
          className="w-24 rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
          min={100}
          max={50000}
        />
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Theme</span>
        <select
          data-testid="setting-theme"
          value={settings.theme}
          onChange={(e) => {
            updateSetting("theme", e.target.value);
            if (e.target.value === "dark") {
              document.documentElement.classList.add("dark");
            } else if (e.target.value === "light") {
              document.documentElement.classList.remove("dark");
            }
          }}
          className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Overlay position</span>
        <select
          data-testid="setting-overlay-position"
          value={settings.overlayPosition}
          onChange={(e) => updateSetting("overlayPosition", e.target.value)}
          className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
        >
          <option value="cursor">At cursor</option>
          <option value="center">Center</option>
          <option value="top-right">Top right</option>
        </select>
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Overlay max items</span>
        <input
          type="number"
          value={settings.overlayMaxItems}
          onChange={(e) => updateSetting("overlayMaxItems", e.target.value)}
          data-testid="setting-overlay-max-items"
          className="w-24 rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
          min={3}
          max={20}
        />
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Enable deduplication</span>
        <input
          type="checkbox"
          checked={settings.dedupEnabled}
          onChange={(e) =>
            updateSetting("dedupEnabled", String(e.target.checked))
          }
          data-testid="setting-dedup-enabled"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Show type badges</span>
        <input
          type="checkbox"
          checked={settings.showTypeBadges}
          onChange={(e) =>
            updateSetting("showTypeBadges", String(e.target.checked))
          }
          data-testid="setting-show-type-badges"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Show source context</span>
        <input
          type="checkbox"
          checked={settings.showSourceContext}
          onChange={(e) =>
            updateSetting("showSourceContext", String(e.target.checked))
          }
          data-testid="setting-show-source-context"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>
    </div>
  );
}

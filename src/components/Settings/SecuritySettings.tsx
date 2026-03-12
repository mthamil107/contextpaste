// ContextPaste — Security Settings

import { useSettings } from "../../hooks/useSettings";
import { useClipboardStore } from "../../stores/clipboardStore";
import { clearExpiredCredentials } from "../../lib/tauri";

export function SecuritySettings() {
  const { settings, updateSetting } = useSettings();
  const { clearHistory } = useClipboardStore();

  const handleClearExpired = async () => {
    try {
      await clearExpiredCredentials();
    } catch (e) {
      console.error("Failed to clear expired credentials:", e);
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">Security</h3>

      <label className="flex items-center justify-between">
        <div>
          <span className="text-sm text-cp-text">
            Credential auto-expire (minutes)
          </span>
          <p className="text-xs text-cp-muted">
            Detected secrets are automatically deleted after this duration
          </p>
        </div>
        <input
          type="number"
          value={settings.credentialAutoExpireMinutes}
          onChange={(e) =>
            updateSetting("credentialAutoExpireMinutes", e.target.value)
          }
          data-testid="setting-credential-auto-expire"
          className="w-20 rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
          min={1}
          max={1440}
        />
      </label>

      <div className="space-y-2 pt-2">
        <button
          onClick={handleClearExpired}
          data-testid="btn-clear-expired-credentials"
          className="w-full rounded-lg bg-cp-bg px-3 py-2 text-sm text-cp-text hover:bg-cp-border"
        >
          Clear Expired Credentials Now
        </button>

        <button
          onClick={clearHistory}
          data-testid="btn-clear-all-history"
          className="w-full rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400 hover:bg-red-500/20"
        >
          Clear All History
        </button>
      </div>
    </div>
  );
}

// ContextPaste — Shortcut Settings

import { useSettings } from "../../hooks/useSettings";
import { Kbd } from "../shared/Kbd";

export function ShortcutSettings() {
  const { settings } = useSettings();

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">Keyboard Shortcuts</h3>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm text-cp-text">Quick Paste</span>
          <Kbd keys={settings.hotkeyQuickPaste.split("+")} />
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm text-cp-text">History Browser</span>
          <Kbd keys={settings.hotkeyHistory.split("+")} />
        </div>
      </div>

      <p className="text-xs text-cp-muted">
        Shortcut customization coming in a future update.
      </p>
    </div>
  );
}

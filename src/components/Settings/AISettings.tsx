// ContextPaste — AI Settings (Phase 3 placeholder)

import { useSettings } from "../../hooks/useSettings";

export function AISettings() {
  const { settings, updateSetting } = useSettings();

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">AI & Predictions</h3>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Enable predictions</span>
        <input
          type="checkbox"
          checked={settings.enablePredictions}
          onChange={(e) =>
            updateSetting("enablePredictions", String(e.target.checked))
          }
          data-testid="setting-enable-predictions"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      <label className="flex items-center justify-between">
        <div>
          <span className="text-sm text-cp-text">AI Provider</span>
          <p className="text-xs text-cp-muted">
            Semantic search requires an AI provider
          </p>
        </div>
        <select
          data-testid="setting-ai-provider"
          value={settings.aiProvider}
          onChange={(e) => updateSetting("aiProvider", e.target.value)}
          className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
        >
          <option value="local">Local (ONNX)</option>
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic</option>
          <option value="ollama">Ollama</option>
        </select>
      </label>

      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Enable semantic search</span>
        <input
          type="checkbox"
          checked={settings.enableSemanticSearch}
          onChange={(e) =>
            updateSetting("enableSemanticSearch", String(e.target.checked))
          }
          data-testid="setting-enable-semantic-search"
          className="h-4 w-4 accent-cp-accent"
          disabled
        />
      </label>

      <p className="text-xs text-cp-muted">
        Semantic search and BYOK API configuration coming in Phase 3.
      </p>
    </div>
  );
}

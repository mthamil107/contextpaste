// ContextPaste — AI Settings (Phase 3)

import { useState, useEffect } from "react";
import { useSettings } from "../../hooks/useSettings";
import { configureAiProvider, testAiConnection, getAiStatus, backfillEmbeddings } from "../../lib/tauri";
import type { AiStatus } from "../../lib/types";

export function AISettings() {
  const { settings, updateSetting } = useSettings();
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [modelName] = useState("");
  const [status, setStatus] = useState<AiStatus | null>(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [backfilling, setBackfilling] = useState(false);
  const [backfillResult, setBackfillResult] = useState<string | null>(null);

  useEffect(() => {
    getAiStatus().then(setStatus).catch(() => {});
  }, []);

  const handleProviderChange = async (provider: string) => {
    updateSetting("aiProvider", provider);
    setTestResult(null);
    setBackfillResult(null);

    if (provider === "local") {
      try {
        await configureAiProvider(provider);
        const s = await getAiStatus();
        setStatus(s);
      } catch (e) {
        console.error("Failed to configure local provider:", e);
      }
    }
  };

  const handleSaveAndConnect = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      await configureAiProvider(
        settings.aiProvider,
        apiKey || undefined,
        baseUrl || undefined,
        modelName || undefined,
      );
      const result = await testAiConnection();
      setTestResult(result);
      const s = await getAiStatus();
      setStatus(s);
    } catch (e) {
      setTestResult(`Error: ${e}`);
    } finally {
      setTesting(false);
    }
  };

  const handleBackfill = async () => {
    setBackfilling(true);
    setBackfillResult(null);
    try {
      const count = await backfillEmbeddings();
      setBackfillResult(`Indexed ${count} items`);
      const s = await getAiStatus();
      setStatus(s);
    } catch (e) {
      setBackfillResult(`Error: ${e}`);
    } finally {
      setBackfilling(false);
    }
  };

  const showApiKeyField = settings.aiProvider !== "local";
  const showBaseUrlField = settings.aiProvider === "ollama" || settings.aiProvider === "openai";

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">AI & Semantic Search</h3>

      {/* Enable predictions */}
      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Enable predictions</span>
        <input
          type="checkbox"
          checked={settings.enablePredictions}
          onChange={(e) => updateSetting("enablePredictions", String(e.target.checked))}
          data-testid="setting-enable-predictions"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      {/* AI Provider */}
      <label className="flex items-center justify-between">
        <div>
          <span className="text-sm text-cp-text">AI Provider</span>
          <p className="text-xs text-cp-muted">
            Powers semantic search and embeddings
          </p>
        </div>
        <select
          data-testid="setting-ai-provider"
          value={settings.aiProvider}
          onChange={(e) => handleProviderChange(e.target.value)}
          className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
        >
          <option value="local">Local (ONNX)</option>
          <option value="openai">OpenAI</option>
          <option value="ollama">Ollama</option>
        </select>
      </label>

      {/* API Key (for non-local providers) */}
      {showApiKeyField && (
        <label className="flex flex-col gap-1">
          <span className="text-sm text-cp-text">API Key</span>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={settings.aiProvider === "openai" ? "sk-..." : "Not required for Ollama"}
            data-testid="setting-ai-api-key"
            className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
          />
        </label>
      )}

      {/* Base URL */}
      {showBaseUrlField && (
        <label className="flex flex-col gap-1">
          <span className="text-sm text-cp-text">Base URL</span>
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder={settings.aiProvider === "ollama" ? "http://localhost:11434" : "https://api.openai.com"}
            data-testid="setting-ai-base-url"
            className="rounded border border-cp-border bg-cp-bg px-2 py-1 text-sm text-cp-text"
          />
        </label>
      )}

      {/* Save & Test Connection */}
      {showApiKeyField && (
        <button
          onClick={handleSaveAndConnect}
          disabled={testing}
          data-testid="btn-test-ai-connection"
          className="w-full rounded-lg bg-cp-accent/20 px-3 py-2 text-sm text-cp-accent hover:bg-cp-accent/30 disabled:opacity-50"
        >
          {testing ? "Testing..." : "Save & Test Connection"}
        </button>
      )}

      {testResult && (
        <p className={`text-xs ${testResult.startsWith("Error") ? "text-red-400" : "text-green-400"}`}>
          {testResult}
        </p>
      )}

      {/* Enable semantic search */}
      <label className="flex items-center justify-between">
        <div>
          <span className="text-sm text-cp-text">Enable semantic search</span>
          <p className="text-xs text-cp-muted">
            Search clipboard history by meaning, not just keywords
          </p>
        </div>
        <input
          type="checkbox"
          checked={settings.enableSemanticSearch}
          onChange={(e) => updateSetting("enableSemanticSearch", String(e.target.checked))}
          data-testid="setting-enable-semantic-search"
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      {/* AI Status */}
      {status && (
        <div className="rounded-lg border border-cp-border bg-cp-bg p-3 space-y-1">
          <div className="flex items-center gap-2">
            <div className={`h-2 w-2 rounded-full ${status.ready ? "bg-green-400" : "bg-yellow-400"}`} />
            <span className="text-xs text-cp-text font-medium">
              {status.ready ? "AI Ready" : "AI Not Initialized"}
            </span>
          </div>
          <p className="text-xs text-cp-muted truncate">Model: {status.modelName} ({status.dimension}D)</p>
          <p className="text-xs text-cp-muted">
            Indexed: {status.embeddedCount} / {status.totalItems} items
          </p>
        </div>
      )}

      {/* Re-index / Backfill */}
      <button
        onClick={handleBackfill}
        disabled={backfilling || !status?.ready}
        data-testid="btn-backfill-embeddings"
        className="w-full rounded-lg bg-cp-bg px-3 py-2 text-sm text-cp-text hover:bg-cp-border disabled:opacity-50"
      >
        {backfilling ? "Indexing..." : "Re-index All Items"}
      </button>

      {backfillResult && (
        <p className={`text-xs ${backfillResult.startsWith("Error") ? "text-red-400" : "text-green-400"}`}>
          {backfillResult}
        </p>
      )}
    </div>
  );
}

import { useState, useEffect } from "react";
import { useSettings } from "../../hooks/useSettings";
import { getPasteRules, createPasteRule, deletePasteRule, togglePasteRule } from "../../lib/tauri";
import type { PasteRule, ContentType } from "../../lib/types";
import { Plus, Trash2, ToggleLeft, ToggleRight } from "lucide-react";

const CONTENT_TYPES: ContentType[] = [
  "Url", "Email", "IpAddress", "Json", "Yaml", "Sql",
  "ShellCommand", "Code", "AwsArn", "ConnectionString",
  "FilePath", "Credential", "Markdown", "HtmlXml", "PlainText",
];

export function AutoPasteSettings() {
  const { settings, updateSetting } = useSettings();
  const [rules, setRules] = useState<PasteRule[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newRule, setNewRule] = useState({
    name: "",
    appPattern: "",
    windowTitlePattern: "",
    contextPattern: "",
    contentTypeFilter: "",
    actionType: "paste_recent_type",
    actionValue: "Credential",
    priority: 0,
    enabled: true,
  });

  useEffect(() => {
    getPasteRules().then(setRules).catch(() => {});
  }, []);

  const handleAddRule = async () => {
    if (!newRule.name.trim()) return;
    try {
      await createPasteRule({
        name: newRule.name,
        priority: newRule.priority,
        enabled: newRule.enabled,
        appPattern: newRule.appPattern || undefined,
        windowTitlePattern: newRule.windowTitlePattern || undefined,
        contextPattern: newRule.contextPattern || undefined,
        contentTypeFilter: newRule.contentTypeFilter || undefined,
        actionType: newRule.actionType,
        actionValue: newRule.actionValue,
      });
      const updated = await getPasteRules();
      setRules(updated);
      setShowAddForm(false);
      setNewRule({ name: "", appPattern: "", windowTitlePattern: "", contextPattern: "", contentTypeFilter: "", actionType: "paste_recent_type", actionValue: "Credential", priority: 0, enabled: true });
    } catch (e) {
      console.error("Failed to create rule:", e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deletePasteRule(id);
      setRules(rules.filter((r) => r.id !== id));
    } catch (e) {
      console.error("Failed to delete rule:", e);
    }
  };

  const handleToggle = async (id: string) => {
    try {
      await togglePasteRule(id);
      setRules(rules.map((r) => r.id === id ? { ...r, enabled: !r.enabled } : r));
    } catch (e) {
      console.error("Failed to toggle rule:", e);
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-cp-text">Auto-Paste</h3>
      <p className="text-xs text-cp-muted">
        Automatically paste the best-matching clipboard item based on screen context.
      </p>

      {/* Enable toggle */}
      <label className="flex items-center justify-between">
        <div>
          <span className="text-sm text-cp-text">Enable auto-paste</span>
          <p className="text-xs text-cp-muted">
            Ctrl+Shift+V auto-pastes when confident, shows overlay otherwise
          </p>
        </div>
        <input
          type="checkbox"
          checked={settings.enableAutoPaste}
          onChange={(e) => updateSetting("enableAutoPaste", String(e.target.checked))}
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      {/* Confidence threshold */}
      <label className="flex flex-col gap-1">
        <div className="flex items-center justify-between">
          <span className="text-sm text-cp-text">Confidence threshold</span>
          <span className="text-xs text-cp-muted">{Math.round(settings.autoPasteThreshold * 100)}%</span>
        </div>
        <input
          type="range"
          min="0.5"
          max="0.95"
          step="0.05"
          value={settings.autoPasteThreshold}
          onChange={(e) => updateSetting("autoPasteThreshold", e.target.value)}
          className="w-full accent-cp-accent"
        />
        <div className="flex justify-between text-[10px] text-cp-muted">
          <span>More auto-pastes</span>
          <span>Fewer, more accurate</span>
        </div>
      </label>

      {/* Toast toggle */}
      <label className="flex items-center justify-between">
        <span className="text-sm text-cp-text">Show notification on auto-paste</span>
        <input
          type="checkbox"
          checked={settings.showAutoPasteToast}
          onChange={(e) => updateSetting("showAutoPasteToast", String(e.target.checked))}
          className="h-4 w-4 accent-cp-accent"
        />
      </label>

      {/* Paste Rules */}
      <div className="border-t border-cp-border pt-4">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-semibold text-cp-text">Paste Rules</h4>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="flex items-center gap-1 rounded-md bg-cp-accent/10 px-2 py-1 text-xs text-cp-accent hover:bg-cp-accent/20"
          >
            <Plus size={12} />
            Add Rule
          </button>
        </div>
        <p className="text-xs text-cp-muted mb-3">
          Define rules to auto-paste specific items based on app, window title, or screen context.
        </p>

        {/* Add Rule Form */}
        {showAddForm && (
          <div className="rounded-lg border border-cp-border bg-cp-bg p-3 space-y-2 mb-3">
            <input
              type="text"
              placeholder="Rule name (e.g., Git token)"
              value={newRule.name}
              onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
              className="w-full rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
            />
            <input
              type="text"
              placeholder="App pattern (regex, e.g., Terminal|cmd)"
              value={newRule.appPattern}
              onChange={(e) => setNewRule({ ...newRule, appPattern: e.target.value })}
              className="w-full rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
            />
            <input
              type="text"
              placeholder="Window title pattern (regex)"
              value={newRule.windowTitlePattern}
              onChange={(e) => setNewRule({ ...newRule, windowTitlePattern: e.target.value })}
              className="w-full rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
            />
            <input
              type="text"
              placeholder="Context pattern (regex, e.g., password|token)"
              value={newRule.contextPattern}
              onChange={(e) => setNewRule({ ...newRule, contextPattern: e.target.value })}
              className="w-full rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
            />
            <div className="flex gap-2">
              <select
                value={newRule.actionType}
                onChange={(e) => setNewRule({ ...newRule, actionType: e.target.value })}
                className="flex-1 rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
              >
                <option value="paste_recent_type">Paste most recent by type</option>
                <option value="paste_item">Paste specific item</option>
              </select>
              <select
                value={newRule.actionValue}
                onChange={(e) => setNewRule({ ...newRule, actionValue: e.target.value })}
                className="flex-1 rounded border border-cp-border bg-cp-surface px-2 py-1 text-sm text-cp-text"
              >
                {CONTENT_TYPES.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAddRule}
                className="flex-1 rounded-md bg-cp-accent px-3 py-1.5 text-sm text-white hover:bg-cp-accent/90"
              >
                Save Rule
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                className="rounded-md border border-cp-border px-3 py-1.5 text-sm text-cp-muted hover:text-cp-text"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Rules list */}
        {rules.length === 0 && !showAddForm && (
          <p className="text-xs text-cp-muted italic">No rules yet. Add one to get started.</p>
        )}
        {rules.map((rule) => (
          <div
            key={rule.id}
            className="flex items-center justify-between rounded-lg border border-cp-border bg-cp-bg px-3 py-2 mb-2"
          >
            <div className="flex-1 min-w-0">
              <p className={`text-sm font-medium truncate ${rule.enabled ? "text-cp-text" : "text-cp-muted line-through"}`}>
                {rule.name}
              </p>
              <p className="text-[10px] text-cp-muted truncate">
                {[
                  rule.appPattern && `App: ${rule.appPattern}`,
                  rule.contextPattern && `Context: ${rule.contextPattern}`,
                  `→ ${rule.actionValue}`,
                ].filter(Boolean).join(" | ")}
              </p>
              <p className="text-[10px] text-cp-muted">
                Triggered {rule.timesTriggered} times
              </p>
            </div>
            <div className="flex items-center gap-1 ml-2">
              <button
                onClick={() => handleToggle(rule.id)}
                className="p-1 text-cp-muted hover:text-cp-text"
                title={rule.enabled ? "Disable" : "Enable"}
              >
                {rule.enabled ? <ToggleRight size={16} className="text-cp-accent" /> : <ToggleLeft size={16} />}
              </button>
              <button
                onClick={() => handleDelete(rule.id)}
                className="p-1 text-cp-muted hover:text-red-400"
                title="Delete"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

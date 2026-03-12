// ContextPaste — Settings Panel

import { useState } from "react";
import clsx from "clsx";
import { GeneralSettings } from "./GeneralSettings";
import { ShortcutSettings } from "./ShortcutSettings";
import { SecuritySettings } from "./SecuritySettings";
import { AISettings } from "./AISettings";

type SettingsTab = "general" | "shortcuts" | "security" | "ai";

const TABS: { id: SettingsTab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "shortcuts", label: "Shortcuts" },
  { id: "security", label: "Security" },
  { id: "ai", label: "AI" },
];

export function SettingsPanel() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");

  return (
    <div className="flex h-full flex-col" data-testid="settings-panel">
      {/* Tab bar */}
      <div className="flex border-b border-cp-border">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            data-testid={`settings-tab-${tab.id}`}
            onClick={() => setActiveTab(tab.id)}
            className={clsx(
              "px-4 py-2 text-sm font-medium transition-colors",
              activeTab === tab.id
                ? "border-b-2 border-cp-accent text-cp-accent"
                : "text-cp-muted hover:text-cp-text",
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === "general" && <div data-testid="settings-content-general"><GeneralSettings /></div>}
        {activeTab === "shortcuts" && <div data-testid="settings-content-shortcuts"><ShortcutSettings /></div>}
        {activeTab === "security" && <div data-testid="settings-content-security"><SecuritySettings /></div>}
        {activeTab === "ai" && <div data-testid="settings-content-ai"><AISettings /></div>}
      </div>
    </div>
  );
}

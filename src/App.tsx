// ContextPaste — Root Application Component

import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useClipboard } from "./hooks/useClipboard";
import { useSettings } from "./hooks/useSettings";
import { useUIStore } from "./stores/uiStore";
import { QuickPasteOverlay } from "./components/QuickPaste/QuickPasteOverlay";
import { HistoryPanel } from "./components/History/HistoryPanel";
import { SettingsPanel } from "./components/Settings/SettingsPanel";
import { ThemeToggle } from "./components/shared/ThemeToggle";
import { Clipboard, History, Settings } from "lucide-react";
import clsx from "clsx";

type NavItem = { id: "quick-paste" | "history" | "settings"; label: string; Icon: typeof Clipboard; testId: string };

const NAV_ITEMS: NavItem[] = [
  { id: "quick-paste", label: "Clipboard", Icon: Clipboard, testId: "nav-quick-paste" },
  { id: "history", label: "History", Icon: History, testId: "nav-history" },
  { id: "settings", label: "Settings", Icon: Settings, testId: "nav-settings" },
];

function App() {
  useClipboard();
  const { settings } = useSettings();
  const { currentView, setView, showOverlay } = useUIStore();

  // Apply theme on load
  useEffect(() => {
    const applyTheme = () => {
      const isDark =
        settings.theme === "dark" ||
        (settings.theme === "system" &&
          window.matchMedia("(prefers-color-scheme: dark)").matches);
      document.documentElement.classList.toggle("dark", isDark);
    };
    applyTheme();
  }, [settings.theme]);

  // Listen for tray navigation events
  useEffect(() => {
    const unlistenHistory = listen("nav:history", () => setView("history"));
    const unlistenSettings = listen("nav:settings", () => setView("settings"));

    return () => {
      unlistenHistory.then((fn) => fn());
      unlistenSettings.then((fn) => fn());
    };
  }, [setView]);

  return (
    <div className="flex h-screen w-screen flex-col bg-cp-surface" data-testid="app-container">
      {/* Quick Paste Overlay (floats above everything) */}
      <QuickPasteOverlay />

      {/* Nav bar */}
      <div className="flex items-center justify-between border-b border-cp-border px-2 py-1" data-testid="nav-bar">
        <div className="flex items-center gap-1">
          {NAV_ITEMS.map(({ id, label, Icon, testId }) => (
            <button
              key={id}
              data-testid={testId}
              onClick={() => setView(id)}
              className={clsx(
                "flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
                currentView === id
                  ? "bg-cp-accent/10 text-cp-accent"
                  : "text-cp-muted hover:text-cp-text",
              )}
            >
              <Icon size={14} />
              {label}
            </button>
          ))}
        </div>
        <ThemeToggle />
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-hidden">
        {currentView === "quick-paste" && (
          <div className="flex h-full flex-col items-center justify-center p-4 text-center" data-testid="view-quick-paste">
            <Clipboard size={48} className="mb-4 text-cp-muted" />
            <h1 className="text-lg font-bold text-cp-text">ContextPaste</h1>
            <p className="mt-1 text-sm text-cp-muted">
              AI-Powered Smart Clipboard
            </p>
            <button
              onClick={showOverlay}
              data-testid="open-quick-paste-btn"
              className="mt-6 rounded-lg bg-cp-accent px-4 py-2 text-sm font-medium text-white hover:bg-cp-accent/90"
            >
              Open Quick Paste
            </button>
            <p className="mt-2 text-xs text-cp-muted">
              or press Ctrl+Shift+V anywhere
            </p>
          </div>
        )}
        {currentView === "history" && <HistoryPanel />}
        {currentView === "settings" && <SettingsPanel />}
      </div>
    </div>
  );
}

export default App;

// ContextPaste — Root Application Component

import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useClipboard } from "./hooks/useClipboard";
import { useSettings } from "./hooks/useSettings";
import { useUIStore } from "./stores/uiStore";
import { QuickPasteOverlay } from "./components/QuickPaste/QuickPasteOverlay";
import { HistoryPanel } from "./components/History/HistoryPanel";
import { SettingsPanel } from "./components/Settings/SettingsPanel";
import { ThemeToggle } from "./components/shared/ThemeToggle";
import { AutoPasteToast } from "./components/shared/AutoPasteToast";
import { Clipboard, History, Settings, Minus, Square, X } from "lucide-react";
import clsx from "clsx";

type NavItem = { id: "quick-paste" | "history" | "settings"; label: string; Icon: typeof Clipboard; testId: string };

const NAV_ITEMS: NavItem[] = [
  { id: "quick-paste", label: "Clipboard", Icon: Clipboard, testId: "nav-quick-paste" },
  { id: "history", label: "History", Icon: History, testId: "nav-history" },
  { id: "settings", label: "Settings", Icon: Settings, testId: "nav-settings" },
];

function TitleBar() {
  const appWindow = getCurrentWindow();

  return (
    <div
      className="flex h-9 items-center justify-between border-b border-cp-border bg-cp-surface select-none"
      data-testid="title-bar"
    >
      {/* Drag region — takes up all available space */}
      <div
        className="flex flex-1 items-center gap-2 pl-3"
        onMouseDown={(e) => {
          if (e.buttons === 1) {
            e.preventDefault();
            appWindow.startDragging();
          }
        }}
      >
        <div className="flex h-5 w-5 items-center justify-center rounded bg-cp-accent text-[10px] font-bold text-white">
          CP
        </div>
        <span className="text-xs font-medium text-cp-text">ContextPaste</span>
      </div>

      {/* Window controls */}
      <div className="flex h-full">
        <button
          onClick={() => appWindow.minimize()}
          className="flex h-full w-11 items-center justify-center text-cp-muted hover:bg-cp-border/50 hover:text-cp-text transition-colors"
          data-testid="btn-minimize"
          title="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          onClick={() => appWindow.toggleMaximize()}
          className="flex h-full w-11 items-center justify-center text-cp-muted hover:bg-cp-border/50 hover:text-cp-text transition-colors"
          data-testid="btn-maximize"
          title="Maximize"
        >
          <Square size={12} />
        </button>
        <button
          onClick={() => appWindow.hide()}
          className="flex h-full w-11 items-center justify-center text-cp-muted hover:bg-red-500 hover:text-white transition-colors"
          data-testid="btn-close"
          title="Close to tray"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
}

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

  // Listen for shortcut and tray navigation events
  useEffect(() => {
    const unlistenQuickPaste = listen("shortcut:quick-paste", () => {
      showOverlay();
      setView("quick-paste");
    });
    const unlistenHistoryShortcut = listen("shortcut:history", () => {
      setView("history");
    });
    const unlistenHistory = listen("nav:history", () => setView("history"));
    const unlistenSettings = listen("nav:settings", () => setView("settings"));

    return () => {
      unlistenQuickPaste.then((fn) => fn());
      unlistenHistoryShortcut.then((fn) => fn());
      unlistenHistory.then((fn) => fn());
      unlistenSettings.then((fn) => fn());
    };
  }, [setView, showOverlay]);

  return (
    <div className="flex h-screen w-screen flex-col bg-cp-surface" data-testid="app-container">
      {/* Custom title bar with window controls */}
      <TitleBar />

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
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-cp-accent/10 mb-4">
              <Clipboard size={32} className="text-cp-accent" />
            </div>
            <h1 className="text-lg font-bold text-cp-text">ContextPaste</h1>
            <p className="mt-1 text-sm text-cp-muted">
              AI-Powered Smart Clipboard Manager
            </p>
            <p className="mt-3 text-xs text-cp-muted max-w-xs">
              Copy anything from any app. ContextPaste captures, classifies, and learns your paste patterns.
            </p>
            <button
              onClick={showOverlay}
              data-testid="open-quick-paste-btn"
              className="mt-6 rounded-lg bg-cp-accent px-5 py-2.5 text-sm font-medium text-white hover:bg-cp-accent/90 shadow-md hover:shadow-lg transition-all"
            >
              Open Quick Paste
            </button>
            <p className="mt-2 text-xs text-cp-muted">
              or press <kbd className="rounded border border-cp-border bg-cp-bg px-1.5 py-0.5 text-[10px] font-mono">Ctrl+Shift+V</kbd> anywhere
            </p>
          </div>
        )}
        {currentView === "history" && <HistoryPanel />}
        {currentView === "settings" && <SettingsPanel />}
      </div>

      {/* Status bar */}
      <div className="flex items-center justify-between border-t border-cp-border px-3 py-1 text-[10px] text-cp-muted">
        <span>v0.1.0</span>
        <span>GPL v3 — Free Forever</span>
      </div>

      {/* Auto-paste notification toast */}
      <AutoPasteToast />
    </div>
  );
}

export default App;

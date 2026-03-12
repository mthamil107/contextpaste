// ContextPaste — Theme Toggle

import { Sun, Moon } from "lucide-react";
import { useSettings } from "../../hooks/useSettings";

export function ThemeToggle() {
  const { settings, updateSetting } = useSettings();

  const isDark =
    settings.theme === "dark" ||
    (settings.theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  const toggle = () => {
    const next = isDark ? "light" : "dark";
    updateSetting("theme", next);
    document.documentElement.classList.toggle("dark", next === "dark");
  };

  return (
    <button
      onClick={toggle}
      data-testid="theme-toggle"
      className="rounded-md p-1.5 text-cp-muted hover:bg-cp-border hover:text-cp-text"
      title={`Switch to ${isDark ? "light" : "dark"} mode`}
    >
      {isDark ? <Sun size={16} /> : <Moon size={16} />}
    </button>
  );
}

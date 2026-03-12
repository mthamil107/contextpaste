// ContextPaste — Keyboard Shortcut Hook

import { useEffect } from "react";

type KeyHandler = (e: KeyboardEvent) => void;

export function useShortcut(key: string, handler: KeyHandler, deps: unknown[] = []) {
  useEffect(() => {
    const listener = (e: KeyboardEvent) => {
      if (e.key === key) {
        handler(e);
      }
    };
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, [key, handler, ...deps]);
}

export function useKeyboardNav(opts: {
  onUp: () => void;
  onDown: () => void;
  onEnter: () => void;
  onEscape: () => void;
  onTab?: () => void;
  enabled: boolean;
}) {
  useEffect(() => {
    if (!opts.enabled) return;

    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
          e.preventDefault();
          opts.onUp();
          break;
        case "ArrowDown":
          e.preventDefault();
          opts.onDown();
          break;
        case "Enter":
          e.preventDefault();
          opts.onEnter();
          break;
        case "Escape":
          e.preventDefault();
          opts.onEscape();
          break;
        case "Tab":
          if (opts.onTab) {
            e.preventDefault();
            opts.onTab();
          }
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [opts.enabled, opts.onUp, opts.onDown, opts.onEnter, opts.onEscape, opts.onTab]);
}

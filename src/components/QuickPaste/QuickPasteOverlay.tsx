// ContextPaste — Quick Paste Overlay (Primary UI)
// Triggered by Ctrl+Shift+V. Uses cmdk-style keyboard navigation.

import { useCallback, useEffect, useMemo, useState } from "react";
import { useClipboardStore } from "../../stores/clipboardStore";
import { useUIStore } from "../../stores/uiStore";
import { useSearch } from "../../hooks/useSearch";
import { useKeyboardNav } from "../../hooks/useShortcut";
import { onChainDetected } from "../../lib/tauri";
import { SearchBar } from "./SearchBar";
import { ClipItemRow } from "./ClipItem";
import { ChainIndicator } from "./ChainIndicator";
import { Kbd } from "../shared/Kbd";
import { DEFAULTS } from "../../lib/constants";

export function QuickPasteOverlay() {
  const { items, predictions } = useClipboardStore();
  const { pasteItem } = useClipboardStore();
  const {
    overlayVisible,
    hideOverlay,
    selectedIndex,
    setSelectedIndex,
    moveSelection,
    activeChain,
    setActiveChain,
  } = useUIStore();
  const { query, setQuery, results, searching, isSemanticActive } = useSearch();

  // Ghost paste flash state: holds the id of the item being flashed
  const [flashId, setFlashId] = useState<string | null>(null);

  // Listen for workflow chain events
  useEffect(() => {
    const unlistenPromise = onChainDetected((chain) => {
      setActiveChain(chain);
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [setActiveChain]);

  // Determine which items to show
  const displayItems = useMemo(() => {
    if (query.trim()) {
      return results;
    }
    // Show predictions first, fall back to recent items
    if (predictions.length > 0) {
      return predictions.map((p) => p.item);
    }
    return items.slice(0, DEFAULTS.OVERLAY_MAX_ITEMS);
  }, [query, results, predictions, items]);

  const handlePaste = useCallback(
    async (index: number) => {
      const item = displayItems[index];
      if (item) {
        await pasteItem(item.id);
        hideOverlay();
      }
    },
    [displayItems, pasteItem, hideOverlay],
  );

  // Ghost paste: copy to clipboard without dismissing overlay
  const handleGhostPaste = useCallback(
    async (index: number) => {
      const item = displayItems[index];
      if (item) {
        await pasteItem(item.id);
        setFlashId(item.id);
        setTimeout(() => setFlashId(null), 600);
      }
    },
    [displayItems, pasteItem],
  );

  useKeyboardNav({
    enabled: overlayVisible,
    onUp: () => moveSelection(-1, displayItems.length),
    onDown: () => moveSelection(1, displayItems.length),
    onEnter: () => handlePaste(selectedIndex),
    onEscape: hideOverlay,
    onTab: () => handleGhostPaste(selectedIndex),
  });

  if (!overlayVisible) return null;

  return (
    <div className="animate-fade-in fixed inset-0 z-50" data-testid="quick-paste-backdrop" onClick={hideOverlay}>
      <div
        className="mx-auto mt-20 w-[420px] animate-slide-up overflow-hidden rounded-xl border border-cp-border bg-cp-surface/95 shadow-2xl backdrop-blur-xl"
        data-testid="quick-paste-overlay"
        onClick={(e) => e.stopPropagation()}
      >
        <SearchBar value={query} onChange={setQuery} isSemanticActive={isSemanticActive} />

        {/* Chain indicator */}
        {activeChain && <ChainIndicator chain={activeChain} />}

        <div className="max-h-[380px] overflow-y-auto py-1">
          {searching && (
            <div className="px-3 py-4 text-center text-sm text-cp-muted">
              Searching...
            </div>
          )}

          {!searching && displayItems.length === 0 && (
            <div className="px-3 py-8 text-center text-sm text-cp-muted" data-testid="quick-paste-empty">
              {query ? "No results found" : "No clipboard history yet"}
            </div>
          )}

          {!searching &&
            displayItems.map((item, index) => (
              <ClipItemRow
                key={item.id}
                item={item}
                selected={index === selectedIndex}
                flash={flashId === item.id}
                onSelect={() => setSelectedIndex(index)}
                onPaste={() => handlePaste(index)}
              />
            ))}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between border-t border-cp-border px-3 py-1.5" data-testid="quick-paste-footer">
          <div className="flex items-center gap-3 text-[10px] text-cp-muted">
            <span className="flex items-center gap-1">
              <Kbd keys={["↑", "↓"]} /> navigate
            </span>
            <span className="flex items-center gap-1">
              <Kbd keys={["↵"]} /> paste
            </span>
            <span className="flex items-center gap-1">
              <Kbd keys={["tab"]} /> ghost
            </span>
            <span className="flex items-center gap-1">
              <Kbd keys={["esc"]} /> close
            </span>
          </div>
          <span className="text-[10px] text-cp-muted">
            {displayItems.length} items
          </span>
        </div>
      </div>
    </div>
  );
}

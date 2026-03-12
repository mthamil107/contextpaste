// ContextPaste — Full History Browser

import { useState, useMemo } from "react";
import type { ContentType } from "../../lib/types";
import { useClipboardStore } from "../../stores/clipboardStore";
import { useSearch } from "../../hooks/useSearch";
import { FilterBar } from "./FilterBar";
import { ClipDetail } from "./ClipDetail";
import { ClipItemRow } from "../QuickPaste/ClipItem";

export function HistoryPanel() {
  const { items } = useClipboardStore();
  const { pasteItem } = useClipboardStore();
  const { query, setQuery, results } = useSearch();
  const [activeFilter, setActiveFilter] = useState<ContentType | "all">("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const displayItems = useMemo(() => {
    let source = query.trim() ? results : items;
    if (activeFilter !== "all") {
      source = source.filter((item) => item.contentType === activeFilter);
    }
    return source;
  }, [query, results, items, activeFilter]);

  const selectedItem = useMemo(
    () => displayItems.find((i) => i.id === selectedId) ?? null,
    [displayItems, selectedId],
  );

  return (
    <div className="flex h-full flex-col" data-testid="history-panel">
      <FilterBar
        searchQuery={query}
        onSearchChange={setQuery}
        activeFilter={activeFilter}
        onFilterChange={setActiveFilter}
      />

      <div className="flex flex-1 overflow-hidden">
        {/* Item list */}
        <div className="flex-1 overflow-y-auto" data-testid="history-item-list">
          {displayItems.length === 0 && (
            <div className="px-3 py-8 text-center text-sm text-cp-muted" data-testid="history-empty">
              {query ? "No results found" : "No clipboard history"}
            </div>
          )}
          {displayItems.map((item) => (
            <ClipItemRow
              key={item.id}
              item={item}
              selected={item.id === selectedId}
              onSelect={() => setSelectedId(item.id)}
              onPaste={() => pasteItem(item.id)}
            />
          ))}
        </div>

        {/* Detail panel */}
        {selectedItem && (
          <div className="w-[240px] border-l border-cp-border">
            <ClipDetail
              item={selectedItem}
              onClose={() => setSelectedId(null)}
            />
          </div>
        )}
      </div>
    </div>
  );
}

// ContextPaste — Search Hook

import { useState, useEffect, useCallback, useRef } from "react";
import type { ClipItem } from "../lib/types";
import { searchItems, semanticSearch } from "../lib/tauri";
import { useSettingsStore } from "../stores/settingsStore";
import { DEFAULTS } from "../lib/constants";

function isNaturalLanguage(query: string): boolean {
  return query.includes(" ") && query.length > 10;
}

export function useSearch() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ClipItem[]>([]);
  const [searching, setSearching] = useState(false);
  const [isSemanticActive, setIsSemanticActive] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const search = useCallback(async (q: string) => {
    if (!q.trim()) {
      setResults([]);
      setSearching(false);
      setIsSemanticActive(false);
      return;
    }
    setSearching(true);
    try {
      const enableSemanticSearch = useSettingsStore.getState().settings.enableSemanticSearch;
      const useSemantic = enableSemanticSearch && isNaturalLanguage(q);
      setIsSemanticActive(useSemantic);

      const items = useSemantic
        ? await semanticSearch(q, 20)
        : await searchItems(q, 20);
      setResults(items);
    } catch (e) {
      console.error("Search failed:", e);
    } finally {
      setSearching(false);
    }
  }, []);

  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      search(query);
    }, DEFAULTS.SEARCH_DEBOUNCE_MS);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [query, search]);

  return { query, setQuery, results, searching, isSemanticActive };
}

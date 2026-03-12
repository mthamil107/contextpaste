// ContextPaste — Search Hook

import { useState, useEffect, useCallback, useRef } from "react";
import type { ClipItem } from "../lib/types";
import { searchItems } from "../lib/tauri";
import { DEFAULTS } from "../lib/constants";

export function useSearch() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ClipItem[]>([]);
  const [searching, setSearching] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const search = useCallback(async (q: string) => {
    if (!q.trim()) {
      setResults([]);
      setSearching(false);
      return;
    }
    setSearching(true);
    try {
      const items = await searchItems(q, 20);
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

  return { query, setQuery, results, searching };
}

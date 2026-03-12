// ContextPaste — Clipboard Hook

import { useEffect } from "react";
import { useClipboardStore } from "../stores/clipboardStore";
import { onNewClipItem, onClipboardError } from "../lib/tauri";

export function useClipboard() {
  const { items, predictions, loading, error, fetchItems, fetchPredictions, addItem } =
    useClipboardStore();

  useEffect(() => {
    fetchItems();
    fetchPredictions();

    const unlistenNew = onNewClipItem((item) => {
      addItem(item);
      fetchPredictions();
    });

    const unlistenError = onClipboardError((err) => {
      console.error("Clipboard error:", err);
    });

    return () => {
      unlistenNew.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  return { items, predictions, loading, error };
}

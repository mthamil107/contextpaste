import { useEffect, useState } from "react";
import { onAutoPasteSuccess } from "../../lib/tauri";
import type { AutoPasteResult } from "../../lib/types";
import { Zap, X } from "lucide-react";

export function AutoPasteToast() {
  const [result, setResult] = useState<AutoPasteResult | null>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const unlisten = onAutoPasteSuccess((res) => {
      setResult(res);
      setVisible(true);
      // Auto-dismiss after 3 seconds
      setTimeout(() => setVisible(false), 3000);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  if (!visible || !result || !result.item) return null;

  const preview = result.item.content.length > 60
    ? result.item.content.substring(0, 60) + "..."
    : result.item.content;

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-slide-up">
      <div className="flex items-center gap-3 rounded-lg border border-cp-border bg-cp-surface px-4 py-3 shadow-xl max-w-sm">
        <Zap size={16} className="text-cp-accent shrink-0" />
        <div className="flex-1 min-w-0">
          <p className="text-xs font-medium text-cp-text truncate">
            Auto-pasted
          </p>
          <p className="text-[10px] text-cp-muted truncate">{preview}</p>
          {result.matchedRule && (
            <p className="text-[10px] text-cp-accent">Rule: {result.matchedRule}</p>
          )}
        </div>
        <span className="text-[10px] text-cp-muted shrink-0">
          {Math.round(result.confidence * 100)}%
        </span>
        <button
          onClick={() => setVisible(false)}
          className="text-cp-muted hover:text-cp-text shrink-0"
        >
          <X size={12} />
        </button>
      </div>
    </div>
  );
}

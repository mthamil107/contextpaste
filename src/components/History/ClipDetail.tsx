// ContextPaste — Clipboard Item Detail View

import { Pin, Star, Trash2, Copy } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import type { ClipItem } from "../../lib/types";
import { TypeBadge } from "../QuickPaste/TypeBadge";
import { useClipboardStore } from "../../stores/clipboardStore";

interface ClipDetailProps {
  item: ClipItem | null;
  onClose: () => void;
}

export function ClipDetail({ item, onClose }: ClipDetailProps) {
  const { togglePin, toggleStar, removeItem, pasteItem } = useClipboardStore();

  if (!item) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-cp-muted">
        Select an item to view details
      </div>
    );
  }

  const timeAgo = (() => {
    try {
      const date = new Date(item.createdAt.replace(" ", "T") + "Z");
      return formatDistanceToNow(date, { addSuffix: true });
    } catch {
      return item.createdAt;
    }
  })();

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-cp-border p-3">
        <div className="flex items-center gap-2">
          <TypeBadge type={item.contentType} />
          <span className="text-xs text-cp-muted">{timeAgo}</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => togglePin(item.id)}
            className={`rounded p-1 hover:bg-cp-border ${item.isPinned ? "text-cp-accent" : "text-cp-muted"}`}
            title={item.isPinned ? "Unpin" : "Pin"}
          >
            <Pin size={14} />
          </button>
          <button
            onClick={() => toggleStar(item.id)}
            className={`rounded p-1 hover:bg-cp-border ${item.isStarred ? "text-yellow-400" : "text-cp-muted"}`}
            title={item.isStarred ? "Unstar" : "Star"}
          >
            <Star size={14} />
          </button>
          <button
            onClick={() => pasteItem(item.id)}
            className="rounded p-1 text-cp-muted hover:bg-cp-border hover:text-cp-text"
            title="Copy to clipboard"
          >
            <Copy size={14} />
          </button>
          <button
            onClick={() => {
              removeItem(item.id);
              onClose();
            }}
            className="rounded p-1 text-cp-muted hover:bg-red-500/10 hover:text-red-400"
            title="Delete"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-3">
        <pre className="whitespace-pre-wrap break-all text-sm text-cp-text">
          {item.content}
        </pre>
      </div>

      {/* Footer metadata */}
      <div className="border-t border-cp-border px-3 py-2 text-[10px] text-cp-muted">
        <div className="flex justify-between">
          <span>{item.contentLength} chars</span>
          <span>Pasted {item.pasteCount} times</span>
        </div>
        {item.sourceApp && <div>Source: {item.sourceApp}</div>}
      </div>
    </div>
  );
}

// ContextPaste — Single Clipboard Item Row

import { Pin, Star, Lock } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import clsx from "clsx";
import type { ClipItem as ClipItemType } from "../../lib/types";
import { TypeBadge } from "./TypeBadge";
import { DEFAULTS, MASK_CHAR } from "../../lib/constants";

interface ClipItemProps {
  item: ClipItemType;
  selected: boolean;
  flash?: boolean;
  onSelect: () => void;
  onPaste: () => void;
}

function maskCredential(content: string): string {
  if (content.length <= 8) return MASK_CHAR.repeat(8);
  return (
    content.slice(0, 4) + MASK_CHAR.repeat(8) + content.slice(-4)
  );
}

function getPreview(item: ClipItemType): string {
  if (item.isCredential) {
    return maskCredential(item.content);
  }
  const firstLine = item.content.split("\n")[0];
  if (firstLine.length > DEFAULTS.PREVIEW_MAX_LENGTH) {
    return firstLine.slice(0, DEFAULTS.PREVIEW_MAX_LENGTH) + "...";
  }
  return firstLine;
}

function getTimeAgo(dateStr: string): string {
  try {
    // Handle both ISO and SQLite datetime formats
    const date = new Date(dateStr.replace(" ", "T") + "Z");
    return formatDistanceToNow(date, { addSuffix: true });
  } catch {
    return "";
  }
}

export function ClipItemRow({ item, selected, flash, onSelect, onPaste }: ClipItemProps) {
  return (
    <div
      data-testid="clip-item"
      className={clsx(
        "flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 transition-colors",
        selected
          ? "bg-cp-accent/10 text-cp-text"
          : "text-cp-text hover:bg-cp-border/50",
        flash && "animate-ghost-flash",
      )}
      onClick={onSelect}
      onDoubleClick={onPaste}
      role="option"
      aria-selected={selected}
    >
      {/* Ghost paste indicator */}
      {flash && (
        <span className="absolute right-3 text-[10px] font-medium text-green-500 animate-ghost-flash">
          Copied!
        </span>
      )}

      {/* Status icons */}
      <div className="flex shrink-0 items-center gap-1">
        {item.isPinned && <Pin size={12} className="text-cp-accent" />}
        {item.isStarred && <Star size={12} className="fill-yellow-400 text-yellow-400" />}
        {item.isCredential && <Lock size={12} className="text-red-400" />}
      </div>

      {/* Type badge */}
      <TypeBadge type={item.contentType} />

      {/* Content preview */}
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm">{getPreview(item)}</p>
      </div>

      {/* Metadata */}
      <div className="flex shrink-0 items-center gap-2">
        {item.sourceApp && (
          <span className="text-[10px] text-cp-muted">{item.sourceApp}</span>
        )}
        <span className="text-[10px] text-cp-muted">
          {getTimeAgo(item.createdAt)}
        </span>
      </div>
    </div>
  );
}

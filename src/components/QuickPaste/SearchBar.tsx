// ContextPaste — Inline Search Bar

import { Search, Sparkles } from "lucide-react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  isSemanticActive?: boolean;
}

export function SearchBar({
  value,
  onChange,
  placeholder = "Search clipboard...",
  isSemanticActive = false,
}: SearchBarProps) {
  return (
    <div className="flex items-center gap-2 border-b border-cp-border px-3 py-2" data-testid="quick-paste-search">
      <Search size={14} className="shrink-0 text-cp-muted" />
      <input
        data-testid="search-input"
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full bg-transparent text-sm text-cp-text outline-none placeholder:text-cp-muted"
        autoFocus
      />
      {isSemanticActive && (
        <span
          className="flex shrink-0 items-center gap-1 rounded bg-cp-accent/20 px-1.5 py-0.5 text-xs text-cp-accent"
          data-testid="semantic-search-badge"
        >
          <Sparkles size={10} />
          AI
        </span>
      )}
    </div>
  );
}

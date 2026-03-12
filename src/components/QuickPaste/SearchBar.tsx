// ContextPaste — Inline Search Bar

import { Search } from "lucide-react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SearchBar({
  value,
  onChange,
  placeholder = "Search clipboard...",
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
    </div>
  );
}

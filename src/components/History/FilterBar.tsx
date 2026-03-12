// ContextPaste — History Filter Bar

import { Search } from "lucide-react";
import type { ContentType } from "../../lib/types";

const FILTER_TYPES: { value: ContentType | "all"; label: string }[] = [
  { value: "all", label: "All" },
  { value: "Url", label: "URLs" },
  { value: "Code", label: "Code" },
  { value: "Json", label: "JSON" },
  { value: "Sql", label: "SQL" },
  { value: "ShellCommand", label: "Shell" },
  { value: "Credential", label: "Secrets" },
];

interface FilterBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  activeFilter: ContentType | "all";
  onFilterChange: (filter: ContentType | "all") => void;
}

export function FilterBar({
  searchQuery,
  onSearchChange,
  activeFilter,
  onFilterChange,
}: FilterBarProps) {
  return (
    <div className="space-y-2 border-b border-cp-border p-3" data-testid="history-filter-bar">
      <div className="flex items-center gap-2 rounded-lg bg-cp-bg px-3 py-1.5">
        <Search size={14} className="text-cp-muted" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Search history..."
          data-testid="history-search-input"
          className="w-full bg-transparent text-sm text-cp-text outline-none placeholder:text-cp-muted"
        />
      </div>
      <div className="flex flex-wrap gap-1">
        {FILTER_TYPES.map((f) => (
          <button
            key={f.value}
            data-testid={`history-filter-btn-${f.label.toLowerCase()}`}
            onClick={() => onFilterChange(f.value)}
            className={`rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors ${
              activeFilter === f.value
                ? "bg-cp-accent text-white"
                : "bg-cp-bg text-cp-muted hover:text-cp-text"
            }`}
          >
            {f.label}
          </button>
        ))}
      </div>
    </div>
  );
}

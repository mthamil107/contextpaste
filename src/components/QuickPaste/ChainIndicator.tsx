// ContextPaste — Chain Queue Indicator
// Shows when a workflow chain is detected, displaying queued items.

import { GitBranch } from "lucide-react";
import type { WorkflowChain } from "../../lib/types";
import { TypeBadge } from "./TypeBadge";

interface ChainIndicatorProps {
  chain: WorkflowChain;
}

export function ChainIndicator({ chain }: ChainIndicatorProps) {
  return (
    <div
      data-testid="chain-indicator"
      className="flex items-center gap-2 border-b border-cp-border bg-cp-accent/5 px-3 py-1.5"
    >
      <GitBranch size={14} className="shrink-0 text-cp-accent" />
      <span className="text-xs font-medium text-cp-accent">
        Chain: {chain.items.length} items queued
      </span>

      {/* Horizontal mini-preview of chain items */}
      <div className="flex items-center gap-1 overflow-hidden">
        {chain.items.map((chainItem, idx) => (
          <div key={idx} className="flex shrink-0 items-center gap-1">
            {idx > 0 && (
              <span className="text-[10px] text-cp-muted">&rarr;</span>
            )}
            <TypeBadge type={chainItem.contentType} />
            <span className="max-w-[60px] truncate text-[10px] text-cp-muted">
              {chainItem.preview}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ContextPaste — Prediction Indicator

import { Sparkles } from "lucide-react";

interface PredictionIndicatorProps {
  reason: string;
  score: number;
}

export function PredictionIndicator({ reason }: PredictionIndicatorProps) {
  if (reason === "pinned") return null;

  return (
    <span className="inline-flex items-center gap-0.5 text-[10px] text-cp-accent">
      <Sparkles size={10} />
      <span className="capitalize">{reason}</span>
    </span>
  );
}

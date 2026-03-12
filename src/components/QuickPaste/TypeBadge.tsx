// ContextPaste — Content Type Badge

import type { ContentType } from "../../lib/types";
import { CONTENT_TYPE_CONFIG } from "../../lib/types";

const BADGE_COLORS: Record<ContentType, string> = {
  Url: "#3B82F6",
  Email: "#06B6D4",
  IpAddress: "#84CC16",
  Json: "#10B981",
  Yaml: "#CB171E",
  Sql: "#8B5CF6",
  ShellCommand: "#F59E0B",
  Code: "#6366F1",
  AwsArn: "#FF9900",
  ConnectionString: "#D946EF",
  FilePath: "#78716C",
  Credential: "#EF4444",
  Markdown: "#64748B",
  HtmlXml: "#E34F26",
  PlainText: "#94A3B8",
};

interface TypeBadgeProps {
  type: ContentType;
}

export function TypeBadge({ type }: TypeBadgeProps) {
  const config = CONTENT_TYPE_CONFIG[type];

  return (
    <span
      className="inline-flex shrink-0 items-center rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase leading-none text-white"
      style={{ backgroundColor: BADGE_COLORS[type] }}
    >
      {config.label}
    </span>
  );
}

// ContextPaste — Keyboard Shortcut Badge

interface KbdProps {
  keys: string[];
}

export function Kbd({ keys }: KbdProps) {
  return (
    <span className="inline-flex items-center gap-0.5">
      {keys.map((key, i) => (
        <kbd
          key={i}
          className="rounded border border-cp-border bg-cp-bg px-1.5 py-0.5 text-[10px] font-medium text-cp-muted"
        >
          {key}
        </kbd>
      ))}
    </span>
  );
}

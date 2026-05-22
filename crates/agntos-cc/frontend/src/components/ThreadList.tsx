import { useCallback, useEffect, useState } from "react";
import { ChevronLeft, ChevronRight, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

export interface PiSession {
  path: string;
  title: string;
  modified: number;
}

interface ThreadListProps {
  activePath: string | null;
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
  onSelect: (session: PiSession) => void;
  onNew: () => void;
}

function formatWhen(ts: number) {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const now = Date.now();
  const diff = now - d.getTime();
  if (diff < 86_400_000) {
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  return d.toLocaleDateString([], { month: "short", day: "numeric" });
}

export default function ThreadList({
  activePath,
  collapsed,
  onCollapsedChange,
  onSelect,
  onNew,
}: ThreadListProps) {
  const [sessions, setSessions] = useState<PiSession[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!window.__TAURI__) return;
    setLoading(true);
    try {
      const list = (await window.__TAURI__.core.invoke("list_sessions")) as PiSession[];
      setSessions(Array.isArray(list) ? list : []);
    } catch {
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh, activePath]);

  if (collapsed) {
    return (
      <div className="flex w-10 shrink-0 flex-col items-center border-r border-border bg-card/30 py-2 gap-1">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-8"
          onClick={() => onCollapsedChange(false)}
          aria-label="Expand sessions"
        >
          <ChevronRight className="size-4" />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-8"
          onClick={onNew}
          aria-label="New session"
        >
          <Plus className="size-4" />
        </Button>
      </div>
    );
  }

  return (
    <aside className="flex w-56 shrink-0 flex-col border-r border-border bg-card/30">
      <div className="flex h-11 items-center gap-1 border-b border-border px-2">
        <span className="flex-1 text-xs font-semibold text-muted-foreground uppercase tracking-wide px-1">
          Sessions
        </span>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={onNew}
          aria-label="New session"
        >
          <Plus className="size-4" />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={() => onCollapsedChange(true)}
          aria-label="Collapse sessions"
        >
          <ChevronLeft className="size-4" />
        </Button>
      </div>
      <ScrollArea className="flex-1">
        <div className="flex flex-col gap-0.5 p-2">
          {loading && (
            <p className="px-2 py-3 text-xs text-muted-foreground">Loading…</p>
          )}
          {!loading && sessions.length === 0 && (
            <p className="px-2 py-3 text-xs text-muted-foreground">No sessions yet</p>
          )}
          {sessions.map((s) => {
            const active = activePath === s.path;
            return (
              <button
                key={s.path}
                type="button"
                onClick={() => onSelect(s)}
                className={cn(
                  "flex w-full flex-col items-start rounded-lg px-2.5 py-2 text-left text-sm transition-colors",
                  active
                    ? "bg-sidebar-primary/15 text-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )}
              >
                <span className="line-clamp-2 font-medium leading-snug">{s.title}</span>
                <span className="mt-0.5 text-[10px] opacity-70">{formatWhen(s.modified)}</span>
              </button>
            );
          })}
        </div>
      </ScrollArea>
    </aside>
  );
}

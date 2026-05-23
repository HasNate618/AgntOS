import { useEffect, useMemo, useState } from "react";
import { Search, RefreshCw, CheckCircle, FileEdit, RotateCcw, Circle } from "lucide-react";
import { Card } from "./ui/card";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { useTauriInvoke } from "../hooks/useTauriInvoke";
import type { AuditEntry } from "../lib/types";

function actionColor(action: string): string {
  switch (action) {
    case "apply":
      return "var(--success)";
    case "propose":
      return "var(--warning)";
    case "rollback":
      return "var(--destructive)";
    default:
      return "var(--muted-foreground)";
  }
}

function actionIcon(action: string) {
  const cls = "size-4 shrink-0 mt-0.5";
  switch (action) {
    case "apply":
      return <CheckCircle className={cls} style={{ color: "var(--success)" }} />;
    case "propose":
      return <FileEdit className={cls} style={{ color: "var(--warning)" }} />;
    case "rollback":
      return <RotateCcw className={cls} style={{ color: "var(--destructive)" }} />;
    default:
      return <Circle className={cls} size={8} style={{ color: "var(--muted-foreground)" }} />;
  }
}

export default function ActivityPage() {
  const [search, setSearch] = useState("");
  const { data, loading, execute } = useTauriInvoke<AuditEntry[]>("list_audit_entries");

  useEffect(() => {
    execute({ limit: 50 });
  }, [execute]);

  const filtered = useMemo(() => {
    const entries = data ?? [];
    if (!search) return entries;
    const q = search.toLowerCase();
    return entries.filter(
      (e) =>
        e.description?.toLowerCase().includes(q) ||
        e.message?.toLowerCase().includes(q) ||
        e.action?.toLowerCase().includes(q),
    );
  }, [data, search]);

  const handleRevert = async (generation?: number) => {
    if (generation == null) return;
    try {
      await window.__TAURI__.core.invoke("rollback_to", { generation });
      await execute({ limit: 50 });
    } catch {}
  };

  return (
    <div className="flex flex-col h-full p-4 gap-4">
      <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground shrink-0">
        Activity Log
      </span>

      <div className="flex items-center gap-2 shrink-0">
        <div className="relative flex-1">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search activity..."
            className="pl-8"
          />
        </div>
        <Button variant="ghost" size="sm" onClick={() => execute({ limit: 50 })}>
          <RefreshCw className="size-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto space-y-2">
        {loading && filtered.length === 0 ? (
          <div className="text-center text-sm text-muted-foreground py-8">Loading...</div>
        ) : filtered.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-sm text-muted-foreground">No activity yet</p>
            <p className="text-xs text-muted-foreground/60 mt-1">
              System mutations will appear here
            </p>
          </div>
        ) : (
          filtered.map((entry) => (
            <Card
              key={entry.id}
              size="sm"
              className="!p-3 !gap-0"
              style={{ borderLeft: `3px solid ${actionColor(entry.action)}` }}
            >
              <div className="flex items-start gap-2">
                {actionIcon(entry.action)}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-sm font-medium">{entry.action}</span>
                    <span className="text-xs text-muted-foreground shrink-0 ml-auto">
                      {new Date(entry.timestamp ?? entry.time ?? Date.now()).toLocaleString()}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    {entry.description || entry.message || ""}
                  </p>
                  <div className="flex items-center gap-2 mt-1">
                    <code className="text-[10px] text-primary font-mono">{entry.id}</code>
                    {entry.generation != null && (
                      <span className="text-[10px]" style={{ color: "var(--warning)" }}>
                        gen {entry.generation}
                      </span>
                    )}
                  </div>
                </div>
                {entry.action === "apply" && (
                  <Button
                    variant="destructive"
                    size="xs"
                    className="shrink-0"
                    onClick={() => handleRevert(entry.generation)}
                  >
                    Revert
                  </Button>
                )}
              </div>
            </Card>
          ))
        )}
      </div>
    </div>
  );
}

import { useCallback, useState } from "react";
import { Thread } from "@/components/assistant-ui/thread";
import { AgntRuntimeProvider } from "@/hooks/AgntRuntimeProvider";
import { useAgentStore } from "@/hooks/TauriProvider";
import AgntLogo from "@/components/AgntLogo";
import ThreadList, { type PiSession } from "@/components/ThreadList";
import { Badge } from "@/components/ui/badge";

function ChatChrome() {
  const { state } = useAgentStore();
  const { connection } = state;

  return (
    <header className="flex items-center h-11 px-4 shrink-0 border-b border-border bg-background/80 backdrop-blur-sm gap-2">
      <AgntLogo size={22} />
      <span
        className="text-sm font-semibold tracking-tight"
        style={{ fontFamily: "var(--font-display)" }}
      >
        Agent
      </span>
      <div className="flex-1" />
      <Badge
        variant="outline"
        className={
          connection.connected
            ? "border-[color-mix(in_oklab,var(--success)_35%,transparent)] text-[var(--success)]"
            : "border-destructive/40 text-destructive"
        }
      >
        <span
          className={`mr-1.5 size-1.5 rounded-full ${
            connection.connected ? "bg-[var(--success)]" : "bg-destructive"
          }`}
        />
        {connection.connected ? "Connected" : "Disconnected"}
      </Badge>
      {connection.model && (
        <span className="ml-2 text-xs font-mono text-muted-foreground truncate max-w-[180px]">
          {connection.model}
        </span>
      )}
    </header>
  );
}

export default function ChatPage() {
  const [sessionKey, setSessionKey] = useState("initial");
  const [activePath, setActivePath] = useState<string | null>(null);
  const [listCollapsed, setListCollapsed] = useState(false);

  const onNewSession = useCallback(async () => {
    if (window.__TAURI__) {
      try {
        await window.__TAURI__.core.invoke("new_session");
      } catch {
        // ignore
      }
    }
    setActivePath(null);
    setSessionKey(`new-${Date.now()}`);
  }, []);

  const onSelectSession = useCallback(async (session: PiSession) => {
    if (window.__TAURI__) {
      try {
        await window.__TAURI__.core.invoke("switch_session", {
          session_path: session.path,
        });
      } catch {
        // ignore
      }
    }
    setActivePath(session.path);
    setSessionKey(session.path);
  }, []);

  return (
    <div className="flex h-full min-h-0">
      <ThreadList
        activePath={activePath}
        collapsed={listCollapsed}
        onCollapsedChange={setListCollapsed}
        onSelect={onSelectSession}
        onNew={onNewSession}
      />
      <AgntRuntimeProvider key={sessionKey}>
        <div className="flex min-w-0 flex-1 flex-col">
          <ChatChrome />
          <div className="min-h-0 flex-1">
            <Thread />
          </div>
        </div>
      </AgntRuntimeProvider>
    </div>
  );
}

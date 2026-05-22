import { Thread } from "@/components/assistant-ui/thread";
import { AgntRuntimeProvider } from "@/hooks/AgntRuntimeProvider";
import { useAgentStore } from "@/hooks/TauriProvider";
import { Badge } from "@/components/ui/badge";

function ChatChrome() {
  const { state } = useAgentStore();
  const { connection } = state;

  return (
    <header className="flex items-center h-11 px-4 shrink-0 border-b border-border bg-background/80 backdrop-blur-sm">
      <span
        className="text-sm font-semibold tracking-tight"
        style={{ fontFamily: "var(--font-display)" }}
      >
        AgntOS
      </span>
      <span className="text-muted-foreground text-xs mx-2">·</span>
      <span className="text-xs text-muted-foreground">Agent</span>
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
  return (
    <AgntRuntimeProvider>
      <div className="flex h-full flex-col">
        <ChatChrome />
        <div className="flex-1 min-h-0">
          <Thread />
        </div>
      </div>
    </AgntRuntimeProvider>
  );
}

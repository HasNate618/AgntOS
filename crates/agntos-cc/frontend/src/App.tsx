import { useState } from "react";
import { TauriProvider, useAgentStore } from "@/hooks/TauriProvider";
import Sidebar from "@/components/Sidebar";
import ChatPage from "@/components/ChatPage";
import StatusPage from "@/components/StatusPage";
import ProposalsPage from "@/components/ProposalsPage";
import ActivityPage from "@/components/ActivityPage";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { Page } from "@/lib/types";

function TopBar() {
  const { state } = useAgentStore();
  const { connection } = state;

  return (
    <header className="flex items-center h-12 px-5 shrink-0 border-b border-border bg-card/50 backdrop-blur-sm">
      <div className="flex items-center gap-2.5">
        <span
          className="text-base font-bold tracking-tight text-foreground"
          style={{ fontFamily: "var(--font-display)" }}
        >
          AgntOS
        </span>
        <span className="text-[11px] text-muted-foreground font-medium">Control Centre</span>
      </div>
      <div className="flex-1" />
      <div className="flex items-center gap-2.5 text-xs">
        <span
          className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border ${
            connection.connected
              ? "border-[color-mix(in_oklab,var(--success)_30%,transparent)] bg-[color-mix(in_oklab,var(--success)_10%,transparent)] text-[var(--success)]"
              : "border-[color-mix(in_oklab,var(--destructive)_30%,transparent)] bg-[color-mix(in_oklab,var(--destructive)_10%,transparent)] text-destructive"
          }`}
        >
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              connection.connected ? "bg-[var(--success)]" : "bg-destructive"
            }`}
          />
          {connection.connected ? "Connected" : "Disconnected"}
        </span>
        {connection.model && (
          <span className="font-mono text-muted-foreground">{connection.model}</span>
        )}
      </div>
    </header>
  );
}

function PageContent({ page }: { page: Page }) {
  switch (page) {
    case "chat":
      return <ChatPage />;
    case "status":
      return <StatusPage />;
    case "proposals":
      return <ProposalsPage />;
    case "activity":
      return <ActivityPage />;
  }
}

function AppShell() {
  const [activePage, setActivePage] = useState<Page>("chat");
  const isChat = activePage === "chat";

  return (
    <div className="flex w-full h-screen overflow-hidden bg-background">
      <Sidebar activePage={activePage} onNavigate={setActivePage} />
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {!isChat && <TopBar />}
        <main className="flex-1 overflow-hidden">
          <PageContent page={activePage} />
        </main>
      </div>
    </div>
  );
}

function App() {
  return (
    <TooltipProvider delayDuration={300}>
      <TauriProvider>
        <AppShell />
      </TauriProvider>
    </TooltipProvider>
  );
}

export default App;

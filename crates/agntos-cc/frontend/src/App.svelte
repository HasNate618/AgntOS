<script>
  import { onMount } from "svelte";
  import ChatPage from "./components/ChatPage.svelte";
  import StatusPage from "./components/StatusPage.svelte";
  import ProposalsPage from "./components/ProposalsPage.svelte";
  import ActivityPage from "./components/ActivityPage.svelte";
  import StatusIndicator from "./components/StatusIndicator.svelte";
  import { connection } from "./stores/index.js";

  let currentPage = $state("chat");

  const pages = [
    { id: "chat", icon: "💬", label: "Chat" },
    { id: "status", icon: "📊", label: "Status" },
    { id: "proposals", icon: "📋", label: "Proposals" },
    { id: "activity", icon: "📜", label: "Activity" },
  ];

  onMount(async () => {
    // Poll for manual bridge status (set via eval from Rust)
    for (let i = 0; i < 30; i++) {
      const s = window.__AGNTOS_BRIDGE_STATUS__;
      if (s && s.connected) {
        connection.update((c) => ({ ...c, connected: true, state: s.state || "idle" }));
        break;
      }
      await new Promise((r) => setTimeout(r, 200));
    }

    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;

    listen("agent:connected", () => {
      connection.update((c) => ({ ...c, connected: true }));
      invoke("get_connection_status").then((status) => {
        if (status.model) {
          connection.update((c) => ({ ...c, model: status.model }));
        }
      }).catch(() => {});
    }).catch((e) => console.error("listen agent:connected failed", e));

    listen("agent:disconnected", () => {
      connection.update((c) => ({ ...c, connected: false, state: "disconnected" }));
    }).catch((e) => console.error("listen agent:disconnected failed", e));

    listen("agent:start", () => {
      connection.update((c) => ({ ...c, state: "thinking" }));
    }).catch((e) => console.error("listen agent:start failed", e));

    listen("agent:end", () => {
      connection.update((c) => ({ ...c, state: "idle" }));
    }).catch((e) => console.error("listen agent:end failed", e));

    listen("agent:rpc-response", (event) => {
      const data = JSON.parse(event.payload);
      if (data.command === "set_model" && data.data?.model) {
        const model = data.data.model;
        connection.update((c) => ({ ...c, model: model.name || model.id }));
      }
    }).catch((e) => console.error("listen agent:rpc-response failed", e));
  });
</script>

<div class="app-layout">
  <nav class="sidebar">
    {#each pages as page}
      <button
        class={currentPage === page.id ? "active" : ""}
        class:active-icon={currentPage === page.id}
        onclick={() => (currentPage = page.id)}
        data-label={page.label}
      >
        {page.icon}
      </button>
    {/each}
  </nav>

  <div class="main-content">
    <StatusIndicator />
    {#if currentPage === "chat"}
      <ChatPage />
    {:else if currentPage === "status"}
      <StatusPage />
    {:else if currentPage === "proposals"}
      <ProposalsPage />
    {:else if currentPage === "activity"}
      <ActivityPage />
    {/if}
  </div>
</div>
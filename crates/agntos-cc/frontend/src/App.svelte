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

  onMount(() => {
    const { listen } = window.__TAURI__.event;

    listen("agent:connected", () => {
      connection.update((c) => ({ ...c, connected: true }));
    });

    listen("agent:disconnected", () => {
      connection.update((c) => ({ ...c, connected: false, state: "disconnected" }));
    });

    listen("agent:start", () => {
      connection.update((c) => ({ ...c, state: "thinking" }));
    });

    listen("agent:end", () => {
      connection.update((c) => ({ ...c, state: "idle" }));
    });
  });
</script>

<div class="app-layout">
  <nav class="sidebar">
    {#each pages as page}
      <button
        class={currentPage === page.id ? "active" : ""}
        onclick={() => (currentPage = page.id)}
        title={page.label}
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
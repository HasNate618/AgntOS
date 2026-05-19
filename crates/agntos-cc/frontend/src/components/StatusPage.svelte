<script>
  import { onMount } from "svelte";
  import { connection } from "../stores";

  let systemInfo = $state(null);
  let conn = $derived($connection);

  onMount(async () => {
    try {
      const { invoke } = window.__TAURI__.core;
      systemInfo = await invoke("get_system_info");
    } catch (e) {
      console.error("Failed to get system info:", e);
    }
  });
</script>

<div class="status-page">
  <div class="page-header">System Status</div>

  <div class="status-section">
    <h3>Agent Connection</h3>
    <div class="status-grid">
      <div class="status-item">
        <span class="label">Status</span>
        <span class="value" class:connected={conn.connected} class:disconnected={!conn.connected}>
          {conn.connected ? "Connected" : "Disconnected"}
        </span>
      </div>
      <div class="status-item">
        <span class="label">State</span>
        <span class="value">{conn.state}</span>
      </div>
      <div class="status-item">
        <span class="label">Model</span>
        <span class="value">{conn.model || "Unknown"}</span>
      </div>
    </div>
  </div>

  {#if systemInfo}
    <div class="status-section">
      <h3>System Information</h3>
      <div class="system-info">
        <pre>{JSON.stringify(systemInfo, null, 2)}</pre>
      </div>
    </div>
  {:else}
    <div class="status-section">
      <h3>System Information</h3>
      <p class="muted">Run `agntctl inspect system` to populate</p>
    </div>
  {/if}
</div>

<style>
  .status-page {
    padding: 16px;
    overflow-y: auto;
  }
  .status-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 12px;
  }
  .status-section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
  }
  .status-grid {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 8px;
  }
  .status-item {
    display: contents;
  }
  .label {
    color: var(--text-secondary);
    font-size: 13px;
  }
  .value {
    font-size: 13px;
    font-weight: 500;
  }
  .value.connected {
    color: var(--success);
  }
  .value.disconnected {
    color: var(--error);
  }
  .system-info pre {
    background: var(--bg-primary);
    padding: 12px;
    border-radius: 6px;
    font-size: 12px;
    overflow-x: auto;
    color: var(--text-secondary);
  }
  .muted {
    color: var(--text-secondary);
    font-size: 13px;
  }
</style>
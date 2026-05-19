<script>
  import { connection, messages } from "../stores/index.js";

  let conn = $derived($connection);

  function stateLabel(state) {
    switch (state) {
      case "idle": return "Ready";
      case "thinking": return "Thinking...";
      case "streaming": return "Streaming...";
      case "disconnected": return "Disconnected";
      default: return state;
    }
  }

  function stateColor(state) {
    if (!conn.connected) return "var(--error)";
    switch (state) {
      case "idle": return "var(--success)";
      case "thinking":
      case "streaming": return "var(--warning)";
      default: return "var(--text-secondary)";
    }
  }

  async function newSession() {
    try {
      const { invoke } = window.__TAURI__.core;
      await invoke("new_session");
      messages.set([]);
    } catch (e) {
      console.error("Failed to create new session:", e);
    }
  }
</script>

<div class="status-bar">
  <div class="status-left">
    <span class="status-dot" style="background: {stateColor(conn.state)}"></span>
    <span class="status-text">{stateLabel(conn.state)}</span>
    {#if conn.model}
      <span class="status-separator">·</span>
      <span class="status-model">{conn.model}</span>
    {/if}
  </div>
  <div class="status-right">
    <button class="new-session-btn" onclick={newSession}>+ New Session</button>
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    height: 40px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-primary);
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-text {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .status-separator {
    color: var(--border-color);
    font-size: 14px;
  }

  .status-model {
    font-size: 12px;
    color: var(--accent);
    font-weight: 500;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .new-session-btn {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .new-session-btn:hover {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
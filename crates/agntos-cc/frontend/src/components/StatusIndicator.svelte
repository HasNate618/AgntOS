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
  <span class="status-dot" style="background: {stateColor(conn.state)}"></span>
  <span class="status-text">{stateLabel(conn.state)}</span>
  {#if conn.model}
    <span class="status-separator">·</span>
    <span class="status-model">{conn.model}</span>
  {/if}
  <div class="status-spacer"></div>
  <button class="new-session-btn" onclick={newSession} title="New Session">+ New Session</button>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 16px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    font-size: 12px;
    color: var(--text-secondary);
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .status-separator {
    color: var(--border-color);
  }
  .status-model {
    color: var(--accent);
  }
  .status-spacer {
    flex: 1;
  }
  .new-session-btn {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s ease;
  }
  .new-session-btn:hover {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
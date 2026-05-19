<script>
  import { onMount } from "svelte";

  let entries = $state([]);

  onMount(async () => {
    try {
      const { Command } = window.__TAURI__.shell;
      const output = await Command.create("agntctl", ["audit", "list", "--limit", "50"]).execute();
      entries = output.stdout
        .split("\n")
        .filter((line) => line.trim())
        .map((line) => {
          try {
            return JSON.parse(line);
          } catch {
            return { raw: line };
          }
        });
    } catch (e) {
      console.error("Failed to load audit log:", e);
    }
  });
</script>

<div class="activity-page">
  <div class="page-header">Activity Log</div>

  {#if entries.length === 0}
    <div class="empty-state">
      <p>No activity yet</p>
      <p class="muted">System changes will appear here</p>
    </div>
  {:else}
    <div class="activity-list">
      {#each entries as entry}
        <div class="activity-entry">
          <span class="entry-time">
            {entry.timestamp || entry.time || "—"}
          </span>
          <span class="entry-action">{entry.action || entry.type || "action"}</span>
          <span class="entry-detail">
            {entry.description || entry.message || entry.raw || ""}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .activity-page {
    padding: 16px;
    overflow-y: auto;
  }
  .empty-state {
    text-align: center;
    padding: 48px;
    color: var(--text-secondary);
  }
  .empty-state p:first-child {
    font-size: 16px;
    margin-bottom: 4px;
    color: var(--text-primary);
  }
  .muted {
    font-size: 13px;
  }
  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .activity-entry {
    display: flex;
    gap: 12px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-radius: 6px;
    font-size: 13px;
    align-items: baseline;
  }
  .entry-time {
    color: var(--text-secondary);
    font-family: monospace;
    font-size: 12px;
    white-space: nowrap;
  }
  .entry-action {
    color: var(--accent);
    font-weight: 500;
    min-width: 80px;
  }
  .entry-detail {
    color: var(--text-primary);
    flex: 1;
  }
</style>
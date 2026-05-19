<script>
  import { proposals } from "../stores";

  let items = $derived($proposals);
</script>

<div class="proposals-page">
  <div class="page-header">Pending Proposals</div>

  {#if items.length === 0}
    <div class="empty-state">
      <p>No pending proposals</p>
      <p class="muted">Ask the agent to make changes to see proposals here</p>
    </div>
  {:else}
    {#each items as proposal (proposal.id)}
      <div class="proposal-card">
        <div class="proposal-header">
          <span class="proposal-id">{proposal.id}</span>
          <span class="proposal-status {proposal.status}">{proposal.status}</span>
        </div>
        <div class="proposal-description">{proposal.description}</div>
        {#if proposal.status === "pending"}
          <div class="proposal-actions">
            <button class="approve">Approve</button>
            <button class="reject">Dismiss</button>
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .proposals-page {
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
  .proposal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 8px;
  }
  .proposal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .proposal-id {
    font-family: monospace;
    font-size: 13px;
    color: var(--accent);
  }
  .proposal-status {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 4px;
  }
  .proposal-status.pending {
    background: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }
  .proposal-status.applied {
    background: rgba(34, 197, 94, 0.15);
    color: var(--success);
  }
  .proposal-description {
    font-size: 14px;
    color: var(--text-secondary);
    margin-bottom: 12px;
  }
  .proposal-actions {
    display: flex;
    gap: 8px;
  }
  .proposal-actions button {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }
  .approve {
    background: var(--success);
    color: white;
  }
  .reject {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }
</style>
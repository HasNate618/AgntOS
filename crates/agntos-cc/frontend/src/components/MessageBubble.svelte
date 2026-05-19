<script>
  import { renderMarkdown } from "../lib/markdown";
  import { getToolMeta } from "../lib/types";

  let { msg, currentPartial = "", approveProposal, rejectProposal } = $props();

  let isUser = $derived(msg.role === "user");
  let isAssistant = $derived(msg.role === "assistant");
  let isTool = $derived(msg.role === "tool");
  let isError = $derived(msg.role === "error");
  let isApproval = $derived(msg.role === "approval");

  let renderedContent = $derived.by(() => {
    if (msg.content) return renderMarkdown(msg.content);
    if (currentPartial && isAssistant) return renderMarkdown(currentPartial);
    return "";
  });

  let timeStr = $derived(msg.timestamp ? new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : "");
</script>

<div class="message-bubble" class:user={isUser} class:assistant={isAssistant} class:error={isError}>
  {#if isUser}
    <div class="user-bubble-wrapper">
      <div class="bubble user-bubble">
        <div class="content">{msg.content}</div>
      </div>
      {#if timeStr}
        <div class="timestamp">{timeStr}</div>
      {/if}
    </div>
  {:else if isAssistant}
    <div class="assistant-bubble-wrapper">
      <div class="assistant-label">Agent</div>
      <div class="bubble assistant-bubble">
        {@html renderedContent}
      </div>
      {#if timeStr}
        <div class="timestamp">{timeStr}</div>
      {/if}
    </div>
  {:else if isTool}
    {@const meta = getToolMeta(msg.name)}
    <div class="tool-card">
      <div class="tool-header" style="border-left-color: {meta.color}">
        <span class="tool-icon">{meta.icon}</span>
        <span class="tool-name">{msg.name}</span>
        {#if msg.state === "running"}
          <span class="spinner"></span>
        {:else}
          <span class="done">✓</span>
        {/if}
      </div>
      {#if msg.state === "done" && msg.result}
        <details class="tool-result">
          <summary>Result</summary>
          <pre>{typeof msg.result === "string" ? msg.result : JSON.stringify(msg.result, null, 2)}</pre>
        </details>
      {/if}
    </div>
  {:else if isApproval}
    <div class="approval-card">
      <div class="approval-header">
        <span class="approval-icon">⚠️</span>
        <span class="approval-title">{msg.title || "Approval Required"}</span>
      </div>
      <div class="approval-message">{msg.message}</div>
      <div class="approval-actions">
        <button class="approve" onclick={() => approveProposal?.(msg.id)}>Approve</button>
        <button class="reject" onclick={() => rejectProposal?.(msg.id)}>Dismiss</button>
      </div>
    </div>
  {:else if isError}
    <div class="error-bubble-wrapper">
      <div class="error-label">Error</div>
      <div class="error-bubble">{msg.content}</div>
    </div>
  {/if}
</div>

<style>
  .message-bubble {
    max-width: 85%;
  }

  .message-bubble.user {
    align-self: flex-end;
  }

  .message-bubble.assistant,
  .message-bubble.error {
    align-self: flex-start;
  }

  .bubble {
    padding: 12px 18px;
    border-radius: 14px;
    line-height: 1.6;
    font-size: 15px;
  }

  .user-bubble-wrapper,
  .assistant-bubble-wrapper {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .assistant-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    margin-left: 4px;
  }

  .error-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--error);
    margin-left: 4px;
  }

  .user-bubble {
    background: var(--accent);
    color: white;
  }

  .assistant-bubble {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .assistant-bubble :global(pre) {
    background: var(--bg-primary);
    padding: 14px;
    border-radius: 8px;
    overflow-x: auto;
    margin: 8px 0;
  }

  .assistant-bubble :global(code) {
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 13px;
  }

  .assistant-bubble :global(p code) {
    background: var(--bg-primary);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .assistant-bubble :global(p) {
    margin: 6px 0;
  }

  .assistant-bubble :global(p:first-child) {
    margin-top: 0;
  }

  .assistant-bubble :global(p:last-child) {
    margin-bottom: 0;
  }

  .timestamp {
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.5;
    margin: 0 6px;
  }

  .tool-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
    width: 100%;
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    font-size: 13px;
    border-left: 3px solid var(--accent);
  }

  .tool-icon {
    font-size: 16px;
  }

  .tool-name {
    font-weight: 600;
    flex: 1;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .done {
    color: var(--success);
    font-weight: bold;
    font-size: 14px;
  }

  .tool-result {
    border-top: 1px solid var(--border-color);
  }

  .tool-result summary {
    padding: 8px 14px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    font-weight: 500;
  }

  .tool-result summary:hover {
    background: var(--bg-tertiary);
  }

  .tool-result pre {
    margin: 0;
    padding: 10px 14px;
    font-size: 12px;
    overflow-x: auto;
    color: var(--text-secondary);
    background: var(--bg-primary);
  }

  .approval-card {
    background: var(--bg-secondary);
    border: 1px solid var(--warning);
    border-radius: 10px;
    padding: 16px;
    width: 100%;
  }

  .approval-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }

  .approval-icon {
    font-size: 16px;
  }

  .approval-title {
    font-weight: 600;
    font-size: 14px;
  }

  .approval-message {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 14px;
    line-height: 1.5;
  }

  .approval-actions {
    display: flex;
    gap: 8px;
  }

  .approval-actions button {
    padding: 8px 20px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.15s ease;
  }

  .approve {
    background: var(--success);
    color: white;
  }

  .approve:hover {
    filter: brightness(1.1);
  }

  .reject {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
  }

  .reject:hover {
    background: var(--error);
    color: white;
    border-color: var(--error);
  }

  .error-bubble-wrapper {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .error-bubble {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 10px;
    padding: 12px 16px;
    color: var(--error);
    font-size: 14px;
    line-height: 1.5;
  }
</style>
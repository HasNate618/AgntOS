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
</script>

<div class="message-bubble" class:user={isUser} class:assistant={isAssistant} class:error={isError}>
  {#if isUser}
    <div class="bubble user-bubble">
      <div class="content">{msg.content}</div>
    </div>
  {:else if isAssistant}
    <div class="bubble assistant-bubble">
      {@html renderedContent}
    </div>
  {:else if isTool}
    {@const meta = getToolMeta(msg.name)}
    <div class="tool-card">
      <div class="tool-header" style="border-left: 3px solid {meta.color}">
        <span class="tool-icon">{meta.icon}</span>
        <span class="tool-name">{msg.name}</span>
        {#if msg.state === "running"}
          <span class="spinner">⏳</span>
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
      <div class="approval-title">{msg.title || "Approval Required"}</div>
      <div class="approval-message">{msg.message}</div>
      <div class="approval-actions">
        <button class="approve" onclick={() => approveProposal?.(msg.id)}>Approve</button>
        <button class="reject" onclick={() => rejectProposal?.(msg.id)}>Dismiss</button>
      </div>
    </div>
  {:else if isError}
    <div class="error-bubble">{msg.content}</div>
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
    padding: 10px 14px;
    border-radius: 12px;
    line-height: 1.5;
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
    padding: 12px;
    border-radius: 8px;
    overflow-x: auto;
  }
  .assistant-bubble :global(code) {
    font-family: "JetBrains Mono", monospace;
    font-size: 13px;
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
    padding: 8px 12px;
    font-size: 13px;
  }
  .tool-name {
    font-weight: 500;
    flex: 1;
  }
  .spinner {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .tool-result {
    border-top: 1px solid var(--border-color);
    padding: 8px 12px;
  }
  .tool-result pre {
    margin: 0;
    font-size: 12px;
    overflow-x: auto;
    color: var(--text-secondary);
  }
  .approval-card {
    background: var(--bg-secondary);
    border: 2px solid var(--warning);
    border-radius: 8px;
    padding: 12px;
    width: 100%;
  }
  .approval-title {
    font-weight: 600;
    margin-bottom: 8px;
  }
  .approval-message {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 12px;
  }
  .approval-actions {
    display: flex;
    gap: 8px;
  }
  .approval-actions button {
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
  .error-bubble {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid var(--error);
    border-radius: 8px;
    padding: 10px 14px;
    color: var(--error);
  }
</style>
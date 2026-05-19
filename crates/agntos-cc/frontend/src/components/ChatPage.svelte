<script>
  import { onMount } from "svelte";
  import { messages } from "../stores/index.js";
  import MessageBubble from "./MessageBubble.svelte";

  let inputText = $state("");
  let messageContainer;
  let currentPartial = $state("");

  onMount(() => {
    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;

    listen("agent:message-update", (event) => {
      const data = JSON.parse(event.payload);
      const msgEvent = data.assistantMessageEvent;

      if (msgEvent?.type === "text_delta" && msgEvent.delta) {
        currentPartial += msgEvent.delta;
      } else if (msgEvent?.type === "text_end") {
        messages.update((msgs) => [
          ...msgs,
          { role: "assistant", content: currentPartial, timestamp: Date.now() },
        ]);
        currentPartial = "";
      }
    }).catch((e) => console.error("listen agent:message-update failed", e));

    listen("agent:tool-start", (event) => {
      const data = JSON.parse(event.payload);
      messages.update((msgs) => [
        ...msgs,
        {
          role: "tool",
          name: data.toolName,
          args: data.args,
          state: "running",
          timestamp: Date.now(),
        },
      ]);
    }).catch((e) => console.error("listen agent:tool-start failed", e));

    listen("agent:tool-end", (event) => {
      const data = JSON.parse(event.payload);
      messages.update((msgs) => {
        const last = msgs[msgs.length - 1];
        if (last?.role === "tool" && last.name === data.toolName) {
          return [
            ...msgs.slice(0, -1),
            { ...last, result: data.result, state: "done" },
          ];
        }
        return [
          ...msgs,
          { role: "tool", name: data.toolName, result: data.result, state: "done", timestamp: Date.now() },
        ];
      });
    }).catch((e) => console.error("listen agent:tool-end failed", e));

    listen("agent:approval-request", (event) => {
      const data = JSON.parse(event.payload);
      messages.update((msgs) => [
        ...msgs,
        {
          role: "approval",
          id: data.id,
          title: data.title,
          message: data.message,
          resolved: false,
          timestamp: Date.now(),
        },
      ]);
    }).catch((e) => console.error("listen agent:approval-request failed", e));

    listen("agent:error", (event) => {
      const data = JSON.parse(event.payload);
      messages.update((msgs) => [
        ...msgs,
        { role: "error", content: data.message || "An error occurred", timestamp: Date.now() },
      ]);
    }).catch((e) => console.error("listen agent:error failed", e));
  });

  async function sendMessage() {
    if (!inputText.trim()) return;

    const { invoke } = window.__TAURI__.core;

    messages.update((msgs) => [
      ...msgs,
      { role: "user", content: inputText, timestamp: Date.now() },
    ]);

    const msg = inputText;
    inputText = "";

    try {
      await invoke("send_prompt", { message: msg });
    } catch (e) {
      messages.update((msgs) => [
        ...msgs,
        { role: "error", content: e.toString(), timestamp: Date.now() },
      ]);
    }
  }

  async function approveProposal(id) {
    const { invoke } = window.__TAURI__.core;
    try {
      await invoke("send_extension_ui_response", { id, confirmed: true });
      messages.update((msgs) =>
        msgs.map((m) =>
          m.role === "approval" && m.id === id ? { ...m, resolved: true } : m
        )
      );
    } catch (e) {
      console.error("Approval failed:", e);
    }
  }

  async function rejectProposal(id) {
    const { invoke } = window.__TAURI__.core;
    try {
      await invoke("send_extension_ui_response", { id, confirmed: false });
      messages.update((msgs) =>
        msgs.map((m) =>
          m.role === "approval" && m.id === id
            ? { ...m, resolved: true, rejected: true }
            : m
        )
      );
    } catch (e) {
      console.error("Rejection failed:", e);
    }
  }

  $effect(() => {
    if (messageContainer) {
      messageContainer.scrollTop = messageContainer.scrollHeight;
    }
  });
</script>

<div class="chat-page">
  <div class="messages" bind:this={messageContainer}>
    {#each $messages as msg (msg.timestamp)}
      <MessageBubble
        {msg}
        {currentPartial}
        {approveProposal}
        {rejectProposal}
      />
    {/each}
  </div>

  <div class="input-area">
    <textarea
      bind:value={inputText}
      placeholder="Ask the agent..."
      onkeydown={(e) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          sendMessage();
        }
      }}
    ></textarea>
    <button onclick={sendMessage} disabled={!inputText.trim()}>Send</button>
  </div>
</div>

<style>
  .chat-page {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .input-area {
    display: flex;
    padding: 16px 20px 20px;
    border-top: 1px solid var(--border-color);
    background: var(--bg-primary);
    gap: 12px;
    align-items: flex-end;
  }

  .input-area textarea {
    flex: 1;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    color: var(--text-primary);
    padding: 14px 16px;
    font-size: 16px;
    resize: none;
    min-height: 52px;
    max-height: 150px;
    font-family: inherit;
    line-height: 1.5;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .input-area textarea:focus {
    border-color: var(--accent);
  }

  .input-area textarea::placeholder {
    color: var(--text-secondary);
    opacity: 0.7;
  }

  .input-area button {
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 10px;
    padding: 14px 24px;
    cursor: pointer;
    font-size: 15px;
    font-weight: 600;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .input-area button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .input-area button:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }
</style>
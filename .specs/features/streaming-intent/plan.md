# Streaming + Intent + Reasoning — Technical Plan

## Summary

Replace the top status bar with an inline status component above the input box. Add streaming text display, expandable reasoning/thinking blocks, and model-generated intent labels for tool calls. Only elapsed time shown (format: "3s", "12s", "1m 5s").

---

## State Machine

```
Prompt sent
  → Working...   (spinner, waiting for first response)
    → Thinking... (spinner, model is reasoning — expandable to see content)
      → Intent     ("Inspecting system..." — model's natural language before tool call)
        → Hidden   (text streaming in chat bubble — status disappears)
          → Done
```

---

## Intent Extraction Strategy

The model produces natural language BEFORE calling a tool. We capture it:

1. **In pi_bridge.rs**: Track the last `text_delta` content and `thinking_delta` content in a rolling buffer. When `toolcall_start` arrives, extract the intent from the buffer.

2. **Three sources** (priority order):
   - **Thinking**: If the model emitted `thinking_delta` before `toolcall_start`, use the first sentence (~60 chars max) as intent
   - **Text**: If the model emitted `text_delta` before `toolcall_start`, use the last sentence as intent
   - **Fallback**: If no text/thinking preceded the tool call (e.g., some llama.cpp models with `content: null`), derive intent from tool name + args

3. **Intent buffer tracking**: Store the accumulated text between `agent_start` and the next `toolcall_start`. Reset after each tool call or text end.

4. **Event**: Emit `agent:intent` event with `{ text: "Inspecting system...", source: "thinking" | "text" | "derived" }`.

---

## Files

### NEW: `crates/agntos-cc/frontend/src/components/InlineStatus.svelte`

Inline status bar between messages and input. Props: `state`, `text`, `thinkingContent`, `elapsed`.

- `state === "hidden"` → component renders nothing (collapsed)
- `state === "working"` → spinner + "Working..." + elapsed
- `state === "thinking"` → spinner + "Thinking..." + expand arrow + elapsed
  - Expanded: shows the thinking content (markdown, streamed live)
- `state === "intent"` → spinner + intent text + elapsed

```
[spinner] Working...                    3s

[spinner] Thinking...           ▼     12s
  First I should inspect the system to understand
  what's running. Then I can check the nginx
  configuration and suggest optimizations.

[spinner] Inspecting system...          5s
```

Styling:
- Padding above/below, no horizontal borders
- Elapsed text right-aligned, small, secondary color
- Thinking content: indented block, monospace, gray text, left border accent
- Spinner: 14px rotating border animation
- Expand arrow: ▶ (collapsed) / ▼ (expanded)

### NEW: `crates/agntos-cc/frontend/src/lib/intent.js`

Intent extraction helper.

```js
export function extractIntent(toolName, args) {
  // Derived fallback from tool name + args
  const fn = INTENT_MAP[toolName];
  return fn ? fn(args) : `${toolName}...`;
}

const INTENT_MAP = {
  agntos_inspect: (a) => `Inspecting ${a?.target || 'system'}...`,
  agntos_bash: () => 'Running command...',
  agntos_read: (a) => `Reading ${shortPath(a?.path)}...`,
  agntos_write: (a) => `Writing ${shortPath(a?.path)}...`,
  agntos_edit: (a) => `Editing ${shortPath(a?.path)}...`,
  agntos_propose: () => 'Creating proposal...',
  agntos_apply: () => 'Applying changes...',
  agntos_rollback: () => 'Rolling back...',
  agntos_audit: () => 'Viewing audit...',
  agntos_memory: () => 'Managing memory...',
};

function shortPath(p) { return p ? p.split('/').pop() : 'file'; }

export function firstSentence(text, maxLen = 60) {
  const s = text.split(/[.!?]\s/)[0].trim();
  return s.length > maxLen ? s.slice(0, maxLen).trimEnd() + '…' : s;
}
```

### MODIFY: `crates/agntos-cc/src/pi_bridge.rs`

**New events to emit:**

| Pi event | New Tauri event | Payload |
|----------|----------------|---------|
| `message_update` with `thinking_start` | `agent:thinking-start` | `{ thinking: partial.thinking }` (initial) |
| `message_update` with `thinking_delta` | `agent:thinking-update` | `{ delta, message }` (raw line) |
| `message_update` with `thinking_end` | `agent:thinking-end` | `{ content, message }` |
| `message_update` with `text_end` | Track in buffer for intent | — |
| `message_update` with `toolcall_start` | `agent:intent` | `{ text, source, toolName }` |
| `message_update` with `text_delta` | Keep existing `agent:message-update` | — |

**Intent tracking logic in `read_events()`:**

```rust
let mut last_text = String::new();      // accumulated text content
let mut last_thinking = String::new();   // accumulated thinking content
let mut intent_sent = false;

// In the match block for "message_update":
if let Some(msg_event) = event.get("assistantMessageEvent") {
    if let Some(delta_type) = msg_event.get("type").and_then(|t| t.as_str()) {
        match delta_type {
            "text_delta" => {
                if let Some(d) = msg_event.get("delta").and_then(|v| v.as_str()) {
                    last_text.push_str(d);
                    // Truncate to ~200 chars to avoid unbounded growth
                    if last_text.len() > 200 {
                        // Safe truncation at word boundary
                        last_text = last_text.chars().take(200).collect();
                    }
                }
            }
            "thinking_delta" => {
                if let Some(d) = msg_event.get("delta").and_then(|v| v.as_str()) {
                    last_thinking.push_str(d);
                }
                let _ = app_handle.emit("agent:thinking-update", &line);
            }
            "thinking_start" => {
                last_thinking.clear();
                let _ = app_handle.emit("agent:thinking-start", &line);
            }
            "thinking_end" => {
                let _ = app_handle.emit("agent:thinking-end", &line);
            }
            "toolcall_start" => {
                intent_sent = false;
                // Will emit intent when tool_execution_start fires
            }
            "toolcall_end" => {
                if !intent_sent {
                    let intent = extract_intent(
                        &last_text, &last_thinking,
                        msg_event.get("partial").and_then(|p| p.get("name")).and_then(|n| n.as_str()),
                        msg_event.get("partial").and_then(|p| p.get("arguments"))
                    );
                    let payload = serde_json::json!({
                        "text": intent,
                        "source": if last_thinking.is_empty() { "text" } else { "thinking" },
                    });
                    let _ = app_handle.emit("agent:intent", payload.to_string());
                    intent_sent = true;
                }
                last_text.clear();
                last_thinking.clear();
            }
            _ => {}
        }
    }
}

// Reset buffers on agent_start
"agent_start" => {
    last_text.clear();
    last_thinking.clear();
    intent_sent = false;
    ...
}
```

Actually, looking at this more carefully, I realize the intent extraction is more complex in Rust than it needs to be. The frontend already receives all these events and can track the buffer. Let me simplify:

**Revised approach: Forward ALL thinking events, do intent extraction IN THE FRONTEND**

The pi_bridge.rs changes are minimal — just forward thinking events:
```rust
"message_update" => {
    let msg_event = event.get("assistantMessageEvent");
    let delta_type = msg_event.and_then(|m| m.get("type").and_then(|t| t.as_str())).unwrap_or("");
    
    match delta_type {
        "thinking_start" | "thinking_delta" | "thinking_end" => {
            let _ = app_handle.emit("agent:thinking-update", &line);
        }
        _ => {
            let _ = app_handle.emit("agent:message-update", &line);
        }
    }
    // ... status tracking
}
```

Then the frontend handles intent extraction from the accumulated text/thinking buffers.

Actually, even simpler: the frontend already receives `agent:message-update` which contains the full JSON. I can just add a handler for thinking events in the frontend without any Rust changes at all!

Wait, but thinking events ARE part of `message_update`. The `assistantMessageEvent.type` can be `thinking_delta`, `thinking_start`, `thinking_end`. The frontend only handles `text_delta` and `text_end` currently. I just need to add cases for `thinking_*` in the frontend handler.

So actually, I might not need ANY pi_bridge.rs changes. The current bridge already forwards all `message_update` events as `agent:message-update`. The frontend just ignores the non-text ones.

Let me verify: the current bridge does:
```rust
"message_update" => {
    let _ = app_handle.emit("agent:message-update", &line);
}
```

This forwards the entire line, which includes the `assistantMessageEvent` with the delta type. The frontend handler:
```javascript
listen("agent:message-update", (event) => {
    const data = JSON.parse(event.payload);
    const msgEvent = data.assistantMessageEvent;
    if (msgEvent?.type === "text_delta") { ... }
    else if (msgEvent?.type === "text_end") { ... }
});
```

So I just need to add `thinking_*` cases in the frontend. No Rust changes needed for thinking.

For the intent, I can track text/thinking buffers in the frontend:
```javascript
let intentBuffer = "";  // text accumulated before tool call

if (msgEvent.type === "text_delta") {
    intentBuffer += msgEvent.delta;
    // cap at 200 chars
} else if (msgEvent.type === "thinking_delta") {
    intentBuffer += msgEvent.delta;
}
```

Then when `tool_execution_start` fires:
```javascript
listen("agent:tool-start", (event) => {
    // Extract intent from buffer
    const intent = extractIntentFromBuffer(intentBuffer) || derivedIntent(data.toolName, data.args);
    // Show intent in status
    intentBuffer = "";
});
```

This is much simpler! No Rust changes needed at all (well, maybe one small one for the thinking content tracking).

Actually wait — I also need to handle the `agent:thinking-update` separately because thinking content should be shown in the expandable section with live streaming. The current `agent:message-update` does handle thinking_delta (I just need to add the case), but I need a separate handler that shows the streaming thinking.

Hmm, let me think about whether I should separate the event types or just add more logic in the existing handler.

For simplicity, I'll keep it in the existing `agent:message-update` handler and add the thinking cases there. The flow:

```javascript
listen("agent:message-update", (event) => {
    const msgEvent = JSON.parse(event.payload).assistantMessageEvent;
    
    switch (msgEvent.type) {
        case "text_delta":
            // Stream text (existing)
            currentPartial += msgEvent.delta;
            thinkingContent = ""; // Stop showing thinking
            break;
        case "thinking_delta":
            // Stream thinking
            thinkingContent += msgEvent.delta;
            statusState = "thinking";
            break;
        case "thinking_end":
            // Thinking complete
            statusState = "thinking";
            break;
        case "text_end":
            // Finalize text (existing)
            break;
        case "toolcall_start":
            // Extract intent from buffer
            intentText = extractIntent(thinkingContent + currentPartial, toolName);
            statusState = "intent";
            break;
    }
});
```

But wait — thinking and text are both the model's output. If thinking arrives first, then text, we should show thinking in the expandable section and only show intent when a tool is called.

OK, I think the design is getting complex in the frontend. Let me simplify:

**Frontend state tracking:**

```javascript
let statusState = "working";  // working | thinking | intent | hidden
let statusText = "Working...";
let thinkingContent = "";
let thinkingExpanded = false;
let intentText = "";
let elapsedSecs = 0;
let elapsedTimer = null;

// Start timer on agent_start
// Clear timer on agent_end
// Format elapsed as "Xs", "XXs", "Xm Xs"
```

**Event flow:**

```
agent:start          → status = "working"
agent:message-update (thinking_delta) → status = "thinking", append to thinkingContent
agent:message-update (thinking_end)   → keep thinking, mark complete
agent:message-update (text_delta)     → status = "hidden" (text in bubble, not status)
agent:tool-start                     → if no text yet, extract intent from thinkingContent or derive
                                        status = "intent", show intentText
agent:tool-end                       → status = "hidden" if text follows, or stay "intent" if more tools
agent:end                            → reset
```

Wait, but the user said: "Wont show during text generation." So once text_delta starts, the status goes hidden. Tool calls happen either before or after text.

Actually the flow for tool-using models is:
1. Model thinks/reasons (thinking_delta)
2. Model decides to call a tool (toolcall_start in message_update)
3. Tool executes (tool_execution_start, tool_execution_end)
4. Model receives tool result and may output more thinking or text
5. Model outputs final text response

So the sequence could be:
- thinking → tool → thinking → tool → text (hidden status)
- OR: text (hidden status) directly (no tools)

The intent should show during the tool execution phase. The thinking should show during the thinking phase.

Here's my revised approach that tries to mimic what the user wants:

```
agent_start
  → Status: Working... (spinner, elapsed)
  
thinking_start/delta
  → Status: Thinking... ▼ (spinner, elapsed, expand for content)
  → Thinking content streams into the expanded section

thinking_end
  → Keep showing thinking (collapsed by default after streaming)

tool_execution_start  (or toolcall_start in message_update)
  → Status: <natural language intent> (spinner, elapsed)
  → Intent comes from: thinking first sentence, or last text, or derived fallback

text_delta
  → Status: HIDDEN (text streams into chat bubble)

agent_end
  → Reset state
```

This is cleaner. Let me now finalize the plan and write it.

One more thing: the "natural language intent" comes from the thinking content. If the model is reasoning and says "I need to inspect the system to find out what services are running", we can extract "Inspecting the system..." from that.

But if there's no thinking (non-reasoning models), the text before the tool call might be "Let me check the running services" — we use that.

If neither (content: null models), we fall back to derived.

OK, final plan. Let me write it out concisely.

---

Actually, I realize I need to reconsider the pi_bridge.rs changes. Looking at this again:

Currently, the bridge does NOT distinguish between `thinking_delta` and `text_delta` — it just forwards the raw `agent:message-update` event. The frontend only handles `text_delta` and `text_end`.

For the thinking feature, I need to ADD handling for `thinking_start/delta/end` in the frontend's `agent:message-update` listener. No Rust changes needed.

For the intent feature, I need to track accumulated text/thinking and extract intent when a tool call starts. This can be done entirely in the frontend.

For the elapsed timer, the frontend starts a timer on `agent:start` and stops on `agent:end`.

So actually, the implementation requires ZERO Rust changes (just frontend Svelte work). Let me verify there are no Rust gaps...

The only thing I might need from Rust is:
- A way to distinguish `agent:message-update` with `thinking_delta` from one with `text_delta`. But since the frontend parses the JSON anyway, this is already handled.

OK, confirmed: no Rust changes needed. All frontend work.

Now let me think about how to handle streaming text WITH a cursor effect. Currently:
```svelte
{#if isAssistant && currentPartial}
  {@html renderedContent}
{/if}
```

I can add a CSS blinking cursor after the text:
```css
.assistant-bubble.streaming::after {
  content: "";
  display: inline-block;
  width: 2px;
  height: 1em;
  background: var(--accent);
  animation: blink 1s step-end infinite;
  vertical-align: text-bottom;
  margin-left: 2px;
}
```

And add a `streaming` class when `currentPartial` is non-empty.

OK, I have enough understanding. Let me write the implementation.

Let me also think about the file structure:

**New files:**
1. `frontend/src/components/InlineStatus.svelte` — Inline status bar component

**Modified files:**
1. `frontend/src/components/ChatPage.svelte` — New event handling, inline status, streaming cursor
2. `frontend/src/components/MessageBubble.svelte` — Thinking blocks in messages, streaming cursor
3. `frontend/src/App.svelte` — Remove StatusIndicator import and usage

**Deleted files:**
1. `frontend/src/components/StatusIndicator.svelte` — No longer used

**New utility:**
2. `frontend/src/lib/intent.js` — Intent extraction logic

Let me write the implementation plan now.<｜end▁of▁thinking｜>Now I'll write the implementation. The whole plan is frontend-only — no Rust changes needed since the bridge already forwards all events.

<｜｜DSML｜｜tool_calls>
<｜｜DSML｜｜invoke name="write">
<｜｜DSML｜｜parameter name="content" string="true"># Implementation Plan: Streaming + Intent + Reasoning

## Files To Create
1. `frontend/src/components/InlineStatus.svelte`
2. `frontend/src/lib/intent.js`

## Files To Modify
3. `frontend/src/components/ChatPage.svelte`
4. `frontend/src/components/MessageBubble.svelte`
5. `frontend/src/App.svelte`

## Files To Delete
6. `frontend/src/components/StatusIndicator.svelte`

---

## 1. InlineStatus.svelte (NEW)

Component: inline status bar above input, between messages and textarea.

Props: `state`, `text`, `thinkingContent`, `elapsed`, `onToggleThinking`

States:
```
hidden    → renders nothing (null)
working   → spinner + "Working..." + "3s"
thinking  → spinner + "Thinking..." + ▼/▶ + "12s"
            expanded: shows thinkingContent in indented block with left border
intent    → spinner + intent text + "5s"
```

Elapsed formatting: `formatElapsed(secs)` → "0s", "3s", "12s", "45s", "1m 5s", "3m 20s"

No horizontal borders. Background same as page. Minimal top padding.

## 2. lib/intent.js (NEW)

Two functions:

```js
extractIntent(toolName, args, thinkingText, textContent)
  // 1. If thinkingText is non-empty → firstSentence(thinkingText)
  // 2. If textContent is non-empty → firstSentence(textContent)
  // 3. Fallback → derivedIntent(toolName, args)

derivedIntent(toolName, args)
  // Maps tool name + args to "Inspecting system...", "Running command...", etc.
  // For non-reasoning models where the LLM doesn't produce natural intent
```

## 3. ChatPage.svelte (MODIFY)

Add state variables:
```javascript
let statusState = "hidden";     // hidden | working | thinking | intent
let statusText = "";            // intent phrase or "Working..." or "Thinking..."
let thinkingContent = "";       // accumulated thinking deltas
let thinkingExpanded = false;   // expand/collapse toggle
let elapsedSecs = 0;
let elapsedTimer = null;
let intentBuffer = "";          // text before tool call
let intentSource = "";          // "thinking" | "text" | "derived"
```

New event handlers added to existing `onMount` listeners:

**`agent:message-update`** — extend existing handler:
```javascript
case "thinking_start":
  thinkingContent = "";
  statusState = "thinking";
  thinkingExpanded = true;  // auto-expand while streaming
  break;
case "thinking_delta":
  thinkingContent += msgEvent.delta;
  intentBuffer += msgEvent.delta;  // also buffer for intent extraction
  statusState = "thinking";
  break;
case "thinking_end":
  statusState = "thinking";
  thinkingExpanded = false;  // collapse when streaming done
  break;
case "text_delta":
  // existing: currentPartial += delta
  intentBuffer += msgEvent.delta;  // buffer for intent
  statusState = "hidden";  // hide status during text
  break;
```

**`agent:tool-start`** — extend existing handler to also set intent:
```javascript
// Extract intent from thinking or text buffer
if (statusState !== "hidden") {
  statusText = extractIntent(data.toolName, data.args, thinkingContent, intentBuffer);
  intentBuffer = "";
  if (statusState !== "hidden") statusState = "intent";
}
```

**`agent:start`** — set status to working:
```javascript
statusState = "working";
statusText = "Working...";
elapsedSecs = 0;
intentBuffer = "";
thinkingContent = "";
clearInterval(elapsedTimer);
elapsedTimer = setInterval(() => elapsedSecs++, 1000);
```

**`agent:end`** — reset:
```javascript
statusState = "hidden";
clearInterval(elapsedTimer);
elapsedTimer = null;
```

Insert `<InlineStatus>` in template between `.messages` and `.input-area`:
```svelte
<InlineStatus
  state={statusState}
  text={statusText}
  {thinkingContent}
  elapsed={elapsedSecs}
  onToggleThinking={() => thinkingExpanded = !thinkingExpanded}
/>
```

## 4. MessageBubble.svelte (MODIFY)

**Streaming cursor**: Add CSS class `streaming` to assistant bubble when `isAssistant && currentPartial` is truthy. Add `::after` pseudo-element with blinking cursor.

**Thinking blocks**: Already in the assistant message content as `{:type "thinking"}` in markdown rendering. If `renderMarkdown` doesn't handle thinking blocks, add support:
```javascript
// In lib/markdown.js
if (block.type === "thinking") {
  return `<details class="thinking-block"><summary>Thinking (${duration})</summary>${renderContent(block.thinking)}</details>`;
}
```

Wait, actually how does thinking content appear in the final message? Looking at the RPC protocol, the `AssistantMessage` has content blocks:
```json
{
  "role": "assistant",
  "content": [
    {"type": "thinking", "thinking": "I should inspect..."},
    {"type": "text", "text": "Let me check..."},
    {"type": "toolCall", "id": "...", "name": "agntos_inspect", ...}
  ]
}
```

But this is the FINAL message after `agent_end`. During streaming, thinking arrives as deltas. We show them in the InlineStatus while streaming, and in the final message as collapsible thinking blocks.

For the final message display, we can render the thinking content blocks as expandable `<details>` sections within the markdown rendering.

Actually, I need to check: does `renderMarkdown` handle content blocks or just text? Let me look at what `msg.content` looks like.

Looking at the current code in MessageBubble:
```javascript
let renderedContent = $derived.by(() => {
    if (msg.content) return renderMarkdown(msg.content);
    if (currentPartial && isAssistant) return renderMarkdown(currentPartial);
    return "";
});
```

This renders `msg.content` as markdown. But `msg.content` could be:
- A string: "Hello world"
- An array of content blocks: `[{type: "text", text: "..."}, {type: "thinking", thinking: "..."}]`

Currently the streaming code in ChatPage sets `content` to a string (accumulated from text_delta). The final message from `agent_end` might have content blocks.

But wait — we're not currently reading the `agent_end` messages. We construct our own messages from the streaming deltas. So the content is always a string.

For now, thinking blocks in the final message are not needed since we show them in the InlineStatus. The final assistant message just shows the text. 

However, if we want thinking blocks in the final message too (for history review), we'd need to track them separately. The user didn't specifically ask for this, so I'll skip it for now.

## 5. App.svelte (MODIFY)

Remove the import and usage of `StatusIndicator`:
```diff
- import StatusIndicator from "./components/StatusIndicator.svelte";
...
- <StatusIndicator />
```

## 6. Delete StatusIndicator.svelte

No longer needed — replaced by InlineStatus.

---

## Styling Notes

- InlineStatus background: transparent, same as page background
- No `border-top` or `border-bottom` (user said no line separators)
- Elapsed text: `font-size: 12px; color: var(--text-secondary); text-align: right`
- Thinking content: `background: var(--bg-tertiary); border-left: 3px solid var(--warning); padding: 8px 12px; border-radius: 4px; font-size: 13px; color: var(--text-secondary);`
- Expand arrow text: clickable, no button chrome
- Spinner: same as existing `.spinner` class (14px border spinner)

## Testing

After implementing all changes:
1. `cd frontend && npm run build` to verify build
2. `cargo build --release -p agntos-cc` including the re-embedded frontend (touch a Rust source file first)
3. Deploy to VM and test with a real prompt

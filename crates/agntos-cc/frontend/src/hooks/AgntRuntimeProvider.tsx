import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import {
  AssistantRuntimeProvider,
  useExternalStoreRuntime,
  type AppendMessage,
  type ThreadMessageLike,
} from "@assistant-ui/react";
import { extractIntent } from "@/lib/intent";
import {
  agntId,
  appendToolCall,
  textFromAppend,
  updateToolCall,
  upsertPart,
  AGNT_SUGGESTIONS,
  type AgntMessage,
  type ContentPart,
} from "@/lib/agnt-runtime";
import { useTauriEvent } from "@/hooks/useTauriEvent";

type ApprovalHandlers = {
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
};

const ApprovalContext = createContext<ApprovalHandlers | null>(null);

export function useApprovalHandlers() {
  const ctx = useContext(ApprovalContext);
  if (!ctx) throw new Error("useApprovalHandlers requires AgntRuntimeProvider");
  return ctx;
}

function partsOf(msg: AgntMessage): ContentPart[] {
  const c = msg.content;
  if (typeof c === "string") return [{ type: "text", text: c }];
  return [...c];
}

function withParts(msg: AgntMessage, parts: ContentPart[]): AgntMessage {
  return { ...msg, content: parts };
}

function ensureAssistant(
  messages: AgntMessage[],
  assistantId: string | null,
): { messages: AgntMessage[]; id: string } {
  if (assistantId) {
    const exists = messages.some((m) => m.id === assistantId);
    if (exists) return { messages, id: assistantId };
  }
  const id = agntId();
  return {
    id,
    messages: [
      ...messages,
      {
        id,
        role: "assistant",
        content: [],
        status: { type: "running" },
        createdAt: new Date(),
      },
    ],
  };
}

function updateAssistant(
  messages: AgntMessage[],
  assistantId: string,
  updater: (parts: ContentPart[]) => ContentPart[],
): AgntMessage[] {
  return messages.map((m) =>
    m.id === assistantId ? withParts(m, updater(partsOf(m))) : m,
  );
}

function finalizeAssistant(messages: AgntMessage[], assistantId: string): AgntMessage[] {
  return messages.map((m) =>
    m.id === assistantId ? { ...m, status: { type: "complete", reason: "stop" } } : m,
  );
}

export function AgntRuntimeProvider({ children }: { children: ReactNode }) {
  const [messages, setMessages] = useState<AgntMessage[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const assistantIdRef = useRef<string | null>(null);
  const hasToolCallsRef = useRef(false);
  const intentBufferRef = useRef("");

  const patchAssistant = useCallback(
    (updater: (parts: ContentPart[]) => ContentPart[]) => {
      setMessages((prev) => {
        const { messages: next, id } = ensureAssistant(prev, assistantIdRef.current);
        assistantIdRef.current = id;
        return updateAssistant(next, id, updater);
      });
    },
    [],
  );

  const onStart = useCallback(() => {
    setIsRunning(true);
    hasToolCallsRef.current = false;
    intentBufferRef.current = "";
    assistantIdRef.current = null;
    setMessages((prev) => {
      const { messages: next, id } = ensureAssistant(prev, null);
      assistantIdRef.current = id;
      return next;
    });
  }, []);

  const onEnd = useCallback(() => {
    setIsRunning(false);
    setMessages((prev) => {
      const id = assistantIdRef.current;
      if (!id) return prev;
      const msg = prev.find((m) => m.id === id);
      const parts = msg ? partsOf(msg) : [];
      if (parts.length === 0 && !hasToolCallsRef.current) {
        return prev.filter((m) => m.id !== id);
      }
      return finalizeAssistant(prev, id);
    });
    assistantIdRef.current = null;
  }, []);

  const onMessageUpdate = useCallback(
    (payload: string) => {
      let data: Record<string, unknown>;
      try {
        data = JSON.parse(payload);
      } catch {
        return;
      }
      const msgEvent = data.assistantMessageEvent as Record<string, unknown> | undefined;
      if (!msgEvent) return;

      switch (msgEvent.type) {
        case "thinking_start":
        case "thinking_delta": {
          const delta = msgEvent.delta as string | undefined;
          if (!delta && msgEvent.type !== "thinking_start") break;
          patchAssistant((parts) =>
            upsertPart(parts, "reasoning", (existing) => ({
              type: "reasoning",
              text:
                existing?.type === "reasoning"
                  ? existing.text + (delta ?? "")
                  : delta ?? "",
            })),
          );
          if (delta) intentBufferRef.current += delta;
          break;
        }
        case "text_delta": {
          const delta = msgEvent.delta as string | undefined;
          if (!delta) break;
          if (!hasToolCallsRef.current) {
            patchAssistant((parts) =>
              upsertPart(parts, "reasoning", (existing) => ({
                type: "reasoning",
                text:
                  existing?.type === "reasoning"
                    ? existing.text + delta
                    : delta,
              })),
            );
            intentBufferRef.current += delta;
          } else {
            patchAssistant((parts) =>
              upsertPart(parts, "text", (existing) => ({
                type: "text",
                text:
                  existing?.type === "text" ? existing.text + delta : delta,
              })),
            );
          }
          break;
        }
        case "text_end":
        case "done": {
          const id = assistantIdRef.current;
          if (id) {
            setMessages((prev) => finalizeAssistant(prev, id));
          }
          break;
        }
      }
    },
    [patchAssistant],
  );

  const onToolStart = useCallback(
    (payload: string) => {
      let data: Record<string, unknown>;
      try {
        data = JSON.parse(payload);
      } catch {
        return;
      }
      hasToolCallsRef.current = true;
      const toolName = data.toolName as string;
      const args = data.args as Record<string, unknown> | undefined;

      setMessages((prev) => {
        let next = prev;
        const id = assistantIdRef.current;
        if (id) {
          const msg = prev.find((m) => m.id === id);
          if (msg) {
            const derived = extractIntent(
              toolName,
              args,
              partsOf(msg)
                .filter((p) => p.type === "reasoning")
                .map((p) => (p.type === "reasoning" ? p.text : ""))
                .join(""),
              intentBufferRef.current,
            );
            void derived;
          }
        }
        const ensured = ensureAssistant(next, assistantIdRef.current);
        assistantIdRef.current = ensured.id;
        next = updateAssistant(ensured.messages, ensured.id, (parts) =>
          appendToolCall(parts, {
            toolCallId: agntId(),
            toolName,
            args,
          }),
        );
        return next;
      });
      intentBufferRef.current = "";
    },
    [],
  );

  const onToolEnd = useCallback((payload: string) => {
    let data: Record<string, unknown>;
    try {
      data = JSON.parse(payload);
    } catch {
      return;
    }
    const toolName = data.toolName as string;
    const result = data.result as string | null;
    const id = assistantIdRef.current;
    if (!id) return;
    setMessages((prev) =>
      updateAssistant(prev, id, (parts) => updateToolCall(parts, toolName, result)),
    );
  }, []);

  const onApprovalRequest = useCallback((payload: string) => {
    let data: Record<string, unknown>;
    try {
      data = JSON.parse(payload);
    } catch {
      return;
    }
    setMessages((prev) => [
      ...prev,
      {
        id: agntId(),
        role: "assistant",
        content: [
          {
            type: "data-approval",
            data: {
              id: data.id,
              title: data.title,
              message: data.message,
              resolved: false,
            },
          },
        ],
        createdAt: new Date(),
        status: { type: "complete", reason: "stop" },
      },
    ]);
  }, []);

  const onError = useCallback((payload: string) => {
    let data: Record<string, unknown>;
    try {
      data = JSON.parse(payload);
    } catch {
      return;
    }
    setIsRunning(false);
    assistantIdRef.current = null;
    setMessages((prev) => [
      ...prev,
      {
        id: agntId(),
        role: "assistant",
        content: [
          {
            type: "text",
            text: (data.message as string) || "An error occurred",
          },
        ],
        status: { type: "incomplete", reason: "error" },
        createdAt: new Date(),
      },
    ]);
  }, []);

  useTauriEvent("agent:start", onStart);
  useTauriEvent("agent:end", onEnd);
  useTauriEvent("agent:message-update", onMessageUpdate);
  useTauriEvent("agent:tool-start", onToolStart);
  useTauriEvent("agent:tool-end", onToolEnd);
  useTauriEvent("agent:approval-request", onApprovalRequest);
  useTauriEvent("agent:error", onError);

  const handleApprove = useCallback(async (id: string) => {
    setMessages((prev) =>
      prev.map((m) => ({
        ...m,
        content: partsOf(m).map((p) =>
          p.type === "data-approval" &&
          (p as { data?: { id?: string } }).data?.id === id
            ? {
                ...p,
                data: {
                  ...(p as { data: Record<string, unknown> }).data,
                  resolved: true,
                  rejected: false,
                },
              }
            : p,
        ),
      })),
    );
    try {
      await window.__TAURI__?.core.invoke("send_extension_ui_response", {
        id,
        confirmed: true,
      });
    } catch {
      // ignore
    }
  }, []);

  const handleReject = useCallback(async (id: string) => {
    setMessages((prev) =>
      prev.map((m) => ({
        ...m,
        content: partsOf(m).map((p) =>
          p.type === "data-approval" &&
          (p as { data?: { id?: string } }).data?.id === id
            ? {
                ...p,
                data: {
                  ...(p as { data: Record<string, unknown> }).data,
                  resolved: true,
                  rejected: true,
                },
              }
            : p,
        ),
      })),
    );
    try {
      await window.__TAURI__?.core.invoke("send_extension_ui_response", {
        id,
        confirmed: false,
      });
    } catch {
      // ignore
    }
  }, []);

  const onNew = useCallback(async (message: AppendMessage) => {
    const text = textFromAppend(message);
    if (!text.trim()) return;

    const userMsg: AgntMessage = {
      id: agntId(),
      role: "user",
      content: [{ type: "text", text }],
      createdAt: new Date(),
    };
    setMessages((prev) => [...prev, userMsg]);
    setIsRunning(true);
    assistantIdRef.current = null;
    hasToolCallsRef.current = false;
    intentBufferRef.current = "";

    try {
      await window.__TAURI__?.core.invoke("send_prompt", { message: text });
    } catch (e) {
      setIsRunning(false);
      setMessages((prev) => [
        ...prev,
        {
          id: agntId(),
          role: "assistant",
          content: [{ type: "text", text: String(e) }],
          status: { type: "incomplete", reason: "error" },
          createdAt: new Date(),
        },
      ]);
    }
  }, []);

  const onCancel = useCallback(async () => {
    try {
      await window.__TAURI__?.core.invoke("send_abort");
    } catch {
      // ignore
    }
    setIsRunning(false);
    const id = assistantIdRef.current;
    if (id) {
      setMessages((prev) => finalizeAssistant(prev, id));
    }
  }, []);

  const runtime = useExternalStoreRuntime({
    messages,
    isRunning,
    convertMessage: (m: AgntMessage): ThreadMessageLike => m,
    onNew,
    onCancel,
    setMessages,
    suggestions: AGNT_SUGGESTIONS.map((s) => ({
      prompt: s.prompt,
      title: s.title,
      description: s.description,
    })),
  });

  return (
    <ApprovalContext.Provider
      value={{ onApprove: handleApprove, onReject: handleReject }}
    >
      <AssistantRuntimeProvider runtime={runtime}>{children}</AssistantRuntimeProvider>
    </ApprovalContext.Provider>
  );
}

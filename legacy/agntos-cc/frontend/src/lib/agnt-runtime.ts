import type { AppendMessage, ThreadMessageLike } from "@assistant-ui/react";

export type AgntMessage = ThreadMessageLike & {
  id: string;
  createdAt: Date;
};

export type ContentPart = NonNullable<AgntMessage["content"]> extends readonly (infer P)[]
  ? P
  : never;

export function agntId(): string {
  return crypto.randomUUID();
}

export function textFromAppend(message: AppendMessage): string {
  const { content } = message;
  if (typeof content === "string") return content;
  const text = content.find((p) => p.type === "text");
  return text && "text" in text ? String(text.text) : "";
}

export function upsertPart(
  parts: readonly ContentPart[],
  type: ContentPart["type"],
  updater: (existing: ContentPart | undefined) => ContentPart,
): ContentPart[] {
  const idx = parts.findIndex((p) => p.type === type);
  if (idx >= 0) {
    return parts.map((p, i) => (i === idx ? updater(p) : p));
  }
  return [...parts, updater(undefined)];
}

export function appendToolCall(
  parts: readonly ContentPart[],
  tool: {
    toolCallId: string;
    toolName: string;
    args?: Record<string, unknown>;
  },
): ContentPart[] {
  return [
    ...parts,
    {
      type: "tool-call" as const,
      toolCallId: tool.toolCallId,
      toolName: tool.toolName,
      args: tool.args,
    },
  ];
}

export function updateToolCall(
  parts: readonly ContentPart[],
  toolName: string,
  result: string | null,
): ContentPart[] {
  let idx = -1;
  for (let i = parts.length - 1; i >= 0; i--) {
    const p = parts[i];
    if (p.type === "tool-call" && p.toolName === toolName && p.result === undefined) {
      idx = i;
      break;
    }
  }
  if (idx < 0) return [...parts];
  return parts.map((p, i) =>
    i === idx && p.type === "tool-call"
      ? { ...p, result: result ?? "" }
      : p,
  );
}

export const AGNT_SUGGESTIONS = [
  {
    prompt: "What's using the most RAM right now?",
    title: "Inspect memory",
    description: "Check system resource usage",
  },
  {
    prompt: "List any pending Nix proposals",
    title: "Pending proposals",
    description: "Review staged system changes",
  },
  {
    prompt: "Show failed systemd units",
    title: "System health",
    description: "Find services that need attention",
  },
  {
    prompt: "Search the audit log for recent apply actions",
    title: "Audit history",
    description: "Trace what changed and why",
  },
] as const;

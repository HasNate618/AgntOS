export interface ChatEntry {
  role: "user" | "assistant" | "tool" | "approval" | "error";
  content?: string | null;
  timestamp: number;
  name?: string;
  args?: Record<string, unknown>;
  state?: "running" | "done";
  result?: string | null;
  id?: string;
  title?: string;
  message?: string;
  resolved?: boolean;
  rejected?: boolean;
}

export interface ConnectionStatus {
  connected: boolean;
  model: string | null;
  state: "disconnected" | "connecting" | "connected" | "idle" | "thinking" | "error";
  error?: string | null;
}

export interface AuditEntry {
  id: string;
  action: string;
  type?: string;
  description?: string;
  message?: string;
  timestamp?: string | number;
  time?: string | number;
  proposal_id?: string;
  generation?: number;
}

export type Page = "chat" | "status" | "proposals" | "activity" | "models";

export const TOOLS: Record<string, { color: string }> = {
  propose: { color: "#F57C48" },
  audit: { color: "#9C9CA3" },
  option: { color: "#4CAF7A" },
  memory: { color: "#8B5CF6" },
  bash: { color: "#4493F8" },
  read: { color: "#9C9CA3" },
  write: { color: "#F57C48" },
  edit: { color: "#E6A23C" },
};

export function getToolMeta(name: string): { color: string } {
  const key = name.replace("agntos_", "");
  return TOOLS[key] || { color: "#9C9CA3" };
}

export function isUser(msg: ChatEntry): boolean {
  return msg.role === "user";
}

export function isAssistant(msg: ChatEntry): boolean {
  return msg.role === "assistant";
}

export function isTool(msg: ChatEntry): boolean {
  return msg.role === "tool";
}

export function isApproval(msg: ChatEntry): boolean {
  return msg.role === "approval";
}

export function isError(msg: ChatEntry): boolean {
  return msg.role === "error";
}

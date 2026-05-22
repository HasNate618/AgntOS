export function extractIntent(
  toolName: string,
  args: Record<string, unknown> | undefined,
  thinkingText: string,
  textContent: string,
): string {
  if (thinkingText && thinkingText.trim()) {
    return firstSentence(thinkingText);
  }
  if (textContent && textContent.trim()) {
    return firstSentence(textContent);
  }
  return derivedIntent(toolName, args);
}

function derivedIntent(toolName: string, args?: Record<string, unknown>): string {
  const key = toolName.replace("agntos_", "");
  const fn = INTENT_MAP[key];
  return fn ? fn(args) : toolName;
}

const INTENT_MAP: Record<string, (args?: Record<string, unknown>) => string> = {
  inspect: (a) => `Inspecting ${a?.target ? a.target : "system"}...`,
  propose: () => "Creating proposal...",
  apply: () => "Applying changes...",
  rollback: () => "Rolling back...",
  audit: () => "Viewing audit...",
  memory: () => "Managing memory...",
  bash: () => "Running command...",
  read: (a) => `Reading ${shortPath(a?.path as string)}...`,
  write: (a) => `Writing ${shortPath(a?.path as string)}...`,
  edit: (a) => `Editing ${shortPath(a?.path as string)}...`,
};

function shortPath(p?: string): string {
  if (!p) return "file";
  const parts = p.split("/");
  return parts[parts.length - 1];
}

export function firstSentence(text: string, maxLen = 60): string {
  const cleaned = text.trim().replace(/^["'\s]+/, "");
  const sentence = cleaned.split(/[.!?]\s/)[0].trim();
  if (sentence.length <= maxLen) return sentence;
  return sentence.slice(0, maxLen).trimEnd() + "\u2026";
}

export function formatElapsed(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return s > 0 ? `${m}m ${s}s` : `${m}m`;
}

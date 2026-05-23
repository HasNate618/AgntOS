export function extractIntent(toolName, args, thinkingText, textContent) {
  if (thinkingText && thinkingText.trim()) {
    return firstSentence(thinkingText);
  }
  if (textContent && textContent.trim()) {
    return firstSentence(textContent);
  }
  return derivedIntent(toolName, args);
}

function derivedIntent(toolName, args) {
  const key = toolName.replace("agntos_", "");
  const fn = INTENT_MAP[key];
  return fn ? fn(args) : toolName;
}

const INTENT_MAP = {
  inspect: (a) => `Inspecting ${a && a.target ? a.target : 'system'}...`,
  propose: () => 'Creating proposal...',
  apply: () => 'Applying changes...',
  rollback: () => 'Rolling back...',
  audit: () => 'Viewing audit...',
  memory: () => 'Managing memory...',
  bash: () => 'Running command...',
  read: (a) => `Reading ${shortPath(a && a.path)}...`,
  write: (a) => `Writing ${shortPath(a && a.path)}...`,
  edit: (a) => `Editing ${shortPath(a && a.path)}...`,
};

function shortPath(p) {
  if (!p) return 'file';
  const parts = p.split('/');
  return parts[parts.length - 1];
}

export function firstSentence(text, maxLen = 60) {
  const cleaned = text.trim().replace(/^["'\s]+/, '');
  const sentence = cleaned.split(/[.!?]\s/)[0].trim();
  if (sentence.length <= maxLen) return sentence;
  return sentence.slice(0, maxLen).trimEnd() + '\u2026';
}

export function formatElapsed(secs) {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return s > 0 ? `${m}m ${s}s` : `${m}m`;
}

export const TOOLS = {
  inspect: { color: "#22c55e", icon: "🔍" },
  propose: { color: "#6366f1", icon: "📋" },
  apply: { color: "#f59e0b", icon: "✅" },
  rollback: { color: "#ef4444", icon: "↩️" },
  audit: { color: "#a1a1aa", icon: "📜" },
  memory: { color: "#8b5cf6", icon: "🧠" },
  bash: { color: "#06b6d4", icon: "💻" },
  read: { color: "#94a3b8", icon: "📄" },
  write: { color: "#fb923c", icon: "✏️" },
  edit: { color: "#f97316", icon: "🔀" },
};

export function getToolMeta(name) {
  const key = name.replace("agntos_", "");
  return TOOLS[key] || { color: "#94a3b8", icon: "🔧" };
}
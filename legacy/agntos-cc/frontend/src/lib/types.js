export const TOOLS = {
  propose: { color: "#6366f1", icon: "📋" },
  audit: { color: "#a1a1aa", icon: "📜" },
  option: { color: "#22c55e", icon: "📖" },
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
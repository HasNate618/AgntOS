import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { execSync } from "node:child_process";

const AGNTCTL = process.env.AGNTCTL_PATH || "agntctl";
const CONFIG_DIR = "/etc/agntos";

function quote(a: string) {
  return a.includes(' ') ? `"${a}"` : a;
}

function run(args: string[], timeout = 30000): { content: Array<{ type: string; text: string }> } {
  try {
    const result = execSync(`${AGNTCTL} ${args.map(quote).join(" ")}`, {
      encoding: "utf-8",
      timeout,
      maxBuffer: 10 * 1024 * 1024,
    });
    return { content: [{ type: "text" as const, text: result.trim() }] };
  } catch (e: any) {
    const msg = e.stderr || e.stdout || e.message || String(e);
    return { content: [{ type: "text" as const, text: msg }], isError: true };
  }
}

export default function (pi: ExtensionAPI) {
  pi.registerTool({
    name: "agntos_propose",
    label: "AgntOS Propose",
    description:
      "Generate a NixOS configuration change proposal. Describe what you want to change and agntctl will generate the Nix files. Returns a proposal ID for review and application from the UI.",
    parameters: Type.Object({
      prompt: Type.String({
        description: "Description of the desired change, e.g. 'install nginx and enable it as a service'",
      }),
    }),
    promptGuidelines: [
      "Use agntos_propose to stage configuration changes. Present the proposal to the user. The user applies from the UI.",
      "Use agntos_option to look up option docs before proposing changes with unfamiliar options.",
    ],
    async execute(_toolCallId: string, params: { prompt: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["propose", "--config-dir", CONFIG_DIR, params.prompt]);
    },
  });

  pi.registerTool({
    name: "agntos_audit",
    label: "AgntOS Audit",
    description:
      "View the AgntOS audit log. Lists all system mutations (proposals, applies, rollbacks) with timestamps and outcomes.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.String({
          description: "Optional action filter: list (show recent entries), show <id> (show details for one entry)",
        })
      ),
      id: Type.Optional(
        Type.String({
          description: "Audit entry ID to show details for (only used with 'show' action)",
        })
      ),
    }),
    async execute(_toolCallId: string, params: { action?: string; id?: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      const action = params.action || "list";
      if (action === "show" && params.id) {
        return run(["audit", "show", params.id]);
      }
      return run(["audit", "list", "--limit", "50"]);
    },
  });

  pi.registerTool({
    name: "agntos_option",
    label: "NixOS Option Lookup",
    description:
      "Look up a NixOS option's type, default, description, and example. Use before proposing changes with unfamiliar options.",
    parameters: Type.Object({
      option: Type.String({
        description: "The NixOS option path, e.g. services.nginx.enable",
      }),
    }),
    async execute(_toolCallId: string, params: { option: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["option", params.option], 30000);
    },
  });

  pi.registerTool({
    name: "agntos_memory",
    label: "AgntOS Memory",
    description:
      "Read or update AgntOS curated memory (MEMORY.md and USER.md). Stores user preferences, system facts, and learned behavior.",
    parameters: Type.Object({
      action: Type.String({
        description: "What to do: 'show' (read memories), 'add' (append a new fact), 'replace' (overwrite with new content)",
      }),
      content: Type.Optional(
        Type.String({
          description: "Content to add or replace (required for 'add' and 'replace' actions)",
        })
      ),
    }),
    async execute(
      _toolCallId: string,
      params: { action: string; content?: string },
      _signal: AbortSignal,
      _onUpdate: any,
      _ctx: any
    ) {
      switch (params.action) {
        case "show":
          return run(["memory", "show"]);
        case "add":
          if (!params.content) return { content: [{ type: "text", text: "Error: content required for 'add' action" }], isError: true };
          return run(["memory", "add", params.content]);
        case "replace":
          if (!params.content) return { content: [{ type: "text", text: "Error: content required for 'replace' action" }], isError: true };
          return run(["memory", "replace", params.content]);
        default:
          return run(["memory", "show"]);
      }
    },
  });

  pi.registerTool({
    name: "agntos_bash",
    label: "AgntOS Bash",
    description:
      "Execute a shell command and return its output. Use for system administration, checking services, running package managers, etc.",
    parameters: Type.Object({
      command: Type.String({
        description: "Shell command to execute",
      }),
    }),
    async execute(_toolCallId: string, params: { command: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["bash", params.command], 60000);
    },
  });

  pi.registerTool({
    name: "agntos_read",
    label: "AgntOS Read",
    description:
      "Read the contents of a file. Returns the full file content or an error if the file doesn't exist.",
    parameters: Type.Object({
      path: Type.String({
        description: "Absolute path to the file to read",
      }),
    }),
    async execute(_toolCallId: string, params: { path: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["read", params.path]);
    },
  });

  pi.registerTool({
    name: "agntos_write",
    label: "AgntOS Write",
    description:
      "Create or overwrite a file with the given content. Logs the write to the audit trail.",
    parameters: Type.Object({
      path: Type.String({
        description: "Absolute path to the file to write",
      }),
      content: Type.String({
        description: "Content to write to the file",
      }),
    }),
    async execute(_toolCallId: string, params: { path: string; content: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["write", params.path, "--content", params.content]);
    },
  });

  pi.registerTool({
    name: "agntos_edit",
    label: "AgntOS Edit",
    description:
      "Edit a file by replacing old text with new text. Logs the edit to the audit trail.",
    parameters: Type.Object({
      path: Type.String({
        description: "Absolute path to the file to edit",
      }),
      old_text: Type.String({
        description: "Text to find and replace",
      }),
      new_text: Type.String({
        description: "Replacement text",
      }),
    }),
    async execute(
      _toolCallId: string,
      params: { path: string; old_text: string; new_text: string },
      _signal: AbortSignal,
      _onUpdate: any,
      _ctx: any
    ) {
      return run(["edit", params.path, "--old", params.old_text, "--new", params.new_text]);
    },
  });
}
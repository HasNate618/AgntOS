import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { execSync } from "node:child_process";

const AGNTCTL = "agntctl";
const CONFIG_DIR = "/etc/agntos";

function run(args: string[], timeout = 30000): { content: Array<{ type: string; text: string }> } {
  try {
    const result = execSync(`${AGNTCTL} ${args.join(" ")}`, {
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
    name: "agntos_inspect",
    label: "AgntOS Inspect",
    description:
      "Inspect AgntOS system state. Returns CPU, memory, disks, network, GPU, services, or general system info.",
    parameters: Type.Object({
      target: Type.Optional(
        Type.String({
          description:
            "What to inspect: system (overview), cpu, memory, disks, network, gpu, services. Defaults to system.",
        })
      ),
    }),
    async execute(_toolCallId: string, params: { target?: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["inspect", params.target || "system"]);
    },
  });

  pi.registerTool({
    name: "agntos_propose",
    label: "AgntOS Propose",
    description:
      "Generate a NixOS configuration change proposal. Describe what you want to change and agntctl will generate the Nix files. Returns a proposal ID for use with agntos_apply.",
    parameters: Type.Object({
      prompt: Type.String({
        description: "Description of the desired change, e.g. 'install nginx and enable it as a service'",
      }),
    }),
    promptGuidelines: [
      "Use agntos_propose before agntos_apply. Never apply changes without a proposal.",
      "If the user just says 'install nginx', first use agntos_inspect to check current state, then agntos_propose.",
    ],
    async execute(_toolCallId: string, params: { prompt: string }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      return run(["propose", "--config-dir", CONFIG_DIR, params.prompt]);
    },
  });

  pi.registerTool({
    name: "agntos_apply",
    label: "AgntOS Apply",
    description:
      "Apply a NixOS proposal by ID. REQUIRES USER CONFIRMATION before executing. Always present the proposal details to the user before calling this tool.",
    parameters: Type.Object({
      proposalId: Type.String({
        description: "The ID of the proposal to apply (returned by agntos_propose)",
      }),
    }),
    promptGuidelines: [
      "Never call agntos_apply without first presenting the proposal to the user and getting their approval via this tool's confirmation dialog.",
      "agntos_apply triggers a nixos-rebuild which modifies the system configuration.",
    ],
    async execute(
      _toolCallId: string,
      params: { proposalId: string },
      _signal: AbortSignal,
      _onUpdate: any,
      ctx: any
    ) {
      const approved = await ctx.ui.confirm({
        title: `Apply proposal ${params.proposalId}?`,
        message: "This will modify your NixOS configuration and trigger a nixos-rebuild. Continue?",
      });

      if (!approved) {
        return { content: [{ type: "text", text: "Proposal application cancelled by user." }], isError: true };
      }

      return run(["apply", "--config-dir", CONFIG_DIR, params.proposalId], 120000);
    },
  });

  pi.registerTool({
    name: "agntos_rollback",
    label: "AgntOS Rollback",
    description:
      "Roll back to a previous NixOS generation. Specify the generation number or use the latest successful apply as the rollback target.",
    parameters: Type.Object({
      generation: Type.Optional(
        Type.Integer({
          description: "Optional generation number to roll back to. Omit to rollback the last apply.",
        })
      ),
    }),
    async execute(_toolCallId: string, params: { generation?: number }, _signal: AbortSignal, _onUpdate: any, _ctx: any) {
      const args = ["rollback"];
      if (params.generation !== undefined) {
        args.push("--generation", String(params.generation));
      }
      return run(args, 120000);
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
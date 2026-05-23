import { useEffect, useState } from "react";
import { Check, X, Undo2 } from "lucide-react";

import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { Card } from "./ui/card";
import { Separator } from "./ui/separator";
import { useTauriInvoke } from "../hooks/useTauriInvoke";

interface Proposal {
  id: string;
  status: "pending" | "applied" | "dismissed";
  prompt?: string;
  description?: string;
  generation?: number;
}

interface AuditEntry {
  id: string;
  action: string;
  type: string;
  description?: string;
  message?: string;
  timestamp?: string;
  proposal_id?: string;
  generation?: number;
}

export default function ProposalsPage() {
  const proposals = useTauriInvoke<Proposal[]>("list_proposals");
  const audit = useTauriInvoke<AuditEntry[]>("list_audit_entries");
  const applyProposal = useTauriInvoke("apply_proposal");
  const rollbackTo = useTauriInvoke("rollback_to");

  const [dismissedIds, setDismissedIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    proposals.execute();
    audit.execute({ limit: 20 });
  }, []);

  const pending = (proposals.data ?? [])
    .filter((p) => p.status === "pending" && !dismissedIds.has(p.id));

  const applied = (audit.data ?? [])
    .filter((e) => e.action === "apply" || e.type === "apply");

  async function handleApply(id: string) {
    try {
      await applyProposal.execute({ id });
      await Promise.all([
        proposals.execute(),
        audit.execute({ limit: 20 }),
      ]);
    } catch {
      // error handled by hook
    }
  }

  function handleDismiss(id: string) {
    setDismissedIds((prev) => new Set(prev).add(id));
  }

  async function handleRollback(generation?: number) {
    if (generation == null) return;
    try {
      await rollbackTo.execute({ generation });
      await Promise.all([
        proposals.execute(),
        audit.execute({ limit: 20 }),
      ]);
    } catch {
      // error handled by hook
    }
  }

  const loading = proposals.loading || audit.loading;

  return (
    <div className="p-6 overflow-y-auto">
      <section className="mb-6">
        <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3 px-0.5">
          Pending
        </h3>

        {loading && pending.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground text-sm">
            Loading...
          </div>
        ) : pending.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-foreground text-sm font-medium">
              No pending mutations
            </p>
            <p className="text-muted-foreground text-xs mt-1">
              Ask the agent to create a proposal
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {pending.map((p) => (
              <Card
                key={p.id}
                className="p-4 gap-3 border-l-[3px]"
                style={{ borderLeftColor: "var(--warning)" }}
              >
                <div className="flex items-center justify-between">
                  <span className="text-xs text-primary font-mono">
                    {p.id}
                  </span>
                  <Badge
                    variant="outline"
                    className="bg-[color-mix(in_oklab,var(--warning)_15%,transparent)] text-[var(--warning)] border-none"
                  >
                    Pending
                  </Badge>
                </div>
                {(p.prompt || p.description) && (
                  <p className="text-sm text-muted-foreground leading-relaxed">
                    {p.prompt || p.description}
                  </p>
                )}
                <div className="flex gap-2">
                  <Button
                    onClick={() => handleApply(p.id)}
                    className="text-white"
                    style={{ backgroundColor: "var(--success)" }}
                    size="sm"
                  >
                    <Check className="size-3.5" />
                    Apply
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => handleDismiss(p.id)}
                    size="sm"
                  >
                    <X className="size-3.5" />
                    Dismiss
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        )}
      </section>

      <Separator className="mb-6" />

      <section>
        <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3 px-0.5">
          Applied
        </h3>

        {loading && applied.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground text-sm">
            Loading...
          </div>
        ) : applied.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-foreground text-sm font-medium">
              No applied mutations yet
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {applied.map((entry) => (
              <Card
                key={entry.id}
                className="p-4 gap-3 border-l-[3px]"
                style={{ borderLeftColor: "var(--success)" }}
              >
                <div className="flex items-center justify-between">
                  <span className="text-xs text-primary font-mono">
                    {entry.proposal_id || entry.id}
                  </span>
                  <Badge
                    variant="outline"
                    className="bg-[color-mix(in_oklab,var(--success)_15%,transparent)] text-[var(--success)] border-none"
                  >
                    Applied
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  {entry.description || entry.message || entry.action || ""}
                </p>
                <Button
                  variant="outline"
                  onClick={() => handleRollback(entry.generation)}
                  size="sm"
                  className="border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground"
                >
                  <Undo2 className="size-3.5" />
                  Revert
                </Button>
              </Card>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

import type { FC } from "react";
import { Check, X, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export type ApprovalData = {
  id: string;
  title?: string;
  message?: string;
  resolved?: boolean;
  rejected?: boolean;
  onApprove?: (id: string) => void;
  onReject?: (id: string) => void;
};

export const ApprovalCard: FC<{ data: ApprovalData }> = ({ data }) => {
  return (
    <Card className="my-2 border-l-[3px] border-[var(--warning)] py-3 gap-0">
      <CardContent className="!px-3 !py-0 flex flex-col gap-3">
        <div className="flex items-center gap-2">
          <AlertTriangle className="size-4 text-[var(--warning)] shrink-0" />
          <span className="font-medium text-sm">{data.title || "Approval required"}</span>
          <Badge
            variant="outline"
            className="ml-auto border-none bg-[color-mix(in_oklab,var(--warning)_12%,transparent)] text-[var(--warning)]"
          >
            Confirm
          </Badge>
        </div>
        {data.message && (
          <p className="text-xs text-muted-foreground leading-relaxed">{data.message}</p>
        )}
        {data.resolved ? (
          <span className="text-xs text-muted-foreground">
            {data.rejected ? "Dismissed" : "Approved"}
          </span>
        ) : (
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={() => data.onApprove?.(data.id)}
              className="bg-[var(--success)] text-[#141416] hover:bg-[var(--success)]/90"
            >
              <Check className="size-3.5" />
              Approve
            </Button>
            <Button size="sm" variant="outline" onClick={() => data.onReject?.(data.id)}>
              <X className="size-3.5" />
              Dismiss
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

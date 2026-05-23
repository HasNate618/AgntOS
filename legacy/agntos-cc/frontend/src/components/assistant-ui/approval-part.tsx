import type { FC } from "react";
import { ApprovalCard } from "@/components/assistant-ui/approval-card";
import { useApprovalHandlers } from "@/hooks/AgntRuntimeProvider";

export const ApprovalPart: FC<{ data: Record<string, unknown> }> = ({ data }) => {
  const { onApprove, onReject } = useApprovalHandlers();
  return (
    <ApprovalCard
      data={{
        id: String(data.id ?? ""),
        title: data.title as string | undefined,
        message: data.message as string | undefined,
        resolved: Boolean(data.resolved),
        rejected: Boolean(data.rejected),
        onApprove,
        onReject,
      }}
    />
  );
};

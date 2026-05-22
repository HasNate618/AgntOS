import { useEffect } from "react";
import { Circle, Monitor, Cpu, Shield } from "lucide-react";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { useAgentStore } from "@/hooks/TauriProvider";
import { useTauriInvoke } from "@/hooks/useTauriInvoke";

interface SystemInfo {
  cpu?: string;
  ram?: string;
  disk?: string;
  failed_units?: number;
}

function StatusDot({ color }: { color: string }) {
  return <Circle size={8} fill={color} stroke={color} className="inline-block" />;
}

export default function StatusPage() {
  const { state: { connection } } = useAgentStore();
  const { data: sysInfo, loading, error, execute } = useTauriInvoke<SystemInfo>("get_system_info");

  useEffect(() => {
    execute();
  }, [execute]);

  return (
    <div className="p-4 grid grid-cols-1 md:grid-cols-3 gap-3">
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Monitor size={14} className="text-muted-foreground" />
            <span className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
              Agent
            </span>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-x-3 gap-y-1.5">
            <span className="text-xs text-muted-foreground">Status</span>
            <span className="text-sm text-foreground flex items-center gap-1.5">
              <StatusDot
                color={connection.connected ? "var(--success)" : "var(--destructive)"}
              />
              {connection.connected ? "Connected" : "Disconnected"}
            </span>
            <span className="text-xs text-muted-foreground">State</span>
            <span className="text-sm text-foreground capitalize">{connection.state}</span>
            <span className="text-xs text-muted-foreground">Model</span>
            <span className="text-sm text-foreground font-mono">{connection.model || "\u2014"}</span>
            <span className="text-xs text-muted-foreground">Profile</span>
            <span className="text-sm text-foreground">default</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Cpu size={14} className="text-muted-foreground" />
            <span className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
              System
            </span>
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <span className="text-sm text-muted-foreground">Loading...</span>
          ) : error ? (
            <span className="text-sm" style={{ color: "var(--destructive)" }}>
              Failed to load
            </span>
          ) : (
            <div className="grid grid-cols-2 gap-x-3 gap-y-1.5">
              <span className="text-xs text-muted-foreground">CPU</span>
              <span className="text-sm text-foreground">{sysInfo?.cpu || "\u2014"}</span>
              <span className="text-xs text-muted-foreground">RAM</span>
              <span className="text-sm text-foreground">{sysInfo?.ram || "\u2014"}</span>
              <span className="text-xs text-muted-foreground">Disk</span>
              <span className="text-sm text-foreground">{sysInfo?.disk || "\u2014"}</span>
              <span className="text-xs text-muted-foreground">Failed Units</span>
              <span
                className="text-sm"
                style={{
                  color:
                    sysInfo?.failed_units !== undefined && sysInfo.failed_units > 0
                      ? "var(--destructive)"
                      : "var(--success)",
                }}
              >
                {sysInfo?.failed_units?.toString() ?? "\u2014"}
              </span>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield size={14} className="text-muted-foreground" />
            <span className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
              Watchdog
            </span>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-x-3 gap-y-1.5">
            <span className="text-xs text-muted-foreground">Alerts</span>
            <span className="text-sm text-foreground">0</span>
            <span className="text-xs text-muted-foreground">Last Check</span>
            <span className="text-sm text-foreground">{"\u2014"}</span>
            <span className="text-xs text-muted-foreground">Status</span>
            <span className="text-sm text-foreground flex items-center gap-1.5">
              <StatusDot color="var(--success)" />
              Healthy
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

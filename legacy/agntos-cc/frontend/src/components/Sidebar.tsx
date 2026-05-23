import type { Page } from "@/lib/types";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import AgntLogo from "@/components/AgntLogo";
import {
  MessageSquare,
  Activity,
  FileText,
  Clock,
  Cpu,
} from "lucide-react";

interface SidebarProps {
  activePage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { id: Page; label: string; icon: typeof MessageSquare }[] = [
  { id: "chat", label: "Chat", icon: MessageSquare },
  { id: "status", label: "Status", icon: Activity },
  { id: "proposals", label: "Proposals", icon: FileText },
  { id: "activity", label: "Activity", icon: Clock },
  { id: "models", label: "Models", icon: Cpu },
];

export default function Sidebar({ activePage, onNavigate }: SidebarProps) {
  return (
    <nav className="w-[52px] flex flex-col items-center py-3 gap-0.5 shrink-0 bg-sidebar border-r border-sidebar-border">
      <div className="mb-3 flex items-center justify-center w-9 h-9">
        <AgntLogo size={26} />
      </div>

      {navItems.map((item) => {
        const Icon = item.icon;
        const active = activePage === item.id;
        return (
          <Tooltip key={item.id}>
            <TooltipTrigger asChild>
              <button
                type="button"
                onClick={() => onNavigate(item.id)}
                className={cn(
                  "flex items-center justify-center w-10 h-10 rounded-lg transition-all",
                  active
                    ? "bg-sidebar-primary/15 text-sidebar-primary"
                    : "text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent",
                )}
              >
                <Icon className="w-[18px] h-[18px]" strokeWidth={active ? 2.25 : 2} />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" className="text-xs font-medium">
              {item.label}
            </TooltipContent>
          </Tooltip>
        );
      })}

      <div className="flex-1" />
    </nav>
  );
}

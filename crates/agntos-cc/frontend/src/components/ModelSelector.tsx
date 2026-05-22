import { useCallback, useEffect, useState } from "react";
import { Check, ChevronDown } from "lucide-react";
import { useAgentStore } from "@/hooks/TauriProvider";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export interface ModelOption {
  value: string;
  label: string;
  provider: string;
  modelId: string;
}

interface CatalogResponse {
  options?: ModelOption[];
  providers?: {
    id: string;
    endpoint: string;
    error?: string;
    models: { id: string; name: string }[];
  }[];
  selected?: { provider: string; modelId: string; value: string } | null;
}

function optionsFromCatalog(data: CatalogResponse): ModelOption[] {
  if (data.options?.length) {
    return data.options.map((o) => ({
      value: String(o.value ?? `${o.provider}/${o.modelId}`),
      label: String(o.label ?? o.modelId),
      provider: String(o.provider),
      modelId: String(o.modelId),
    }));
  }
  const out: ModelOption[] = [];
  for (const p of data.providers ?? []) {
    for (const m of p.models ?? []) {
      out.push({
        value: `${p.id}/${m.id}`,
        label: `${p.id} · ${m.name}`,
        provider: p.id,
        modelId: m.id,
      });
    }
  }
  return out;
}

export default function ModelSelector({ className }: { className?: string }) {
  const { dispatch } = useAgentStore();
  const [options, setOptions] = useState<ModelOption[]>([]);
  const [selected, setSelected] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const applyModel = useCallback(
    async (opt: ModelOption) => {
      if (!window.__TAURI__) return;
      setError(null);
      try {
        await window.__TAURI__.core.invoke("set_chat_model", {
          provider: opt.provider,
          model_id: opt.modelId,
        });
        await window.__TAURI__.core.invoke("set_model", {
          provider: opt.provider,
          model_id: opt.modelId,
        });
        dispatch({
          type: "SET_CONNECTION",
          payload: { model: opt.label },
        });
      } catch (e) {
        setError(String(e));
      }
    },
    [dispatch],
  );

  const refresh = useCallback(async () => {
    if (!window.__TAURI__) return;
    setLoading(true);
    setError(null);
    try {
      const data = (await window.__TAURI__.core.invoke(
        "list_model_catalog",
      )) as CatalogResponse;
      const parsed = optionsFromCatalog(data);
      setOptions(parsed);
      const sel = data.selected;
      if (sel?.value && parsed.some((o) => o.value === sel.value)) {
        setSelected(sel.value);
      } else if (sel?.provider && sel.modelId) {
        const value = `${sel.provider}/${sel.modelId}`;
        if (parsed.some((o) => o.value === value)) {
          setSelected(value);
        } else if (parsed.length > 0) {
          setSelected(parsed[0].value);
        }
      } else if (parsed.length > 0 && !selected) {
        setSelected(parsed[0].value);
      }
    } catch (e) {
      setOptions([]);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handlePick = async (value: string) => {
    setSelected(value);
    const opt = options.find((o) => o.value === value);
    if (opt) await applyModel(opt);
  };

  const current = options.find((o) => o.value === selected);
  const grouped = options.reduce<Record<string, ModelOption[]>>((acc, o) => {
    (acc[o.provider] ??= []).push(o);
    return acc;
  }, {});

  if (loading && options.length === 0) {
    return (
      <span className={cn("text-xs text-muted-foreground px-2", className)}>
        Models…
      </span>
    );
  }

  if (options.length === 0) {
    return error ? (
      <span
        className={cn("text-xs text-destructive truncate max-w-[200px]", className)}
        title={error}
      >
        No models
      </span>
    ) : null;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className={cn(
            "h-8 max-w-[240px] gap-1 rounded-full border-input bg-muted/50 font-medium",
            className,
          )}
          aria-label="Select model"
        >
          <span className="truncate text-xs">
            {current?.label.replace(/^[^·]+ · /, "") ?? "Model"}
          </span>
          <ChevronDown className="size-3.5 shrink-0 opacity-60" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="max-h-80 w-72 overflow-y-auto">
        {Object.entries(grouped).map(([provider, items], i) => (
          <DropdownMenuGroup key={provider}>
            {i > 0 && <DropdownMenuSeparator />}
            <DropdownMenuLabel className="text-xs text-muted-foreground">
              {provider}
            </DropdownMenuLabel>
            {items.map((o) => (
              <DropdownMenuItem
                key={o.value}
                onClick={() => handlePick(o.value)}
                className="flex items-center justify-between gap-2"
              >
                <span className="truncate">{o.label.replace(`${provider} · `, "")}</span>
                {selected === o.value && <Check className="size-3.5 shrink-0" />}
              </DropdownMenuItem>
            ))}
          </DropdownMenuGroup>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

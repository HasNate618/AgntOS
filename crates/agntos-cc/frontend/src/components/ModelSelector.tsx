import { useCallback, useEffect, useState } from "react";
import { ChevronDown } from "lucide-react";
import { useAgentStore } from "@/hooks/TauriProvider";
import { cn } from "@/lib/utils";

export interface ModelOption {
  value: string;
  label: string;
  provider: string;
  modelId: string;
}

interface CatalogResponse {
  options?: ModelOption[];
  providers?: { id: string; endpoint: string; error?: string; models: { id: string; name: string }[] }[];
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

  const refresh = useCallback(async () => {
    if (!window.__TAURI__) return;
    setLoading(true);
    try {
      const data = (await window.__TAURI__.core.invoke("list_model_catalog")) as CatalogResponse;
      const parsed = optionsFromCatalog(data);
      setOptions(parsed);
      if (parsed.length > 0 && !selected) {
        setSelected(parsed[0].value);
      }
    } catch {
      setOptions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleChange = async (value: string) => {
    setSelected(value);
    const opt = options.find((o) => o.value === value);
    if (!opt || !window.__TAURI__) return;
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
    } catch {
      // ignore
    }
  };

  if (loading && options.length === 0) {
    return (
      <span className={cn("text-xs text-muted-foreground px-2", className)}>Models…</span>
    );
  }

  if (options.length === 0) return null;

  const grouped = options.reduce<Record<string, ModelOption[]>>((acc, o) => {
    (acc[o.provider] ??= []).push(o);
    return acc;
  }, {});

  return (
    <div className={cn("relative flex items-center", className)}>
      <select
        value={selected}
        onChange={(e) => handleChange(e.target.value)}
        className="h-8 max-w-[220px] appearance-none truncate rounded-full border border-input bg-muted/50 pl-3 pr-8 text-xs font-medium text-foreground outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30"
        aria-label="Model"
      >
        {Object.entries(grouped).map(([provider, items]) => (
          <optgroup key={provider} label={provider}>
            {items.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label.replace(`${provider} · `, "")}
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      <ChevronDown className="pointer-events-none absolute right-2 size-3.5 text-muted-foreground" />
    </div>
  );
}

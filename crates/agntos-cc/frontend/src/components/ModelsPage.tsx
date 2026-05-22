import { useCallback, useEffect, useState } from "react";
import { Plus, Trash2, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { useTauriInvoke } from "@/hooks/useTauriInvoke";

interface ModelProfile {
  endpoint: string;
  model?: string;
  api_key_env?: string;
}

interface ModelsConfig {
  default: ModelProfile;
  profiles?: Record<string, ModelProfile>;
  routing?: Record<string, string>;
}

interface CatalogProvider {
  id: string;
  endpoint: string;
  error?: string;
  models: { id: string; name: string }[];
}

export default function ModelsPage() {
  const configInvoke = useTauriInvoke<ModelsConfig>("get_models_config");
  const addProvider = useTauriInvoke<string>("add_model_provider");
  const removeProfile = useTauriInvoke<string>("remove_model_profile");
  const probe = useTauriInvoke<{ models: { id: string; name: string }[] }>("probe_provider_models");
  const catalogInvoke = useTauriInvoke<{ providers: CatalogProvider[] }>("list_model_catalog");

  const [config, setConfig] = useState<ModelsConfig | null>(null);
  const [catalog, setCatalog] = useState<CatalogProvider[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [probeOk, setProbeOk] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [endpoint, setEndpoint] = useState("");
  const [apiKeyEnv, setApiKeyEnv] = useState("AGNTOS_API_KEY");

  const load = useCallback(async () => {
    setError(null);
    try {
      const c = await configInvoke.execute();
      setConfig(c);
      const cat = await catalogInvoke.execute();
      setCatalog(cat?.providers ?? []);
    } catch (e) {
      setError(String(e));
      setConfig(null);
      setCatalog([]);
    }
  }, []);

  useEffect(() => {
    load();
  }, []);

  const profiles = config?.profiles
    ? Object.entries(config.profiles).map(([n, p]) => ({ name: n, ...p }))
    : [];

  const handleProbe = async () => {
    if (!endpoint.trim()) return;
    setProbeOk(null);
    setError(null);
    try {
      const res = await probe.execute({
        endpoint: endpoint.trim(),
        api_key_env: apiKeyEnv.trim() || undefined,
      });
      const count = res?.models?.length ?? 0;
      setProbeOk(`Reachable — ${count} model(s) listed`);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleAdd = async () => {
    if (!name.trim() || !endpoint.trim()) return;
    setError(null);
    setProbeOk(null);
    try {
      await addProvider.execute({
        name: name.trim(),
        endpoint: endpoint.trim(),
        api_key_env: apiKeyEnv.trim() || undefined,
      });
      setName("");
      setEndpoint("");
      await load();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRemove = async (profileName: string) => {
    setError(null);
    try {
      await removeProfile.execute({ name: profileName });
      await load();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-12 items-center border-b border-border px-5 shrink-0">
        <h1
          className="text-base font-semibold"
          style={{ fontFamily: "var(--font-display)" }}
        >
          Providers
        </h1>
      </header>
      <div className="flex-1 overflow-y-auto p-5 max-w-2xl mx-auto w-full space-y-6">
        <p className="text-sm text-muted-foreground">
          Add OpenAI-compatible API endpoints (e.g.{" "}
          <code className="text-xs">http://10.0.0.45/bifrost/v1</code>). API keys are read from
          the named environment variable on this machine — never stored in the UI. Choose
          models in the chat composer.
        </p>

        {error && (
          <Card className="border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
            {error}
          </Card>
        )}
        {probeOk && (
          <Card className="border-[color-mix(in_oklab,var(--success)_35%,transparent)] bg-[color-mix(in_oklab,var(--success)_8%,transparent)] p-3 text-sm text-[var(--success)]">
            {probeOk}
          </Card>
        )}

        <Card className="p-4 space-y-3">
          <h2 className="text-sm font-semibold">Add provider</h2>
          <div className="grid gap-2">
            <Input
              placeholder="Provider id (e.g. gateway, local)"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <Input
              placeholder="Endpoint URL (https://…/v1)"
              value={endpoint}
              onChange={(e) => setEndpoint(e.target.value)}
            />
            <Input
              placeholder="API key environment variable"
              value={apiKeyEnv}
              onChange={(e) => setApiKeyEnv(e.target.value)}
            />
          </div>
          <div className="flex gap-2">
            <Button type="button" size="sm" variant="outline" onClick={handleProbe} disabled={probe.loading}>
              <Zap className="size-4 mr-1" />
              Test endpoint
            </Button>
            <Button type="button" size="sm" onClick={handleAdd} disabled={addProvider.loading}>
              <Plus className="size-4 mr-1" />
              Add provider
            </Button>
          </div>
        </Card>

        <Separator />

        <div className="space-y-2">
          <h2 className="text-sm font-semibold">Configured providers</h2>
          {config?.default && (
            <Card className="p-3">
              <p className="font-medium text-sm">default</p>
              <p className="text-xs font-mono text-muted-foreground break-all mt-1">
                {config.default.endpoint}
              </p>
              {catalog.find((c) => c.id === "default")?.error && (
                <p className="text-xs text-destructive mt-1">
                  {catalog.find((c) => c.id === "default")?.error}
                </p>
              )}
            </Card>
          )}
          {profiles.map((p) => (
            <Card key={p.name} className="flex items-start justify-between gap-3 p-3">
              <div className="min-w-0 flex-1">
                <p className="font-medium text-sm">{p.name}</p>
                <p className="text-xs font-mono text-muted-foreground break-all mt-1">
                  {p.endpoint}
                </p>
                {catalog.find((c) => c.id === p.name)?.error ? (
                  <p className="text-xs text-destructive mt-1">
                    {catalog.find((c) => c.id === p.name)?.error}
                  </p>
                ) : (
                  <p className="text-xs text-muted-foreground mt-1">
                    {(catalog.find((c) => c.id === p.name)?.models.length ?? 0)} models available
                  </p>
                )}
              </div>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="shrink-0 text-destructive hover:text-destructive"
                onClick={() => handleRemove(p.name)}
              >
                <Trash2 className="size-4" />
              </Button>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}

import { useCallback, useState } from "react";

export function useTauriInvoke<T = unknown>(cmd: string) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [loading, setLoading] = useState(false);

  const execute = useCallback(
    async (args?: Record<string, unknown>): Promise<T> => {
      if (!window.__TAURI__) {
        const err = new Error("Tauri API not available");
        setError(err);
        throw err;
      }
      setLoading(true);
      setError(null);
      try {
        const result = (await window.__TAURI__.core.invoke(cmd, args)) as T;
        setData(result);
        return result;
      } catch (e) {
        const err = e instanceof Error ? e : new Error(String(e));
        setError(err);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [cmd],
  );

  return { data, error, loading, execute };
}

import { useEffect } from "react";

declare global {
  interface Window {
    __TAURI__?: {
      event: {
        listen: (event: string, callback: (payload: { payload: string }) => void) => Promise<() => void>;
      };
      core: {
        invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      };
    };
    __AGNTOS_BRIDGE_STATUS__?: { connected: boolean; state: string };
  }
}

export function useTauriEvent(
  event: string,
  callback: (payload: string) => void,
  enabled = true,
) {
  useEffect(() => {
    if (!enabled || !window.__TAURI__) return;

    let unlisten: (() => void) | undefined;

    window.__TAURI__.event
      .listen(event, (data) => {
        callback(data.payload);
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((e) => {
        console.error(`listen ${event} failed`, e);
      });

    return () => {
      unlisten?.();
    };
  }, [event, callback, enabled]);
}

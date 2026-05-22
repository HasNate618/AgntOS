import { createContext, useContext, useEffect, useReducer, type ReactNode } from "react";
import type { ConnectionStatus, ChatEntry } from "../lib/types";

interface AgentState {
  connection: ConnectionStatus;
  messages: ChatEntry[];
}

type Action =
  | { type: "SET_CONNECTION"; payload: Partial<ConnectionStatus> }
  | { type: "ADD_MESSAGE"; payload: ChatEntry }
  | { type: "UPDATE_LAST_MESSAGE"; payload: Partial<ChatEntry> }
  | { type: "UPDATE_TOOL_MESSAGE"; payload: { name: string; result?: string | null; state: "done" } }
  | { type: "UPDATE_APPROVAL"; payload: { id: string; resolved: boolean; rejected?: boolean } }
  | { type: "SET_MESSAGES"; payload: ChatEntry[] }
  | { type: "SET_STATE"; payload: ConnectionStatus["state"] };

function agentReducer(state: AgentState, action: Action): AgentState {
  switch (action.type) {
    case "SET_CONNECTION":
      return { ...state, connection: { ...state.connection, ...action.payload } };
    case "ADD_MESSAGE":
      return { ...state, messages: [...state.messages, action.payload] };
    case "UPDATE_LAST_MESSAGE": {
      const msgs = [...state.messages];
      if (msgs.length > 0) {
        msgs[msgs.length - 1] = { ...msgs[msgs.length - 1], ...action.payload };
      }
      return { ...state, messages: msgs };
    }
    case "UPDATE_TOOL_MESSAGE": {
      const msgs = [...state.messages];
      let idx = -1;
      for (let i = msgs.length - 1; i >= 0; i--) {
        const m = msgs[i];
        if (m.role === "tool" && m.name === action.payload.name && m.state !== "done") {
          idx = i;
          break;
        }
      }
      if (idx >= 0) {
        msgs[idx] = { ...msgs[idx], ...action.payload };
      } else {
        msgs.push({
          role: "tool",
          name: action.payload.name,
          result: action.payload.result,
          state: "done",
          timestamp: Date.now(),
        });
      }
      return { ...state, messages: msgs };
    }
    case "UPDATE_APPROVAL": {
      const msgs = state.messages.map((m) =>
        m.role === "approval" && m.id === action.payload.id
          ? { ...m, resolved: action.payload.resolved, rejected: action.payload.rejected }
          : m,
      );
      return { ...state, messages: msgs };
    }
    case "SET_MESSAGES":
      return { ...state, messages: action.payload };
    case "SET_STATE":
      return { ...state, connection: { ...state.connection, state: action.payload } };
    default:
      return state;
  }
}

const AgentContext = createContext<{
  state: AgentState;
  dispatch: React.Dispatch<Action>;
} | null>(null);

async function pollBridgeStatus(dispatch: React.Dispatch<Action>) {
  for (let i = 0; i < 30; i++) {
    const s = window.__AGNTOS_BRIDGE_STATUS__;
    if (s && s.connected) {
      dispatch({
        type: "SET_CONNECTION",
        payload: { connected: true, state: (s.state as ConnectionStatus["state"]) || "idle" },
      });
      return;
    }
    await new Promise((r) => setTimeout(r, 200));
  }
}

export function TauriProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(agentReducer, {
    connection: { connected: false, model: null, state: "disconnected" },
    messages: [],
  });

  useEffect(() => {
    pollBridgeStatus(dispatch);

    if (!window.__TAURI__) return;

    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;

    const unlistenFns: (() => void)[] = [];

    listen("agent:connected", () => {
      dispatch({ type: "SET_CONNECTION", payload: { connected: true } });
      invoke("get_connection_status")
        .then((status) => {
          const s = status as { model?: { name?: string } | string };
          if (s?.model) {
            const modelName = typeof s.model === "string" ? s.model : (s.model as { name?: string }).name;
            if (modelName) {
              dispatch({ type: "SET_CONNECTION", payload: { model: modelName } });
            }
          }
        })
        .catch(() => {});
    }).then((fn) => unlistenFns.push(fn)).catch(() => {});

    listen("agent:disconnected", () => {
      dispatch({ type: "SET_CONNECTION", payload: { connected: false, state: "disconnected" } });
    }).then((fn) => unlistenFns.push(fn)).catch(() => {});

    listen("agent:start", () => {
      dispatch({ type: "SET_STATE", payload: "thinking" });
    }).then((fn) => unlistenFns.push(fn)).catch(() => {});

    listen("agent:end", () => {
      dispatch({ type: "SET_STATE", payload: "idle" });
    }).then((fn) => unlistenFns.push(fn)).catch(() => {});

    listen("agent:rpc-response", (event) => {
      try {
        const data = JSON.parse(event.payload);
        if (data.command === "set_model" && data.data?.model) {
          const model = data.data.model;
          dispatch({
            type: "SET_CONNECTION",
            payload: { model: model.name || model.id },
          });
        }
      } catch {
        // ignore parse errors
      }
    }).then((fn) => unlistenFns.push(fn)).catch(() => {});

    return () => {
      unlistenFns.forEach((fn) => fn());
    };
  }, [dispatch]);

  return (
    <AgentContext.Provider value={{ state, dispatch }}>
      {children}
    </AgentContext.Provider>
  );
}

export function useAgentStore() {
  const ctx = useContext(AgentContext);
  if (!ctx) throw new Error("useAgentStore must be used within TauriProvider");
  return ctx;
}

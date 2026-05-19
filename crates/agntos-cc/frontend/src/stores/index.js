import { writable } from "svelte/store";

export const connection = writable({
  connected: false,
  model: null,
  state: "disconnected",
});

export const messages = writable([]);

export const proposals = writable([]);
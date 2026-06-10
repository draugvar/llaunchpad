// Thin typed wrapper around `@tauri-apps/api/core::invoke`.
// Each function mirrors one `#[tauri::command]` in src/commands.rs.

import { invoke } from "@tauri-apps/api/core";
import type { Snapshot, TerminalOption, LaunchResult } from "./types";

export const api = {
  getSnapshot: () => invoke<Snapshot>("get_snapshot"),
  listTerminals: () => invoke<TerminalOption[]>("list_terminals"),

  setOllamaHost: (url: string) => invoke("set_ollama_host", { url }),
  setWorkingDir: (dir: string) => invoke("set_working_dir", { dir }),
  setTerminal: (key: string) => invoke("set_terminal", { key }),

  testConnection: (url: string) => invoke<Snapshot>("test_connection", { url }),
  refresh: () => invoke<Snapshot>("refresh"),

  recordSelection: (agent: string | null, model: string | null) =>
    invoke("record_selection", { agent, model }),

  launch: (params: {
    agentIdx: number;
    model: string;
    ollamaHost?: string;
    workingDir?: string;
    terminalKey?: string;
  }) =>
    invoke<LaunchResult>("launch", {
      agentIdx: params.agentIdx,
      model: params.model,
      ollamaHost: params.ollamaHost ?? null,
      workingDir: params.workingDir ?? null,
      terminalKey: params.terminalKey ?? null,
    }),

  restore: (agentIdx: number) =>
    invoke<LaunchResult>("restore", { agentIdx }),

  isAgentRestorable: (agentIdx: number) =>
    invoke<boolean>("is_agent_restorable", { agentIdx }),

  pickDirectory: (startDir?: string) =>
    invoke<string | null>("pick_directory", { startDir: startDir ?? null }),

  dismissStatus: () => invoke("dismiss_status"),
  toggleSettings: () => invoke("toggle_settings"),
};

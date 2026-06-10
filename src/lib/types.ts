// Mirror of the Rust types exposed over IPC.
// Keep these in lockstep with src/ollama/agents.rs, src/ollama/models.rs,
// src/model.rs, src/repository.rs, src/terminal.rs and src/commands.rs.

export type Agent = {
  name: string;
  display: string;
  isGui: boolean;
  logo: string;
};

export type WorldSnapshot = {
  agents: Agent[];
  running: boolean[];
  installed: boolean[];
  cloudModels: string[];
};

export type Status = { message: string; kind: 0 | 1 | 2 };

export type StateSnapshot = {
  ollamaHost: string;
  workingDir: string;
  localModels: string[];
  world: WorldSnapshot | null;
  status: Status;
  refreshing: boolean;
  settingsOpen: boolean;
  firstLoad: boolean;
  lastAgent: string | null;
  lastModel: string | null;
  lastTerminal: string | null;
};

export type ModelRow = { name: string; local: boolean };

export type Snapshot = StateSnapshot & {
  models: ModelRow[];
  restorable: boolean[];
};

export type TerminalOption = {
  key: string;
  label: string;
  available: boolean;
};

export type LaunchResult = { ok: boolean; message: string };

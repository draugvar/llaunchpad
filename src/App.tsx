import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  Rocket,
  RefreshCw,
  Settings as SettingsIcon,
  FolderOpen,
  Power,
  Undo2,
  Globe,
  X,
  CircleAlert,
  CircleCheck,
  Loader2,
} from "lucide-react";

import { api } from "@/lib/api";
import type { Snapshot, TerminalOption } from "@/lib/types";
import { Select, type SelectOption } from "@/ui/select";
import { Button } from "@/ui/button";
import { Badge } from "@/ui/badge";
import { Input } from "@/ui/input";
import { AgentRow } from "@/components/AgentRow";

export function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [terminals, setTerminals] = useState<TerminalOption[]>([]);
  const [agentIdx, setAgentIdx] = useState<string>("");
  const [modelName, setModelName] = useState<string>("");
  const [host, setHost] = useState<string>("");
  const [hostDraft, setHostDraft] = useState<string>("");
  const [workingDir, setWorkingDir] = useState<string>("");
  const [terminalKey, setTerminalKey] = useState<string>("default");
  const [busy, setBusy] = useState<boolean>(false);

  // Initial load
  useEffect(() => {
    void (async () => {
      const [s, t] = await Promise.all([api.getSnapshot(), api.listTerminals()]);
      applySnapshot(s);
      setTerminals(t);
    })();
  }, []);

  // Live updates from the Rust mirror task
  useEffect(() => {
    const unlisten = listen<Snapshot>("state-updated", (e) => {
      applySnapshot(e.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const applySnapshot = useCallback((s: Snapshot) => {
    setSnap(s);
    setHost(s.ollamaHost);
    setHostDraft(s.ollamaHost);
    setWorkingDir(s.workingDir);
    if (s.lastTerminal) setTerminalKey(s.lastTerminal);

    // Pick initial agent: last-used, otherwise the first installed one
    if (!agentIdx || s.firstLoad) {
      if (s.lastAgent && s.world) {
        const idx = s.world.agents.findIndex((a) => a.name === s.lastAgent);
        if (idx >= 0) {
          setAgentIdx(String(idx));
        } else if (s.world.agents.length > 0) {
          setAgentIdx("0");
        }
      } else if (s.world && s.world.agents.length > 0 && !agentIdx) {
        setAgentIdx("0");
      }
    }
    if (!modelName || s.firstLoad) {
      if (s.lastModel && s.models.some((m) => m.name === s.lastModel)) {
        setModelName(s.lastModel);
      } else if (s.models.length > 0) {
        setModelName(s.models[0].name);
      }
    }
  }, [agentIdx, modelName]);

  // ---- derived dropdown data ----
  const world = snap?.world;
  const agents: SelectOption[] = (world?.agents ?? []).map((a, i) => ({
    value: String(i),
    label: a.display,
    badge: <AgentRow agent={a} isGui={a.isGui} running={world!.running[i] ?? false} installed={world!.installed[i] ?? false} restorable={snap!.restorable[i] ?? false} selected={String(i) === agentIdx} />,
    disabled: !(world!.installed[i] ?? false),
  }));

  const models: SelectOption[] = (snap?.models ?? []).map((m) => ({
    value: m.name,
    label: m.name,
    badge: m.local ? <Badge variant="success">local</Badge> : undefined,
  }));

  const terminalOpts: SelectOption[] = terminals.map((t) => ({
    value: t.key,
    label: t.label,
    disabled: !t.available,
  }));

  const canLaunch =
    !busy &&
    agentIdx !== "" &&
    modelName !== "" &&
    (world?.installed[Number(agentIdx)] ?? false);

  // ---- actions ----
  const onLaunch = async () => {
    setBusy(true);
    try {
      await api.launch({
        agentIdx: Number(agentIdx),
        model: modelName,
        ollamaHost: host,
        workingDir,
        terminalKey,
      });
    } finally {
      setBusy(false);
      const s = await api.getSnapshot();
      applySnapshot(s);
    }
  };

  const onRefresh = async () => {
    setBusy(true);
    try {
      const s = await api.refresh();
      applySnapshot(s);
    } finally {
      setBusy(false);
    }
  };

  const onTest = async () => {
    setBusy(true);
    try {
      const s = await api.testConnection(hostDraft);
      applySnapshot(s);
    } finally {
      setBusy(false);
    }
  };

  const onRestore = async () => {
    if (agentIdx === "") return;
    setBusy(true);
    try {
      await api.restore(Number(agentIdx));
    } finally {
      setBusy(false);
    }
  };

  const onPickDir = async () => {
    const picked = await openDialog({ directory: true, multiple: false, defaultPath: workingDir || undefined });
    if (typeof picked === "string" && picked) {
      setWorkingDir(picked);
      await api.setWorkingDir(picked);
    }
  };

  const commitHost = async (url: string) => {
    setHost(url);
    await api.setOllamaHost(url);
  };

  const onAgentChange = (v: string) => {
    setAgentIdx(v);
    const agent = world?.agents[Number(v)];
    if (agent) void api.recordSelection(agent.name, null);
  };

  const onModelChange = (v: string) => {
    setModelName(v);
    void api.recordSelection(null, v);
  };

  const onTerminalChange = (v: string) => {
    setTerminalKey(v);
    void api.setTerminal(v);
  };

  const onDismissStatus = () => void api.dismissStatus();

  if (!snap) {
    return (
      <div className="grid h-full place-items-center text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin" />
      </div>
    );
  }

  const status = snap.status;
  const showStatus = status.kind !== 0 && status.message;

  return (
    <div className="flex h-full flex-col">
      {/* Title bar (transparent on macOS for the traffic-light buttons) */}
      <div
        data-tauri-drag-region
        className="flex h-9 items-center justify-between border-b border-border/50 bg-background/40 px-3"
      >
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="font-semibold text-foreground/80">Llaunchpad</span>
          <span className="opacity-50">v0.7.0</span>
        </div>
        <div className="flex items-center gap-1">
          {snap.refreshing && (
            <span className="inline-flex items-center gap-1 text-[10px] text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              syncing
            </span>
          )}
        </div>
      </div>

      <div className="flex flex-1 flex-col gap-4 p-4">
        {/* Agent */}
        <Section label="Agent" hint={world ? `${world.agents.length} integrations` : ""}>
          <Select
            ariaLabel="Pick an agent"
            value={agentIdx}
            onChange={onAgentChange}
            options={agents}
            placeholder="Pick an agent…"
          />
        </Section>

        {/* Model */}
        <Section label="Model" hint={`${snap.models.length} available`}>
          <Select
            ariaLabel="Pick a model"
            value={modelName}
            onChange={onModelChange}
            options={models}
            placeholder="Pick a model…"
          />
        </Section>

        {/* Status banner */}
        {showStatus && (
          <div
            className={
              "flex items-start gap-2 rounded-md border px-3 py-2 text-sm " +
              (status.kind === 2
                ? "border-rose-500/30 bg-rose-500/10 text-rose-200"
                : "border-emerald-500/30 bg-emerald-500/10 text-emerald-200")
            }
          >
            {status.kind === 2 ? (
              <CircleAlert className="mt-0.5 h-4 w-4 shrink-0" />
            ) : (
              <CircleCheck className="mt-0.5 h-4 w-4 shrink-0" />
            )}
            <span className="flex-1 break-words">{status.message}</span>
            <button
              onClick={onDismissStatus}
              className="text-muted-foreground transition-colors hover:text-foreground"
              aria-label="Dismiss"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          </div>
        )}

        {/* Actions */}
        <div className="mt-auto flex items-center gap-2 pt-2">
          <Button onClick={onLaunch} disabled={!canLaunch} size="lg" className="flex-1">
            {busy ? <Loader2 className="h-4 w-4 animate-spin" /> : <Rocket className="h-4 w-4" />}
            Launch
          </Button>
          <Button onClick={onRefresh} variant="secondary" size="icon" aria-label="Refresh">
            <RefreshCw className="h-4 w-4" />
          </Button>
          {snap.restorable[Number(agentIdx)] && (
            <Button onClick={onRestore} variant="secondary" size="icon" aria-label="Restore">
              <Undo2 className="h-4 w-4" />
            </Button>
          )}
        </div>

        {/* Settings drawer toggle */}
        <button
          onClick={() => setDrawerOpen(true)}
          className="inline-flex items-center justify-center gap-1.5 text-xs text-muted-foreground transition-colors hover:text-foreground"
        >
          <SettingsIcon className="h-3.5 w-3.5" />
          Settings
        </button>
      </div>

      <SettingsDrawer
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        hostDraft={hostDraft}
        setHostDraft={setHostDraft}
        onCommitHost={commitHost}
        onTest={onTest}
        workingDir={workingDir}
        setWorkingDir={setWorkingDir}
        onPickDir={onPickDir}
        terminalKey={terminalKey}
        onTerminalChange={onTerminalChange}
        terminals={terminalOpts}
      />
    </div>
  );
}

function Section({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="mb-1.5 flex items-baseline justify-between">
        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">{label}</span>
        {hint && <span className="text-[10px] text-muted-foreground/70">{hint}</span>}
      </div>
      {children}
    </div>
  );
}

function SettingsDrawer(props: {
  open: boolean;
  onClose: () => void;
  hostDraft: string;
  setHostDraft: (v: string) => void;
  onCommitHost: (v: string) => void;
  onTest: () => void;
  workingDir: string;
  setWorkingDir: (v: string) => void;
  onPickDir: () => void;
  terminalKey: string;
  onTerminalChange: (v: string) => void;
  terminals: SelectOption[];
}) {
  if (!props.open) return null;
  return (
    <div className="fixed inset-0 z-40 flex items-end justify-center bg-black/40 sm:items-center">
      <div className="w-full max-w-md rounded-t-xl border border-border bg-card p-4 shadow-2xl sm:rounded-xl">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-sm font-semibold">Settings</h2>
          <button onClick={props.onClose} className="text-muted-foreground hover:text-foreground" aria-label="Close">
            <X className="h-4 w-4" />
          </button>
        </div>
        <div className="space-y-3">
          <div>
            <label className="mb-1 block text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              Ollama host
            </label>
            <div className="flex items-center gap-2">
              <Globe className="h-4 w-4 text-muted-foreground" />
              <Input
                value={props.hostDraft}
                onChange={(e) => props.setHostDraft(e.target.value)}
                onBlur={() => props.onCommitHost(props.hostDraft)}
                placeholder="http://localhost:11434"
              />
              <Button onClick={props.onTest} variant="secondary" size="sm">
                <Power className="h-3.5 w-3.5" />
                Test
              </Button>
            </div>
          </div>
          <div>
            <label className="mb-1 block text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              Working directory
            </label>
            <div className="flex items-center gap-2">
              <Input
                value={props.workingDir}
                onChange={(e) => props.setWorkingDir(e.target.value)}
                placeholder="(inherit launcher cwd)"
              />
              <Button onClick={props.onPickDir} variant="secondary" size="icon" aria-label="Pick directory">
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div>
            <label className="mb-1 block text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              Terminal (CLI agents)
            </label>
            <Select
              ariaLabel="Pick a terminal"
              value={props.terminalKey}
              onChange={props.onTerminalChange}
              options={props.terminals}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

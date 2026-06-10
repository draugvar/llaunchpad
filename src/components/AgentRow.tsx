import { Badge } from "@/ui/badge";
import type { Agent } from "@/lib/types";

const PALETTE = [
  "from-sky-500 to-indigo-500",
  "from-fuchsia-500 to-pink-500",
  "from-emerald-500 to-teal-500",
  "from-amber-500 to-orange-500",
  "from-violet-500 to-purple-500",
  "from-rose-500 to-red-500",
];

function initials(display: string): string {
  return display
    .split(/[\s_-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("");
}

export function AgentRow({ agent, isGui, running, installed, restorable, selected }: {
  agent: Agent;
  isGui: boolean;
  running: boolean;
  installed: boolean;
  restorable: boolean;
  selected: boolean;
}) {
  const idx = Math.abs(hash(agent.name)) % PALETTE.length;
  return (
    <div className="flex items-center gap-3 px-3 py-2">
      <div className={`grid h-8 w-8 shrink-0 place-items-center rounded-md bg-gradient-to-br ${PALETTE[idx]} text-xs font-bold text-white shadow-sm`}>
        {initials(agent.display || agent.name)}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="truncate text-sm font-medium">{agent.display}</span>
          {running && <Badge variant="success">running</Badge>}
          {!installed && <Badge variant="warning">not installed</Badge>}
          {isGui && <Badge variant="muted">gui</Badge>}
          {selected && <Badge variant="default">current</Badge>}
        </div>
      </div>
      {restorable && (
        <span className="text-[10px] uppercase tracking-wide text-muted-foreground">restorable</span>
      )}
    </div>
  );
}

function hash(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return h;
}

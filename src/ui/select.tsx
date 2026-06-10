import * as React from "react";
import { cn } from "@/lib/utils";
import { ChevronDown, Check } from "lucide-react";

export type SelectOption = { value: string; label: string; badge?: React.ReactNode; disabled?: boolean };

type SelectProps = {
  value: string;
  onChange: (v: string) => void;
  options: SelectOption[];
  placeholder?: string;
  className?: string;
  ariaLabel?: string;
};

export function Select({ value, onChange, options, placeholder = "Select…", className, ariaLabel }: SelectProps) {
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, []);

  const current = options.find((o) => o.value === value);

  return (
    <div ref={ref} className={cn("relative", className)}>
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel}
        onClick={() => setOpen((o) => !o)}
        className={cn(
          "flex h-9 w-full items-center justify-between gap-2 rounded-md border border-input bg-secondary/40 px-3 text-sm",
          "transition-colors hover:bg-secondary focus:outline-none focus:ring-2 focus:ring-ring/60",
        )}
      >
        <span className="truncate text-left">
          {current ? (
            <span className="inline-flex items-center gap-2">
              {current.badge}
              <span>{current.label}</span>
            </span>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
        </span>
        <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform", open && "rotate-180")} />
      </button>
      {open && (
        <div
          role="listbox"
          className="absolute z-50 mt-1 max-h-72 w-full overflow-auto rounded-md border border-border bg-popover p-1 shadow-lg"
        >
          {options.length === 0 && (
            <div className="px-3 py-2 text-sm text-muted-foreground">No options</div>
          )}
          {options.map((opt) => (
            <button
              key={opt.value}
              type="button"
              role="option"
              aria-selected={opt.value === value}
              disabled={opt.disabled}
              onClick={() => {
                if (opt.disabled) return;
                onChange(opt.value);
                setOpen(false);
              }}
              className={cn(
                "flex w-full items-center justify-between gap-2 rounded-sm px-3 py-2 text-sm",
                "hover:bg-accent focus:bg-accent focus:outline-none",
                opt.disabled && "opacity-50 cursor-not-allowed hover:bg-transparent",
                opt.value === value && "bg-accent/50",
              )}
            >
              <span className="inline-flex items-center gap-2">
                {opt.badge}
                <span>{opt.label}</span>
              </span>
              {opt.value === value && <Check className="h-3.5 w-3.5 text-primary" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

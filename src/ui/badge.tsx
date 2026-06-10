import * as React from "react";
import { cn } from "@/lib/utils";

type BadgeProps = React.HTMLAttributes<HTMLSpanElement> & {
  variant?: "default" | "secondary" | "success" | "warning" | "destructive" | "muted";
};

const styles: Record<NonNullable<BadgeProps["variant"]>, string> = {
  default: "bg-primary/15 text-primary border border-primary/20",
  secondary: "bg-secondary text-secondary-foreground border border-border",
  success: "bg-emerald-500/15 text-emerald-300 border border-emerald-500/30",
  warning: "bg-amber-500/15 text-amber-300 border border-amber-500/30",
  destructive: "bg-rose-500/15 text-rose-300 border border-rose-500/30",
  muted: "bg-muted text-muted-foreground border border-border",
};

export function Badge({ className, variant = "default", ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
        styles[variant],
        className,
      )}
      {...props}
    />
  );
}

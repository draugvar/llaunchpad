import React from "react";

type State = { error: Error | null; info: string };

export class ErrorBoundary extends React.Component<{ children: React.ReactNode }, State> {
  state: State = { error: null, info: "" };

  static getDerivedStateFromError(error: Error): State {
    return { error, info: error.stack ?? error.message };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // Persist the error to localStorage so we can read it from
    // the dev tools even after a refresh.
    try {
      localStorage.setItem(
        "llaunchpad.lastError",
        JSON.stringify({ message: error.message, stack: error.stack, info: info.componentStack }),
      );
    } catch {
      // localStorage might be unavailable
    }
    // Also surface it on the window for the Rust side to pick up.
    (window as any).__LLAUNCHPAD_LAST_ERROR__ = { message: error.message, stack: error.stack };
    console.error("[ErrorBoundary]", error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex h-full flex-col items-start gap-3 overflow-auto bg-background p-4 text-foreground">
          <h1 className="text-lg font-semibold text-destructive">Llaunchpad UI crashed</h1>
          <p className="text-sm text-muted-foreground">
            The React app failed to render. Copy the message below and paste it back to the
            developer.
          </p>
          <pre className="w-full max-w-2xl whitespace-pre-wrap rounded-md border border-rose-500/30 bg-rose-500/10 p-3 text-xs text-rose-200">
            {this.state.error.message}
          </pre>
          <pre className="w-full max-w-2xl whitespace-pre-wrap rounded-md border border-border bg-secondary/40 p-3 text-[10px] text-muted-foreground">
            {this.state.info}
          </pre>
        </div>
      );
    }
    return this.props.children;
  }
}

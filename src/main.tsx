import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { invoke } from "@tauri-apps/api/core";
import "./index.css";

function reportError(kind: string, message: string, stack?: string) {
  try {
    void invoke("log_frontend_error", { message: `${kind}: ${message}`, stack });
  } catch {
    console.error(`[${kind}]`, message, stack);
  }
}

// Surface a startup ping so we can tell whether the WebView
// even loaded our JS.
try {
  void invoke("log_frontend_error", {
    message: "startup: webview alive, JS executed main.tsx",
    stack: undefined,
  });
} catch (e) {
  console.error("[startup] invoke failed:", e);
}

window.addEventListener("error", (event) => {
  const msg = event.error?.message ?? event.message;
  const stack = event.error?.stack;
  reportError("window.error", msg, stack);
});

window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason;
  const msg = reason instanceof Error ? reason.message : String(reason);
  const stack = reason instanceof Error ? reason.stack : undefined;
  reportError("unhandledrejection", msg, stack);
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);

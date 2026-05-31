use crate::ollama::Agent;
use anyhow::{Context, Result};
use std::process::Command;
use sysinfo::System;

/// For GUI agents: (macOS application name used to quit, `.app` bundle dir name).
fn gui_app(agent: &str) -> Option<(&'static str, &'static str)> {
    match agent {
        "codex-app" => Some(("Codex", "Codex.app")),
        "vscode" => Some(("Visual Studio Code", "Visual Studio Code.app")),
        _ => None,
    }
}

/// Running state for many agents in one process-table scan.
/// Detects the GUI app by its main executable path (`<bundle>/Contents/MacOS/`),
/// so it does not false-positive on the `codex` CLI / app-server binary
/// (which lives under `Contents/Resources`). CLI agents are always `false`.
pub fn running_states(agents: &[Agent]) -> Vec<bool> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let exe_paths: Vec<String> = sys
        .processes()
        .values()
        .filter_map(|p| p.exe().map(|e| e.to_string_lossy().to_string()))
        .collect();
    agents
        .iter()
        .map(|a| match gui_app(&a.name) {
            Some((_, bundle)) => {
                let needle = format!("{bundle}/Contents/MacOS/");
                exe_paths.iter().any(|p| p.contains(&needle))
            }
            None => false,
        })
        .collect()
}

/// Best-effort: is the agent's app currently running?
pub fn agent_running(agent: &Agent) -> bool {
    running_states(std::slice::from_ref(agent))
        .first()
        .copied()
        .unwrap_or(false)
}

/// Quit a GUI app gracefully (AppleScript), with a killall fallback.
fn quit_gui(app_name: &str) {
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!("tell application \"{}\" to quit", app_name))
        .status();
    // give it a moment, then force any stragglers (executable name == app name)
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let _ = Command::new("killall").arg(app_name).status();
}

/// Remove the legacy top-level `profile = "..."` line that `ollama launch
/// codex-app` writes — current Codex rejects it. The top-level model/provider
/// keys it also writes are enough.
fn sanitize_codex_config() -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let path = std::path::Path::new(&home).join(".codex/config.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Ok(()); // nothing to clean
    };
    let cleaned: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with("profile ="))
        .collect::<Vec<_>>()
        .join("\n");
    if cleaned.len() != content.trim_end().len() {
        std::fs::write(&path, format!("{cleaned}\n")).context("failed to rewrite codex config")?;
    }
    Ok(())
}

/// Codex App: configure via `--config` (no auto-launch), strip the legacy
/// `profile =` line, then open the app ourselves. Avoids the
/// "legacy profile config no longer supported" error.
fn launch_codex_app(model: &str) -> Result<()> {
    if let Some((app_name, _)) = gui_app("codex-app") {
        // best-effort quit if already open (reuses bundle-path detection)
        let probe = Agent {
            name: "codex-app".to_string(),
            display: String::new(),
            is_gui: true,
        };
        if agent_running(&probe) {
            quit_gui(app_name);
        }
    }
    // configure only (writes config, does not launch)
    Command::new(crate::ollama::ollama_bin())
        .args(["launch", "codex-app", "--model", model, "--config", "-y"])
        .status()
        .context("failed to configure codex-app")?;
    sanitize_codex_config()?;
    Command::new("open")
        .args(["-a", "Codex"])
        .spawn()
        .context("failed to open Codex")?;
    Ok(())
}

/// Launch (or relaunch) an agent with the given model via `ollama launch`.
/// If the agent is already running it is closed first, then relaunched.
pub fn launch_agent(agent: &Agent, model: &str) -> Result<()> {
    if agent.name == "codex-app" {
        return launch_codex_app(model);
    }
    if agent.is_gui {
        if let Some((app_name, _)) = gui_app(&agent.name) {
            // close if already open, then relaunch
            if agent_running(agent) {
                quit_gui(app_name);
            }
        }
        // ollama launch configures the integration and opens the app
        Command::new(crate::ollama::ollama_bin())
            .args(["launch", &agent.name, "--model", model, "-y"])
            .spawn()
            .with_context(|| format!("failed to launch `{}`", agent.name))?;
    } else {
        // CLI agent: run inside Terminal.app (absolute path: GUI PATH is minimal)
        let cmd = format!(
            "{} launch {} --model {} -y",
            crate::ollama::ollama_bin(),
            agent.name,
            model
        );
        let script = format!(
            "tell application \"Terminal\"\n\
             activate\n\
             do script \"{}\"\n\
             end tell",
            cmd.replace('\\', "\\\\").replace('"', "\\\"")
        );
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .spawn()
            .with_context(|| format!("failed to open Terminal for `{}`", agent.name))?;
    }
    Ok(())
}

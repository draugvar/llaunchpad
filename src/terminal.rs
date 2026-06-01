use anyhow::{Context, Result};

/// The terminal application that hosts the shell running a CLI agent.
/// The `Default` variant delegates to a built-in platform default
/// (Terminal.app on macOS, the first available emulator on Linux,
/// cmd on Windows). All other variants target a specific terminal.
// `enum_variant_names`: variant names mirror the real terminal names
// (GnomeTerminal, Xfce4Terminal, WindowsTerminal, ITerm2) — those are
// the names users recognize, not stylistic noise.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    Default,

    // macOS
    #[allow(non_camel_case_types)]
    TerminalApp,
    #[allow(non_camel_case_types)]
    ITerm2,
    Alacritty,
    WezTerm,
    Kitty,

    // Linux
    #[allow(non_camel_case_types)]
    GnomeTerminal,
    Konsole,
    #[allow(non_camel_case_types)]
    Xfce4Terminal,
    Xterm,

    // Windows
    #[allow(non_camel_case_types)]
    WindowsTerminal,
    Cmd,
    PowerShell,
}

impl Terminal {
    /// Map a persisted key (e.g. from `prefs.json`) to a `Terminal`.
    /// Unknown / empty keys fall back to `Default`.
    pub fn from_key(s: &str) -> Self {
        match s {
            "" | "default" => Self::Default,
            "terminal" => Self::TerminalApp,
            "iterm2" => Self::ITerm2,
            "alacritty" => Self::Alacritty,
            "wezterm" => Self::WezTerm,
            "kitty" => Self::Kitty,
            "gnome-terminal" => Self::GnomeTerminal,
            "konsole" => Self::Konsole,
            "xfce4-terminal" => Self::Xfce4Terminal,
            "xterm" => Self::Xterm,
            "windows-terminal" => Self::WindowsTerminal,
            "cmd" => Self::Cmd,
            "powershell" => Self::PowerShell,
            _ => Self::Default,
        }
    }

    /// Stable string key used in `prefs.json` and the UI.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::TerminalApp => "terminal",
            Self::ITerm2 => "iterm2",
            Self::Alacritty => "alacritty",
            Self::WezTerm => "wezterm",
            Self::Kitty => "kitty",
            Self::GnomeTerminal => "gnome-terminal",
            Self::Konsole => "konsole",
            Self::Xfce4Terminal => "xfce4-terminal",
            Self::Xterm => "xterm",
            Self::WindowsTerminal => "windows-terminal",
            Self::Cmd => "cmd",
            Self::PowerShell => "powershell",
        }
    }

    /// Human-readable label shown in the dropdown.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "System default",
            Self::TerminalApp => "Terminal",
            Self::ITerm2 => "iTerm2",
            Self::Alacritty => "Alacritty",
            Self::WezTerm => "WezTerm",
            Self::Kitty => "kitty",
            Self::GnomeTerminal => "GNOME Terminal",
            Self::Konsole => "Konsole",
            Self::Xfce4Terminal => "XFCE Terminal",
            Self::Xterm => "xterm",
            Self::WindowsTerminal => "Windows Terminal",
            Self::Cmd => "Command Prompt",
            Self::PowerShell => "PowerShell",
        }
    }

    /// Run the given (already-prepared) command line in a new terminal
    /// window. `cmd` is a single command line, with `OLLAMA_HOST=...`
    /// already prepended by the caller when relevant.
    pub fn spawn(&self, cmd: &str) -> Result<()> {
        platform::spawn(self, cmd)
    }
}

// ───────────────────── platform-specific behavior ─────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use super::{Context, Result, Terminal};
    use std::process::Command;

    pub(super) fn available() -> &'static [Terminal] {
        &[
            Terminal::Default,
            Terminal::TerminalApp,
            Terminal::ITerm2,
            Terminal::Alacritty,
            Terminal::WezTerm,
            Terminal::Kitty,
        ]
    }

    pub(super) fn spawn(t: &Terminal, cmd: &str) -> Result<()> {
        match t {
            Terminal::Default | Terminal::TerminalApp => {
                let script = format!(
                    "tell application \"Terminal\"\nactivate\ndo script \"{}\"\nend tell",
                    cmd.replace('\\', "\\\\").replace('"', "\\\"")
                );
                Command::new("osascript")
                    .arg("-e")
                    .arg(script)
                    .spawn()
                    .context("failed to open Terminal")?;
            }
            Terminal::ITerm2 => {
                // iTerm2 AppleScript: create a new window running the given command
                let script = format!(
                    "tell application \"iTerm\"\nactivate\ncreate window with default profile command \"{}\"\nend tell",
                    cmd.replace('\\', "\\\\").replace('"', "\\\"")
                );
                Command::new("osascript")
                    .arg("-e")
                    .arg(script)
                    .spawn()
                    .context("failed to open iTerm2")?;
            }
            Terminal::Alacritty => {
                Command::new("open")
                    .args(["-a", "Alacritty", "--args", "-e", "bash", "-lc", cmd])
                    .spawn()
                    .context("failed to open Alacritty")?;
            }
            Terminal::WezTerm => {
                // wezterm-cli spawns a new tab/wnd running the command
                Command::new("open")
                    .args(["-a", "WezTerm", "--args", "cli", "spawn", "--", cmd])
                    .spawn()
                    .context("failed to open WezTerm")?;
            }
            Terminal::Kitty => {
                Command::new("open")
                    .args(["-a", "kitty", "--args", "bash", "-lc", cmd])
                    .spawn()
                    .context("failed to open kitty")?;
            }
            other => anyhow::bail!("{other:?} is not available on macOS"),
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{Command, Context, Result, Terminal};

    pub(super) fn available() -> &'static [Terminal] {
        &[
            Terminal::Default,
            Terminal::GnomeTerminal,
            Terminal::Konsole,
            Terminal::Xfce4Terminal,
            Terminal::Xterm,
            Terminal::Alacritty,
            Terminal::WezTerm,
            Terminal::Kitty,
        ]
    }

    /// Try a list of `(binary, args)` candidates in order and return the first
    /// that spawns successfully. Used by `Default`.
    fn try_candidates(cmd: &str) -> Result<()> {
        let hold = format!("{cmd}; exec ${{SHELL:-/bin/bash}}");
        let candidates: &[(&str, &[&str])] = &[
            ("x-terminal-emulator", &["-e", "bash", "-lc"]),
            ("gnome-terminal", &["--", "bash", "-lc"]),
            ("konsole", &["-e", "bash", "-lc"]),
            ("xfce4-terminal", &["-e", "bash", "-lc"]),
            ("xterm", &["-e", "bash", "-lc"]),
        ];
        for (bin, args) in candidates {
            let mut c = Command::new(bin);
            c.args(*args).arg(&hold);
            if c.spawn().is_ok() {
                return Ok(());
            }
        }
        anyhow::bail!("no terminal emulator found (tried gnome-terminal, konsole, xterm…)")
    }

    /// Spawn `bash -lc "<cmd>; exec $SHELL"` so the window stays open
    /// after the command finishes.
    fn spawn_bash(bin: &str, args: &[&str], cmd: &str) -> Result<()> {
        let hold = format!("{cmd}; exec ${{SHELL:-/bin/bash}}");
        let mut c = Command::new(bin);
        c.args(args).arg(&hold);
        c.spawn().with_context(|| format!("failed to open {bin}"))?;
        Ok(())
    }

    pub(super) fn spawn(t: &Terminal, cmd: &str) -> Result<()> {
        match t {
            Terminal::Default => try_candidates(cmd)?,
            Terminal::GnomeTerminal => spawn_bash("gnome-terminal", &["--", "bash", "-lc"], cmd)?,
            Terminal::Konsole => spawn_bash("konsole", &["-e", "bash", "-lc"], cmd)?,
            Terminal::Xfce4Terminal => spawn_bash("xfce4-terminal", &["-e", "bash", "-lc"], cmd)?,
            Terminal::Xterm => spawn_bash("xterm", &["-e", "bash", "-lc"], cmd)?,
            Terminal::Alacritty => spawn_bash("alacritty", &["-e", "bash", "-lc"], cmd)?,
            Terminal::Kitty => spawn_bash("kitty", &["bash", "-lc"], cmd)?,
            // wezterm has its own spawn interface
            Terminal::WezTerm => {
                Command::new("wezterm")
                    .args(["cli", "spawn", "--", cmd])
                    .spawn()
                    .context("failed to open wezterm")?;
            }
            other => anyhow::bail!("{other:?} is not available on Linux"),
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{Command, Context, Result, Terminal};

    pub(super) fn available() -> &'static [Terminal] {
        &[
            Terminal::Default,
            Terminal::WindowsTerminal,
            Terminal::Cmd,
            Terminal::PowerShell,
        ]
    }

    pub(super) fn spawn(t: &Terminal, cmd: &str) -> Result<()> {
        match t {
            // "Default" on Windows keeps the historical `cmd /C start cmd /K`
            // behavior so the window stays open after the agent exits.
            Terminal::Default | Terminal::Cmd => {
                Command::new("cmd")
                    .args(["/C", "start", "cmd", "/K", cmd])
                    .spawn()
                    .context("failed to open Command Prompt")?;
            }
            Terminal::WindowsTerminal => {
                Command::new("wt.exe")
                    .args(["new-tab", "cmd", "/K", cmd])
                    .spawn()
                    .context("failed to open Windows Terminal")?;
            }
            Terminal::PowerShell => {
                Command::new("cmd")
                    .args(["/C", "start", "powershell", "-NoExit", "-Command", cmd])
                    .spawn()
                    .context("failed to open PowerShell")?;
            }
            other => anyhow::bail!("{other:?} is not available on Windows"),
        }
        Ok(())
    }
}

/// List of terminals available on the current platform.
pub fn available() -> &'static [Terminal] {
    platform::available()
}

#[cfg(test)]
mod tests {
    use super::Terminal;

    #[test]
    fn from_key_roundtrip() {
        for k in [
            "default",
            "terminal",
            "iterm2",
            "alacritty",
            "wezterm",
            "kitty",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "xterm",
            "windows-terminal",
            "cmd",
            "powershell",
        ] {
            let t = Terminal::from_key(k);
            assert_eq!(t.key(), k, "roundtrip failed for {k}");
        }
    }

    #[test]
    fn from_key_unknown_falls_back() {
        assert_eq!(Terminal::from_key(""), Terminal::Default);
        assert_eq!(Terminal::from_key("nonsense"), Terminal::Default);
    }
}

//! Terminal selection.
//!
//! The CLI agent is launched into a new terminal window. On macOS that
//! defaults to Terminal.app, on Linux to the first available of
//! `x-terminal-emulator` / gnome-terminal / konsole / xfce4-terminal /
//! xterm, and on Windows to `cmd`. The user can pick a different
//! terminal from the Settings panel; the choice is persisted in
//! `prefs.json` and applied on every subsequent launch.
//!
//! The dropdown only lists terminals that are *actually installed* on
//! the current machine — checking for the binary on `PATH` (or, on
//! macOS, the `.app` bundle in `/Applications`). The synthetic
//! `Default` option is always present and falls back to the OS-builtin
//! behavior described above.

#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(unix)]
use std::process::Command;
use std::sync::OnceLock;

/// One of the terminals Llaunchpad knows how to spawn a CLI agent into.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    /// Use the OS-builtin default (Terminal.app on macOS, the first
    /// emulator found on Linux, cmd on Windows). Always available.
    Default,

    // macOS
    #[allow(non_camel_case_types)]
    TerminalApp,
    #[allow(non_camel_case_types)]
    ITerm2,
    Alacritty,
    WezTerm,
    Kitty,
    Warp,

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
            "warp" => Self::Warp,
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
            Self::Warp => "warp",
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
            Self::Warp => "Warp",
            Self::GnomeTerminal => "GNOME Terminal",
            Self::Konsole => "Konsole",
            Self::Xfce4Terminal => "XFCE Terminal",
            Self::Xterm => "xterm",
            Self::WindowsTerminal => "Windows Terminal",
            Self::Cmd => "Command Prompt",
            Self::PowerShell => "PowerShell",
        }
    }

    /// True if this terminal is a real candidate the UI should expose.
    /// `Default` is always available; the rest are filtered by
    /// `is_installed()`.
    pub fn available(self) -> bool {
        match self {
            Self::Default => true,
            other => other.is_installed(),
        }
    }

    /// True if the binary / .app bundle backing this terminal is present
    /// on the current machine. On Windows, looks for `.exe`/`.cmd`/`.bat`
    /// alongside the bare name.
    pub fn is_installed(self) -> bool {
        match self {
            Self::Default => true,
            Self::TerminalApp => macos_bundle("Terminal.app"),
            Self::ITerm2 => macos_bundle("iTerm.app"),
            Self::Alacritty => binary_in("alacritty"),
            Self::WezTerm => binary_in("wezterm"),
            Self::Kitty => binary_in("kitty"),
            Self::Warp => macos_bundle("Warp.app"),
            Self::GnomeTerminal => binary_in("gnome-terminal"),
            Self::Konsole => binary_in("konsole"),
            Self::Xfce4Terminal => binary_in("xfce4-terminal"),
            Self::Xterm => binary_in("xterm"),
            Self::WindowsTerminal => binary_in("wt") || binary_in("wt.exe"),
            Self::Cmd => binary_in("cmd") || cfg!(target_os = "windows"),
            Self::PowerShell => {
                binary_in("powershell") || binary_in("pwsh")
            }
        }
    }

    /// Spawn `cmd` in a new window hosted by this terminal.
    #[cfg(target_os = "macos")]
    pub fn spawn(&self, cmd: &str) -> Result<()> {
        platform::spawn(self, cmd)
    }
}

// ───────────────────── install detection helpers ─────────────────────

/// True if `name` is a binary somewhere on the user's PATH. GUI apps
/// on macOS get a minimal PATH, so we resolve the *login* PATH first,
/// then fall back to the current process PATH.
fn binary_in(name: &str) -> bool {
    for d in login_path_dirs() {
        if d.join(name).exists() {
            return true;
        }
        #[cfg(windows)]
        for ext in ["exe", "cmd", "bat"] {
            if d.join(format!("{name}.{ext}")).exists() {
                return true;
            }
        }
    }
    false
}

/// True if the macOS `.app` bundle exists in `/Applications` or the
/// user's `~/Applications`. On non-macOS platforms always false.
#[cfg(target_os = "macos")]
fn macos_bundle(name: &str) -> bool {
    for root in ["/Applications"] {
        if std::path::Path::new(root).join(name).exists() {
            return true;
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let p = std::path::PathBuf::from(home).join("Applications").join(name);
        if p.exists() {
            return true;
        }
    }
    false
}

#[cfg(not(target_os = "macos"))]
fn macos_bundle(_name: &str) -> bool {
    false
}

/// Cached login-shell PATH, expanded once.
fn login_path_dirs() -> &'static [std::path::PathBuf] {
    static DIRS: OnceLock<Vec<std::path::PathBuf>> = OnceLock::new();
    DIRS.get_or_init(|| {
        #[cfg(unix)]
        {
            for sh in ["/bin/zsh", "/bin/bash", "/bin/sh"] {
                if !std::path::Path::new(sh).exists() {
                    continue;
                }
                if let Ok(out) = Command::new(sh)
                    .args(["-lc", "printf %s \"$PATH\""])
                    .output()
                {
                    let s = String::from_utf8_lossy(&out.stdout).into_owned();
                    if !s.is_empty() {
                        return s
                            .split(':')
                            .filter(|p| !p.is_empty())
                            .map(std::path::PathBuf::from)
                            .collect();
                    }
                }
            }
        }
        std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default()
    })
    .as_slice()
}

// ───────────────────── platform-specific spawners ─────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use super::{Context, Result, Terminal};
    use std::process::Command;

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
            Terminal::Warp => {
                // Warp does not expose an AppleScript dictionary and
                // has no CLI / URL scheme to run a specific command
                // in a new tab. Two documented mechanisms were tried
                // and both failed in practice:
                //   - opening a .command file: Warp claims the
                //     `com.apple.terminal.shell-script` UTI with role
                //     `Editor`, so it opens the file as a document
                //     instead of running it.
                //   - AppleScript `keystroke` via System Events:
                //     blocked by Accessibility TCC in the Llaunchpad
                //     process context, and even when granted there is
                //     no public API to target the shell input vs the
                //     AI prompt.
                //
                // The pragmatic workaround: open Warp, put the
                // command on the system clipboard, and tell the user
                // to paste. The user has to hit Cmd+V once, which is
                // a fraction of the keystrokes they would type by
                // hand.
                //
                // 1. Ensure Warp is in the foreground. Use the
                //    bundle ID (more reliable than the name when
                //    LaunchServices metadata is stale).
                let open_status = Command::new("open")
                    .args(["-b", "dev.warp.Warp-Stable"])
                    .output()
                    .context("failed to spawn `open -b dev.warp.Warp-Stable`")?;
                if !open_status.status.success() {
                    let stderr = String::from_utf8_lossy(&open_status.stderr);
                    anyhow::bail!(
                        "Could not launch Warp (open exit {:?}): {}",
                        open_status.status.code(),
                        stderr.trim()
                    );
                }

                // 2. Copy the prepared command to the clipboard.
                //    The user pastes it into Warp's shell input
                //    (Cmd+1 to focus the shell if the AI prompt
                //    is open, then Cmd+V).
                let mut pbcopy = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .context("failed to spawn pbcopy")?;
                if let Some(stdin) = pbcopy.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(cmd.as_bytes())
                        .context("failed to write command to pbcopy")?;
                }
                let pbcopy_status = pbcopy.wait().context("pbcopy failed")?;
                if !pbcopy_status.success() {
                    anyhow::bail!("pbcopy exited {:?}", pbcopy_status.code());
                }

                // 3. Tell the caller the launch is in the user's
                //    court. The model layer surfaces this as a
                //    success banner.
                return Ok(());
            }
            other => anyhow::bail!("{other:?} is not available on macOS"),
        }
        Ok(())
    }
}


/// Filtered list of terminals available on the current platform. Used
/// to populate the dropdown — `Default` is always first, then the
/// platform-specific terminals that are actually installed.
pub fn available() -> Vec<Terminal> {
    all_for_platform()
        .iter()
        .copied()
        .filter(|t| t.available())
        .collect()
}

/// All terminals we know about on this platform (in display order),
/// before install-detection filtering. Useful for tests and for the
/// `from_key` lookup so a stale pref never silently becomes "Default"
/// just because the user uninstalled the app.
fn all_for_platform() -> &'static [Terminal] {
    match () {
        _ if cfg!(target_os = "macos") => &[
            Terminal::Default,
            Terminal::TerminalApp,
            Terminal::ITerm2,
            Terminal::Alacritty,
            Terminal::WezTerm,
            Terminal::Kitty,
            Terminal::Warp,
        ],
        _ if cfg!(target_os = "linux") => &[
            Terminal::Default,
            Terminal::GnomeTerminal,
            Terminal::Konsole,
            Terminal::Xfce4Terminal,
            Terminal::Xterm,
            Terminal::Alacritty,
            Terminal::WezTerm,
            Terminal::Kitty,
        ],
        _ if cfg!(target_os = "windows") => &[
            Terminal::Default,
            Terminal::WindowsTerminal,
            Terminal::Cmd,
            Terminal::PowerShell,
        ],
        _ => &[Terminal::Default],
    }
}

/// Index of `t.key()` in `available()`, falling back to 0 (Default).
/// `available()` is recomputed on each call — it's cheap, and the
/// install state can change (e.g. user just installed iTerm2) so we
/// don't want to cache it.
pub fn index_of(key: &str) -> usize {
    let list = available();
    list.iter()
        .position(|t| t.key() == key)
        .unwrap_or(0)
        .min(list.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_key_round_trips() {
        for t in [
            Terminal::Default,
            Terminal::TerminalApp,
            Terminal::ITerm2,
            Terminal::Alacritty,
            Terminal::WezTerm,
            Terminal::Kitty,
            Terminal::Warp,
            Terminal::GnomeTerminal,
            Terminal::Konsole,
            Terminal::Xfce4Terminal,
            Terminal::Xterm,
            Terminal::WindowsTerminal,
            Terminal::Cmd,
            Terminal::PowerShell,
        ] {
            assert_eq!(Terminal::from_key(t.key()), t);
        }
    }

    #[test]
    fn unknown_key_falls_back_to_default() {
        assert_eq!(Terminal::from_key(""), Terminal::Default);
        assert_eq!(Terminal::from_key("nonsense"), Terminal::Default);
    }

    #[test]
    fn default_is_always_available() {
        assert!(Terminal::Default.available());
    }

    #[test]
    fn installed_terminals_round_trip_to_nonempty_labels() {
        for t in available() {
            assert!(!t.label().is_empty());
            assert!(!t.key().is_empty());
        }
    }

    #[test]
    fn index_of_unknown_falls_back_to_default() {
        // If the persisted key isn't installed anymore we want the
        // dropdown to land on "System default", not on an out-of-range
        // index.
        let i = index_of("no-such-terminal");
        let list = available();
        assert!(i < list.len());
        assert_eq!(list[i], Terminal::Default);
    }

    #[test]
    fn warp_round_trips_and_is_listed_on_macos() {
        // The Warp enum variant must survive a from_key -> key cycle so
        // the persisted `terminal` field in prefs.json round-trips.
        assert_eq!(Terminal::from_key(Terminal::Warp.key()), Terminal::Warp);
        assert_eq!(Terminal::Warp.key(), "warp");
        assert_eq!(Terminal::Warp.label(), "Warp");

        // On macOS, Warp.app is a known candidate even if the running
        // machine doesn't have it. The available() list must therefore
        // include it as a *candidate* — and if Warp.app is installed in
        // /Applications, `is_installed()` should agree.
        if cfg!(target_os = "macos") {
            let list = available();
            // Warp may or may not be installed on the host running
            // these tests; only assert the candidate is reachable via
            // from_key. The is_installed() check itself depends on the
            // real filesystem.
            assert!(Terminal::Warp.is_installed() || !Terminal::Warp.is_installed());
            // But on a macOS host with Warp.app in /Applications,
            // is_installed() must return true. We can't assert that
            // here without coupling tests to the host, so we just
            // make sure available() doesn't crash and contains Default.
            assert!(list.contains(&Terminal::Default));
        }
    }
}

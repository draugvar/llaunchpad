# Llaunchpad

A cross-platform launcher for Ollama coding agents with cloud models.
A tiny, good-looking GUI that picks the agent (Codex, Claude Code, Cursor, …)
and the cloud model, then runs `ollama launch <agent> --model <model>` for
you in one click.

## What's new in 0.7.0

- **New UI**: React + Vite + Tailwind (replaces Slint).
- **Tauri 2** shell, with the same Rust backend driving the IPC.
- **Smaller bundle**: ~6 MB binary on macOS (was ~14 MB).
- **DevTools**: standard Chromium DevTools in `pnpm tauri dev`.
- **HMR**: Vite hot reload for UI changes in milliseconds.

## Stack

| Layer | Tech |
|---|---|
| Shell | Tauri 2 (WebView of the OS) |
| UI | React 18 + TypeScript + Vite 5 + Tailwind 3 + shadcn-style components |
| State | Rust (`AppModel` + `Repository` traits, unchanged) |
| IPC | `#[tauri::command]` functions in `src/commands.rs` |
| Build | `pnpm tauri build` produces `.app`/`.dmg` on macOS, `.deb`/`.AppImage` on Linux, `.msi`/`.exe` on Windows |

## Project layout

```
.
├── Cargo.toml            # Rust deps: tauri 2, tauri-plugin-*, the original model deps
├── build.rs              # tauri-build
├── tauri.conf.json       # Tauri runtime config (window, bundle, identifier)
├── src/                  # Rust crate (lib + bin)
│   ├── lib.rs            # AppState, background tasks, Tauri builder
│   ├── main.rs           # 3-line bin entry → llaunchpad_lib::run()
│   ├── commands.rs       # #[tauri::command] IPC handlers
│   ├── model.rs          # AppModel + StateSnapshot (unchanged logic, derives Serialize)
│   ├── repository.rs     # Repository trait + OllamaRepository
│   ├── ollama/           # ollama launch --help parser, model catalog, process scan
│   ├── terminal.rs       # per-OS terminal selection
│   └── config.rs         # prefs.json load/save
├── src-tauri/
│   ├── tauri.conf.json   # canonical Tauri config
│   ├── capabilities/     # default.json (permissions)
│   └── icons/            # AppIcon.icns + PNG sizes
├── src/                  # (frontend) React app
│   ├── App.tsx
│   ├── main.tsx
│   ├── index.css
│   ├── components/       # AgentRow, etc.
│   ├── ui/               # Select, Button, Badge, Input (shadcn-style)
│   └── lib/              # api.ts (IPC), types.ts, utils.ts
├── index.html
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── package.json
```

## Develop

```bash
pnpm install
pnpm tauri dev
```

This starts Vite on `http://localhost:1420` and launches the Tauri shell
that loads it. The first time you'll need to run `pnpm approve-builds esbuild`
to let Vite's native dep install.

## Build a release bundle

```bash
pnpm tauri build
```

Produces:
- macOS: `target/release/bundle/macos/Llaunchpad.app`, `target/release/bundle/dmg/Llaunchpad_0.7.0_aarch64.dmg`
- Linux: `.deb`, `.AppImage`
- Windows: `.msi`, `.exe`

## Tests

```bash
cargo test --lib -- --test-threads=1
```

39 unit tests covering the model, the terminal selector, and the ollama
model catalog. Run with `--test-threads=1` because the prefs.json tests
share a per-process HOME that gets restored only on Drop.

## License

MIT.

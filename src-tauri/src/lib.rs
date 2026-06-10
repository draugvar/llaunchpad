//! Llaunchpad library crate.
//!
//! This file is the entry point the Tauri runtime calls into (`run()`).
//! It wires the same Model + Repository the legacy Slint UI used, then
//! exposes the Model's intents as `#[tauri::command]` functions so the
//! React frontend can drive them via IPC.

mod config;
mod model;
mod ollama;
mod repository;
mod terminal;

#[cfg(test)]
mod test_util;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub mod commands;

use crate::model::AppModel;
use crate::repository::{OllamaRepository, Repository};

/// Shared application state handed to every Tauri command. `AppModel`
/// is `Clone`-cheap (it wraps `Arc` internals) and is `Send + Sync`
/// so we can hand the same instance to every command invocation.
pub struct AppState {
    pub model: AppModel,
}

impl AppState {
    pub fn new() -> Self {
        let repo: Arc<dyn Repository> = Arc::new(OllamaRepository);
        let prefs = config::load();
        let model = AppModel::new(repo, prefs);
        Self { model }
    }
}

/// Path to the panic log. Tauri release builds set `panic = "abort"`,
/// which means a panic prints a useless backtrace and aborts. We
/// install a custom hook that writes the panic message (and the
/// backtrace if `RUST_BACKTRACE=1`) here so the user can see what
/// actually went wrong.
fn panic_log_path() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push("llaunchpad-panic.log");
    p
}

fn install_panic_hook() {
    let path = panic_log_path();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => (*s).to_string(),
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.clone(),
                None => "<non-string panic payload>".to_string(),
            },
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let body = format!(
            "[llaunchpad] PANIC at {location}\n  message: {msg}\n  backtrace:\n{bt}\n",
        );
        let _ = std::fs::write(&path, &body);
        eprintln!("{body}");
    }));
}

/// Spawn the model's background poller + mirror task. Called once on
/// `setup` so the snapshot is fresh by the time the first command
/// fires.
fn spawn_background_tasks(app: AppHandle, model: AppModel) {
    let handle = tokio::runtime::Handle::current();
    let me = model.clone();
    handle.spawn(async move {
        me.refresh().await;
    });

    // Mirror task: subscribe to model state changes and emit them to
    // the frontend over the `state-updated` Tauri event.
    let mut rx = model.subscribe();
    let app_for_mirror = app.clone();
    handle.spawn(async move {
        // Drop the first value: it's the initial state, the frontend
        // will request it on mount.
        let _ = rx.borrow().clone();
        while rx.changed().await.is_ok() {
            let snap = rx.borrow().clone();
            if let Err(e) = app_for_mirror.emit("state-updated", &snap) {
                eprintln!("[llaunchpad] state emit failed: {e}");
            }
        }
    });

    // 5s poller
    let me = model.clone();
    handle.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        interval.tick().await; // skip the immediate first tick
        loop {
            interval.tick().await;
            me.refresh().await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_hook();

    let state = AppState::new();
    let model_for_setup = state.model.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .manage(state)
        .setup(move |app| {
            spawn_background_tasks(app.handle().clone(), model_for_setup.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::list_terminals,
            commands::set_ollama_host,
            commands::set_working_dir,
            commands::set_terminal,
            commands::test_connection,
            commands::refresh,
            commands::launch,
            commands::restore,
            commands::is_agent_restorable,
            commands::pick_directory,
            commands::dismiss_status,
            commands::toggle_settings,
            commands::record_selection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

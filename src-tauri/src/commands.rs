//! Tauri IPC commands.
//!
//! Each public function here is exposed to the React frontend via
//! `invoke('snake_case_name', { ... })`. The frontend never touches
//! the Model directly — it goes through this thin, typed adapter.

use crate::model::Status;
use crate::ollama::Agent;
use crate::terminal::Terminal;
use crate::AppState;
use serde::Serialize;
use tauri::State;

/// Lightweight DTO mirroring the `Terminal` enum so the frontend
/// doesn't need to know our Rust types. Stable across the IPC
/// boundary.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalDto {
    pub key: String,
    pub label: String,
    pub available: bool,
}

/// Snapshot DTO sent to the frontend. We re-export the existing
/// `StateSnapshot` and `WorldSnapshot` types (which already derive
/// `Serialize`) and add a few derived fields the UI likes.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    #[serde(flatten)]
    pub snapshot: crate::model::StateSnapshot,
    /// Combined model list (local + cloud) with a `local` flag the
    /// frontend can use to badge the rows.
    pub models: Vec<ModelRowDto>,
    /// Index → `is_restorable` so the UI can enable the Restore button.
    pub restorable: Vec<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRowDto {
    pub name: String,
    pub local: bool,
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> SnapshotDto {
    let snap = state.model.snapshot();
    let world = snap.world.clone();

    let mut models: Vec<ModelRowDto> = Vec::new();
    if let Some(w) = &world {
        for name in &w.cloud_models {
            models.push(ModelRowDto {
                name: name.clone(),
                local: false,
            });
        }
    }
    for name in &snap.local_models {
        if !models.iter().any(|m| &m.name == name) {
            models.push(ModelRowDto {
                name: name.clone(),
                local: true,
            });
        }
    }

    let restorable = if let Some(w) = &world {
        w.agents
            .iter()
            .map(|a| state.model.is_agent_restorable(&a.name))
            .collect()
    } else {
        Vec::new()
    };

    SnapshotDto {
        snapshot: snap,
        models,
        restorable,
    }
}

#[tauri::command]
pub fn list_terminals() -> Vec<TerminalDto> {
    crate::terminal::available()
        .into_iter()
        .map(|t| TerminalDto {
            key: t.key().to_string(),
            label: t.label().to_string(),
            available: t.is_installed(),
        })
        .collect()
}

#[tauri::command]
pub fn set_ollama_host(state: State<'_, AppState>, url: String) {
    state.model.set_ollama_host(url);
}

#[tauri::command]
pub fn set_working_dir(state: State<'_, AppState>, dir: String) {
    state.model.set_working_dir(dir);
}

#[tauri::command]
pub fn set_terminal(state: State<'_, AppState>, key: String) {
    state.model.set_terminal(key);
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>, url: String) -> Result<SnapshotDto, String> {
    state.model.test_connection(url).await;
    Ok(get_snapshot(state))
}

#[tauri::command]
pub async fn refresh(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    state.model.refresh().await;
    Ok(get_snapshot(state))
}

#[tauri::command]
pub fn record_selection(
    state: State<'_, AppState>,
    agent: Option<String>,
    model: Option<String>,
) {
    state.model.record_selection(agent, model);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn launch(
    state: State<'_, AppState>,
    agent_idx: i32,
    model: String,
    ollama_host: Option<String>,
    working_dir: Option<String>,
    terminal_key: Option<String>,
) -> Result<LaunchResult, String> {
    let agent: Agent = state
        .model
        .agent_by_index(agent_idx)
        .ok_or_else(|| "invalid agent index".to_string())?;
    let terminal = Terminal::from_key(terminal_key.as_deref().unwrap_or(""));
    let dir = working_dir.as_deref();
    state
        .model
        .record_launch(agent.name.clone(), model.clone());
    let res = state
        .model
        .launch(agent, model, ollama_host, dir, terminal)
        .await;
    match res {
        Ok(()) => Ok(LaunchResult {
            ok: true,
            message: "✓ launched".into(),
        }),
        Err(e) => {
            let msg = format!("✗ {e}");
            state.model.set_status(Status {
                message: msg.clone(),
                kind: 2,
            });
            Ok(LaunchResult { ok: false, message: msg })
        }
    }
}

#[tauri::command]
pub async fn restore(
    state: State<'_, AppState>,
    agent_idx: i32,
) -> Result<LaunchResult, String> {
    let agent: Agent = state
        .model
        .agent_by_index(agent_idx)
        .ok_or_else(|| "invalid agent index".to_string())?;
    match state.model.restore(agent.name).await {
        Ok(()) => Ok(LaunchResult {
            ok: true,
            message: "✓ restored".into(),
        }),
        Err(e) => Ok(LaunchResult {
            ok: false,
            message: format!("✗ {e}"),
        }),
    }
}

#[tauri::command]
pub fn is_agent_restorable(state: State<'_, AppState>, agent_idx: i32) -> bool {
    if let Some(a) = state.model.agent_by_index(agent_idx) {
        state.model.is_agent_restorable(&a.name)
    } else {
        false
    }
}

#[tauri::command]
pub fn pick_directory(start_dir: Option<String>) -> Option<String> {
    crate::ollama::pick_directory(start_dir.as_deref())
}

#[tauri::command]
pub fn dismiss_status(state: State<'_, AppState>) {
    state.model.dismiss_status();
}

#[tauri::command]
pub fn toggle_settings(state: State<'_, AppState>) {
    state.model.toggle_settings();
}

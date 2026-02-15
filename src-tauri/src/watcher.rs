use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use tauri::{AppHandle, Emitter, Manager};

// ─── State ─────────────────────────────────────────────────────

pub struct WatcherState {
    watcher: Mutex<Option<RecommendedWatcher>>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            watcher: Mutex::new(None),
        }
    }
}

impl WatcherState {
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── Start / Stop ──────────────────────────────────────────────

pub fn start_watching(app: &AppHandle, project_path: &str) -> Result<(), String> {
    let state = app.state::<WatcherState>();
    let mut guard = state.watcher.lock().map_err(|e| e.to_string())?;

    // Stop existing watcher if any
    *guard = None;

    let app_handle = app.clone();
    let path = project_path.to_string();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_)
                    | EventKind::Remove(_)
                    | EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
                        let _ = app_handle.emit("fs-changed", &path);
                    }
                    _ => {}
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    watcher
        .watch(Path::new(project_path), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {}", e))?;

    *guard = Some(watcher);
    Ok(())
}

pub fn stop_watching(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<WatcherState>();
    let mut guard = state.watcher.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;
use ps_core::{open_default_store, CaptureSnapshot};
use serde::Serialize;
use sysinfo::{Pid, System};
use tauri::{AppHandle, Emitter};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize)]
pub struct AgentCrashEvent {
    pub app_name: String,
    pub bundle_id: String,
    pub preview: String,
    pub draft_id: Option<i64>,
}

#[derive(Clone)]
struct TrackedAgent {
    app_name: String,
    bundle_id: String,
    slot_key: String,
    preview: String,
}

pub struct AgentWatchdog {
    tracked: Arc<Mutex<HashMap<u32, TrackedAgent>>>,
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl Default for AgentWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentWatchdog {
    pub fn new() -> Self {
        Self {
            tracked: Arc::new(Mutex::new(HashMap::new())),
            stop: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
        }
    }

    pub fn track_snapshot(&self, snapshot: &CaptureSnapshot) {
        if !is_watched_agent(&snapshot.slot.context.bundle_id, &snapshot.slot.context.app_name) {
            return;
        }
        let content = snapshot.content.trim();
        if content.chars().count() < 8 {
            return;
        }
        let pid = snapshot.slot.context.pid;
        let preview: String = content.chars().take(80).collect();
        self.tracked.lock().insert(
            pid,
            TrackedAgent {
                app_name: snapshot.slot.context.app_name.clone(),
                bundle_id: snapshot.slot.context.bundle_id.clone(),
                slot_key: snapshot.slot.key.clone(),
                preview,
            },
        );
    }

    pub fn start(&self, app: AppHandle) {
        if self.handle.lock().is_some() {
            return;
        }
        self.stop.store(false, Ordering::SeqCst);
        let tracked = Arc::clone(&self.tracked);
        let stop = Arc::clone(&self.stop);
        let handle = thread::Builder::new()
            .name("ps-agent-watchdog".into())
            .spawn(move || watchdog_loop(app, tracked, stop))
            .expect("spawn watchdog thread");
        *self.handle.lock() = Some(handle);
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.lock().take() {
            let _ = handle.join();
        }
        self.tracked.lock().clear();
    }
}

impl Drop for AgentWatchdog {
    fn drop(&mut self) {
        self.stop();
    }
}

fn watchdog_loop(app: AppHandle, tracked: Arc<Mutex<HashMap<u32, TrackedAgent>>>, stop: Arc<AtomicBool>) {
    let mut system = System::new();
    while !stop.load(Ordering::SeqCst) {
        let config = ps_core::AppConfig::load().unwrap_or_default();
        if !config.crash_toast_enabled {
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut exited: Vec<(u32, TrackedAgent)> = Vec::new();
        {
            let guard = tracked.lock();
            for (pid, agent) in guard.iter() {
                if system.process(Pid::from_u32(*pid)).is_none() {
                    exited.push((*pid, agent.clone()));
                }
            }
        }

        for (pid, agent) in exited {
            tracked.lock().remove(&pid);
            let draft_id = latest_draft_id_for_slot(&agent.slot_key);
            let event = AgentCrashEvent {
                app_name: agent.app_name.clone(),
                bundle_id: agent.bundle_id.clone(),
                preview: agent.preview.clone(),
                draft_id,
            };
            info!(app = %event.app_name, pid, "detected agent process exit");
            let _ = app.emit("agent-crash", &event);
        }

        thread::sleep(Duration::from_secs(2));
    }
    debug!("agent watchdog stopped");
}

fn latest_draft_id_for_slot(slot_key: &str) -> Option<i64> {
    let store = open_default_store().ok()?;
    let drafts = store.list_recent(20).ok()?;
    drafts
        .into_iter()
        .find(|d| d.slot_key == slot_key)
        .map(|d| d.id)
}

pub fn is_watched_agent(bundle_id: &str, app_name: &str) -> bool {
    let bundle = bundle_id.to_ascii_lowercase();
    let name = app_name.to_ascii_lowercase();
    [
        "cursor",
        "codex",
        "todesktop",
        "anthropic",
        "claude",
        "openai",
        "copilot",
        "windsurf",
        "trae",
        "aider",
    ]
    .iter()
    .any(|needle| bundle.contains(needle) || name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cursor_bundle() {
        assert!(is_watched_agent("com.todesktop.230313mzl4w4u92", "Cursor"));
    }

    #[test]
    fn ignores_unrelated_apps() {
        assert!(!is_watched_agent("com.apple.TextEdit", "TextEdit"));
    }
}

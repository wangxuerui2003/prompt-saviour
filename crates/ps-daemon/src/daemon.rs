use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use ps_core::{
    open_default_store, AppConfig, CaptureSnapshot, DebounceEngine, MergeEngine,
};
use serde::Serialize;
use tracing::{error, info};

use crate::platform_hub::{self, Hub};

#[derive(Debug, Clone, Serialize)]
pub struct CurrentPromptUpdate {
    pub snapshot: Option<CaptureSnapshot>,
    pub char_count: usize,
    pub line_count: usize,
    pub source: Option<String>,
    pub confidence: String,
    pub persisted: bool,
    pub last_saved_at: Option<DateTime<Utc>>,
    pub paused: bool,
}

impl CurrentPromptUpdate {
    fn empty(paused: bool) -> Self {
        Self {
            snapshot: None,
            char_count: 0,
            line_count: 0,
            source: None,
            confidence: "none".into(),
            persisted: false,
            last_saved_at: None,
            paused,
        }
    }

    fn from_snapshot(
        snapshot: CaptureSnapshot,
        persisted: bool,
        last_saved_at: Option<DateTime<Utc>>,
        paused: bool,
    ) -> Self {
        let char_count = snapshot.content.chars().count();
        let line_count = snapshot.content.lines().count();
        let confidence = confidence_for(&snapshot);
        let source = Some(snapshot.source.as_str().to_string());
        Self {
            snapshot: Some(snapshot),
            char_count,
            line_count,
            source,
            confidence,
            persisted,
            last_saved_at,
            paused,
        }
    }
}

fn confidence_for(snapshot: &CaptureSnapshot) -> String {
    match snapshot.source {
        ps_core::CaptureSource::Accessibility => "high".into(),
        ps_core::CaptureSource::Keystroke => "medium".into(),
        ps_core::CaptureSource::Clipboard => "low".into(),
        ps_core::CaptureSource::Merged => "medium".into(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub paused: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub db_path: String,
    pub data_dir: String,
    pub config_path: String,
    pub draft_count: i64,
    pub db_size_bytes: u64,
    pub platform: String,
}

pub type UpdateCallback = Arc<dyn Fn(CurrentPromptUpdate) + Send + Sync>;

pub struct CaptureDaemon {
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    started_at: Arc<Mutex<Option<DateTime<Utc>>>>,
    last_saved_at: Arc<Mutex<Option<DateTime<Utc>>>>,
    current: Arc<Mutex<Option<CaptureSnapshot>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    on_update: Mutex<Option<UpdateCallback>>,
    hub: Mutex<Option<Hub>>,
}

impl Default for CaptureDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureDaemon {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            started_at: Arc::new(Mutex::new(None)),
            last_saved_at: Arc::new(Mutex::new(None)),
            current: Arc::new(Mutex::new(None)),
            handle: Mutex::new(None),
            on_update: Mutex::new(None),
            hub: Mutex::new(None),
        }
    }

    pub fn set_update_callback(&self, callback: UpdateCallback) {
        *self.on_update.lock() = Some(callback);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn current_prompt(&self) -> CurrentPromptUpdate {
        let paused = self.paused.load(Ordering::SeqCst);
        let last_saved_at = *self.last_saved_at.lock();
        match self.current.lock().clone() {
            Some(snapshot) => {
                CurrentPromptUpdate::from_snapshot(snapshot, false, last_saved_at, paused)
            }
            None => CurrentPromptUpdate::empty(paused),
        }
    }

    pub fn status(&self) -> Result<DaemonStatus> {
        let store = open_default_store()?;
        let data_dir = ps_core::data_dir()?;
        Ok(DaemonStatus {
            running: self.running.load(Ordering::SeqCst),
            paused: self.paused.load(Ordering::SeqCst),
            started_at: *self.started_at.lock(),
            db_path: store.db_path().display().to_string(),
            data_dir: data_dir.display().to_string(),
            config_path: ps_core::config::default_config_path()?.display().to_string(),
            draft_count: store.count()?,
            db_size_bytes: store.db_size_bytes()?,
            platform: std::env::consts::OS.to_string(),
        })
    }

    pub fn set_paused(&self, paused: bool) -> Result<()> {
        self.paused.store(paused, Ordering::SeqCst);
        let mut config = AppConfig::load().unwrap_or_default();
        config.capture_paused = paused;
        config.save()?;
        self.emit_update(false);
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let config = AppConfig::load().unwrap_or_default();
        self.paused
            .store(config.capture_paused, Ordering::SeqCst);

        let hub = platform_hub::create_and_start_hub()?;
        *self.hub.lock() = Some(hub);

        self.stop.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);
        *self.started_at.lock() = Some(Utc::now());

        let stop = Arc::clone(&self.stop);
        let paused = Arc::clone(&self.paused);
        let running = Arc::clone(&self.running);
        let started_at = Arc::clone(&self.started_at);
        let last_saved_at = Arc::clone(&self.last_saved_at);
        let current = Arc::clone(&self.current);
        let on_update = self.on_update.lock().clone();
        let hub = self.hub.lock().clone().context("hub not initialized")?;

        let handle = thread::Builder::new()
            .name("ps-daemon-loop".into())
            .spawn(move || {
                let store = match open_default_store() {
                    Ok(store) => store,
                    Err(err) => {
                        error!(?err, "failed to open draft store");
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                };
                let config = AppConfig::load().unwrap_or_default();
                let mut debounce = DebounceEngine::new(config.debounce_ms);

                let emit = |snapshot: Option<CaptureSnapshot>, persisted: bool| {
                    if let Some(cb) = &on_update {
                        let paused = paused.load(Ordering::SeqCst);
                        let saved = *last_saved_at.lock();
                        let update = match snapshot {
                            Some(s) => {
                                CurrentPromptUpdate::from_snapshot(s, persisted, saved, paused)
                            }
                            None => CurrentPromptUpdate::empty(paused),
                        };
                        cb(update);
                    }
                };

                info!(db = %store.db_path().display(), "capture daemon started");

                while !stop.load(Ordering::SeqCst) {
                    if paused.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }

                    if let Ok(Some(injected)) = ps_core::take_inject_snapshot() {
                        if let Some(merged) = MergeEngine::merge(Some(&injected), None, None) {
                            debounce.observe(merged.clone());
                            *current.lock() = Some(merged.clone());
                            emit(Some(merged), false);
                        }
                    }

                    let (ax, keystroke, clipboard) = platform_hub::hub_snapshots(&hub);
                    if let Some(merged) =
                        MergeEngine::merge(ax.as_ref(), keystroke.as_ref(), clipboard.as_ref())
                    {
                        debounce.observe(merged.clone());
                        *current.lock() = Some(merged.clone());
                        emit(Some(merged), false);
                    }

                    if let Some(snapshot) = debounce.poll_ready() {
                        match store.upsert_snapshot(&snapshot) {
                            Ok(true) => {
                                let now = Utc::now();
                                *last_saved_at.lock() = Some(now);
                                *current.lock() = Some(snapshot.clone());
                                info!(
                                    app = %snapshot.slot.context.app_name,
                                    chars = snapshot.content.chars().count(),
                                    "draft persisted"
                                );
                                emit(Some(snapshot), true);
                            }
                            Ok(false) => {}
                            Err(err) => error!(?err, "failed to persist draft"),
                        }
                    }

                    thread::sleep(Duration::from_millis(100));
                }

                platform_hub::hub_stop(&hub);
                running.store(false, Ordering::SeqCst);
                *started_at.lock() = None;
                info!("capture daemon stopped");
            })
            .context("spawn daemon loop")?;

        *self.handle.lock() = Some(handle);
        Ok(())
    }

    pub fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.lock().take() {
            let _ = handle.join();
        }
        if let Some(hub) = self.hub.lock().take() {
            platform_hub::hub_stop(&hub);
        }
        self.running.store(false, Ordering::SeqCst);
        *self.started_at.lock() = None;
    }

    fn emit_update(&self, persisted: bool) {
        if let Some(cb) = self.on_update.lock().clone() {
            let paused = self.paused.load(Ordering::SeqCst);
            let last_saved_at = *self.last_saved_at.lock();
            let update = match self.current.lock().clone() {
                Some(snapshot) => {
                    CurrentPromptUpdate::from_snapshot(snapshot, persisted, last_saved_at, paused)
                }
                None => CurrentPromptUpdate::empty(paused),
            };
            cb(update);
        }
    }

    #[cfg(test)]
    pub fn inject_snapshot_for_test(&self, snapshot: CaptureSnapshot) {
        *self.current.lock() = Some(snapshot);
    }
}

impl Drop for CaptureDaemon {
    fn drop(&mut self) {
        self.stop();
    }
}

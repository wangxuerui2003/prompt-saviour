use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use parking_lot::Mutex;
use ps_core::draft::CaptureSnapshot;
use ps_core::{build_snapshot, CaptureSource, SessionContext};
use tracing::{error, info, warn};

pub use crate::context::frontmost_session;
pub use crate::permissions::{check_permissions, prompt_for_permissions, PermissionStatus};

#[derive(Default)]
pub struct CaptureHub {
    pub(crate) ax_snapshot: Mutex<Option<CaptureSnapshot>>,
    pub(crate) keystroke_snapshot: Mutex<Option<CaptureSnapshot>>,
    pub(crate) clipboard_snapshot: Mutex<Option<CaptureSnapshot>>,
    pub(crate) force_ax_poll: AtomicBool,
    pub(crate) running: AtomicBool,
}

impl CaptureHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn request_immediate_ax_poll(&self) {
        self.force_ax_poll.store(true, Ordering::SeqCst);
    }

    pub fn snapshots(&self) -> (Option<CaptureSnapshot>, Option<CaptureSnapshot>, Option<CaptureSnapshot>) {
        (
            self.ax_snapshot.lock().clone(),
            self.keystroke_snapshot.lock().clone(),
            self.clipboard_snapshot.lock().clone(),
        )
    }

    pub fn start(self: &Arc<Self>) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            anyhow::bail!("capture hub already running");
        }

        let perms = check_permissions();
        if !perms.accessibility {
            warn!("Accessibility permission missing — GUI capture disabled until granted");
        }
        if !perms.input_monitoring {
            warn!("Input Monitoring permission missing — keystroke capture disabled until granted");
        }

        let config = ps_core::config::AppConfig::load().unwrap_or_default();

        if perms.accessibility {
            let hub = Arc::clone(self);
            let poll_ms = config.ax_poll_ms;
            thread::Builder::new()
                .name("ps-ax-poll".into())
                .spawn(move || ax_poll_loop(hub, poll_ms))
                .context("spawn ax poll thread")?;
        }

        if perms.input_monitoring {
            let hub = Arc::clone(self);
            thread::Builder::new()
                .name("ps-keystroke".into())
                .spawn(move || crate::keystroke::run_event_tap(hub))
                .context("spawn keystroke thread")?;
        }

        info!("capture hub started");
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Test helper: inject snapshots without OS capture.
    pub fn inject(
        &self,
        ax: Option<CaptureSnapshot>,
        keystroke: Option<CaptureSnapshot>,
        clipboard: Option<CaptureSnapshot>,
    ) {
        *self.ax_snapshot.lock() = ax;
        *self.keystroke_snapshot.lock() = keystroke;
        *self.clipboard_snapshot.lock() = clipboard;
    }
}

fn ax_poll_loop(hub: Arc<CaptureHub>, poll_ms: u64) {
    while hub.running.load(Ordering::SeqCst) {
        let force = hub.force_ax_poll.swap(false, Ordering::SeqCst);
        poll_ax_once(&hub);
        let delay = if force { 50 } else { poll_ms };
        thread::sleep(Duration::from_millis(delay));
    }
}

fn poll_ax_once(hub: &CaptureHub) {
    let Some(session) = frontmost_session() else {
        return;
    };
    if is_excluded(&session) {
        return;
    }
    match crate::ax::read_focused_text() {
        Ok(Some(text)) => {
            let snapshot = build_snapshot(session, text, CaptureSource::Accessibility);
            *hub.ax_snapshot.lock() = Some(snapshot);
        }
        Ok(None) => {}
        Err(err) => {
            error!(?err, "ax poll failed");
        }
    }
}

fn is_excluded(session: &SessionContext) -> bool {
    let config = ps_core::config::AppConfig::load().unwrap_or_default();
    config
        .excluded_bundle_ids
        .iter()
        .any(|id| id == &session.bundle_id)
}

pub fn build_keystroke_snapshot(session: SessionContext, content: String) -> CaptureSnapshot {
    build_snapshot(session, content, CaptureSource::Keystroke)
}

pub fn build_clipboard_snapshot(session: SessionContext, content: String) -> CaptureSnapshot {
    build_snapshot(session, content, CaptureSource::Clipboard)
}

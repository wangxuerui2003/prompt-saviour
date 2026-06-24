mod clipboard;
mod context;
mod permissions;
mod uia;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Context as AnyhowContext;
use parking_lot::Mutex;
use ps_core::draft::CaptureSnapshot;
use ps_core::{build_snapshot, CaptureSource, SessionContext};
use ps_input::{run_keystroke_listener, KeystrokeHost, PasteChord};
use tracing::{error, info, warn};

pub use context::frontmost_session;
pub use permissions::{check_permissions, prompt_for_permissions, PermissionStatus};

#[derive(Default)]
pub struct CaptureHub {
    ax_snapshot: Mutex<Option<CaptureSnapshot>>,
    keystroke_snapshot: Mutex<Option<CaptureSnapshot>>,
    clipboard_snapshot: Mutex<Option<CaptureSnapshot>>,
    force_gui_poll: AtomicBool,
    running: AtomicBool,
}

impl CaptureHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn request_immediate_ax_poll(&self) {
        self.force_gui_poll.store(true, Ordering::SeqCst);
    }

    pub fn snapshots(&self) -> (Option<CaptureSnapshot>, Option<CaptureSnapshot>, Option<CaptureSnapshot>) {
        (
            self.ax_snapshot.lock().clone(),
            self.keystroke_snapshot.lock().clone(),
            self.clipboard_snapshot.lock().clone(),
        )
    }

    pub fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            anyhow::bail!("capture hub already running");
        }

        let perms = check_permissions();
        if !perms.ui_automation {
            warn!("UI Automation may be unavailable — GUI capture degraded");
        }

        let config = ps_core::config::AppConfig::load().unwrap_or_default();
        let poll_ms = config.ax_poll_ms;

        {
            let hub = Arc::clone(self);
            thread::Builder::new()
                .name("ps-uia-poll".into())
                .spawn(move || uia_poll_loop(hub, poll_ms))
                .context("spawn uia poll thread")?;
        }

        if perms.input_monitoring {
            let hub = Arc::clone(self);
            thread::Builder::new()
                .name("ps-keystroke".into())
                .spawn(move || start_keystroke(hub))
                .context("spawn keystroke thread")?;
        } else {
            warn!("input monitoring unavailable — terminal keystroke capture disabled");
        }

        info!("windows capture hub started");
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn uia_poll_loop(hub: Arc<CaptureHub>, poll_ms: u64) {
    while hub.running.load(Ordering::SeqCst) {
        let force = hub.force_gui_poll.swap(false, Ordering::SeqCst);
        poll_uia_once(&hub);
        let delay = if force { 50 } else { poll_ms };
        thread::sleep(Duration::from_millis(delay));
    }
}

fn poll_uia_once(hub: &CaptureHub) {
    let Some(session) = frontmost_session() else {
        return;
    };
    if is_excluded(&session) {
        return;
    }
    match uia::read_focused_text() {
        Ok(Some(text)) => {
            let snapshot = build_snapshot(session, text, CaptureSource::Accessibility);
            *hub.ax_snapshot.lock() = Some(snapshot);
        }
        Ok(None) => {}
        Err(err) => error!(?err, "uia poll failed"),
    }
}

fn is_excluded(session: &SessionContext) -> bool {
    let config = ps_core::config::AppConfig::load().unwrap_or_default();
    config
        .excluded_bundle_ids
        .iter()
        .any(|id| id == &session.bundle_id)
}

fn start_keystroke(hub: Arc<CaptureHub>) {
    let running = Arc::new(AtomicBool::new(true));
    running.store(hub.running.load(Ordering::SeqCst), Ordering::SeqCst);

    let hub_for_poll = Arc::clone(&hub);
    let hub_for_snap = Arc::clone(&hub);
    let hub_for_session = Arc::clone(&hub);

    let host = KeystrokeHost {
        running: Arc::new(AtomicBool::new(true)),
        on_snapshot: Arc::new(move |snapshot| {
            *hub_for_snap.keystroke_snapshot.lock() = Some(snapshot);
        }),
        session: Arc::new(move || frontmost_session()),
        clipboard: Arc::new(clipboard::read_clipboard_text),
        request_gui_poll: Arc::new(move || hub_for_poll.request_immediate_ax_poll()),
        paste_chord: PasteChord::Control,
    };

    let host_running = Arc::clone(&host.running);
    thread::spawn(move || {
        while hub_for_session.running.load(Ordering::SeqCst) {
            host_running.store(hub_for_session.running.load(Ordering::SeqCst), Ordering::SeqCst);
            thread::sleep(Duration::from_millis(100));
        }
        host_running.store(false, Ordering::SeqCst);
    });

    run_keystroke_listener(host);
}

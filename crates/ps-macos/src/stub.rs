use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use ps_core::draft::CaptureSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub input_monitoring: bool,
}

pub fn check_permissions() -> PermissionStatus {
    PermissionStatus {
        accessibility: false,
        input_monitoring: false,
    }
}

pub fn prompt_for_permissions() -> PermissionStatus {
    check_permissions()
}

pub fn frontmost_session() -> Option<ps_core::SessionContext> {
    None
}

#[derive(Default)]
pub struct CaptureHub {
    ax_snapshot: Mutex<Option<CaptureSnapshot>>,
    keystroke_snapshot: Mutex<Option<CaptureSnapshot>>,
    clipboard_snapshot: Mutex<Option<CaptureSnapshot>>,
    force_ax_poll: AtomicBool,
    running: AtomicBool,
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

    pub fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            anyhow::bail!("capture hub already running");
        }
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

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

pub fn build_keystroke_snapshot(
    session: ps_core::SessionContext,
    content: String,
) -> CaptureSnapshot {
    ps_core::build_snapshot(session, content, ps_core::CaptureSource::Keystroke)
}

pub fn build_clipboard_snapshot(
    session: ps_core::SessionContext,
    content: String,
) -> CaptureSnapshot {
    ps_core::build_snapshot(session, content, ps_core::CaptureSource::Clipboard)
}

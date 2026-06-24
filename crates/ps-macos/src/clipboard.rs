use std::sync::Arc;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use ps_core::SessionContext;
use tracing::debug;

use crate::{build_clipboard_snapshot, CaptureHub};
use crate::context::frontmost_session;

#[allow(dead_code)]
pub fn run_clipboard_watcher(hub: Arc<CaptureHub>) {
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(?err, "clipboard init failed");
            return;
        }
    };
    let mut last_text = String::new();

    while hub.running.load(std::sync::atomic::Ordering::SeqCst) {
        match clipboard.get_text() {
            Ok(text) => {
                if text != last_text && text.trim().len() >= 8 {
                    if let Some(session) = frontmost_session() {
                        let snapshot = build_clipboard_snapshot(session, text.clone());
                        *hub.clipboard_snapshot.lock() = Some(snapshot);
                        debug!(chars = text.chars().count(), "clipboard draft updated");
                    }
                    last_text = text;
                }
            }
            Err(_) => {}
        }
        thread::sleep(Duration::from_millis(350));
    }
}

pub fn read_clipboard_text() -> Option<String> {
    Clipboard::new()
        .ok()?
        .get_text()
        .ok()
        .filter(|t| !t.trim().is_empty())
}

#[allow(dead_code)]
pub fn append_clipboard_to_session(_session: &SessionContext) -> Option<String> {
    read_clipboard_text()
}

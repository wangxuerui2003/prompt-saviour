use std::sync::Arc;

use ps_core::draft::CaptureSnapshot;

#[cfg(target_os = "macos")]
pub type Hub = Arc<ps_macos::CaptureHub>;
#[cfg(target_os = "windows")]
pub type Hub = Arc<ps_windows::CaptureHub>;
#[cfg(target_os = "linux")]
pub type Hub = Arc<ps_linux::CaptureHub>;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub type Hub = Arc<InjectOnlyHub>;

pub fn create_and_start_hub() -> anyhow::Result<Hub> {
    #[cfg(target_os = "macos")]
    {
        let hub = ps_macos::CaptureHub::new();
        hub.start()?;
        return Ok(hub);
    }
    #[cfg(target_os = "windows")]
    {
        let hub = ps_windows::CaptureHub::new();
        hub.start()?;
        return Ok(hub);
    }
    #[cfg(target_os = "linux")]
    {
        let hub = ps_linux::CaptureHub::new();
        hub.start()?;
        return Ok(hub);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let hub = Arc::new(InjectOnlyHub::default());
        hub.start()?;
        Ok(hub)
    }
}

pub fn hub_snapshots(
    hub: &Hub,
) -> (
    Option<CaptureSnapshot>,
    Option<CaptureSnapshot>,
    Option<CaptureSnapshot>,
) {
    hub.snapshots()
}

pub fn hub_stop(hub: &Hub) {
    hub.stop();
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod inject_only {
    use std::sync::atomic::{AtomicBool, Ordering};

    use parking_lot::Mutex;
    use ps_core::draft::CaptureSnapshot;

    #[derive(Default)]
    pub struct InjectOnlyHub {
        ax_snapshot: Mutex<Option<CaptureSnapshot>>,
        keystroke_snapshot: Mutex<Option<CaptureSnapshot>>,
        clipboard_snapshot: Mutex<Option<CaptureSnapshot>>,
        running: AtomicBool,
    }

    impl InjectOnlyHub {
        pub fn start(&self) -> anyhow::Result<()> {
            self.running.store(true, Ordering::SeqCst);
            Ok(())
        }

        pub fn stop(&self) {
            self.running.store(false, Ordering::SeqCst);
        }

        pub fn snapshots(
            &self,
        ) -> (
            Option<CaptureSnapshot>,
            Option<CaptureSnapshot>,
            Option<CaptureSnapshot>,
        ) {
            (
                self.ax_snapshot.lock().clone(),
                self.keystroke_snapshot.lock().clone(),
                self.clipboard_snapshot.lock().clone(),
            )
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use inject_only::InjectOnlyHub;

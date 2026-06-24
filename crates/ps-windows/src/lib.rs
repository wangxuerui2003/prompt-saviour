#[cfg(target_os = "windows")]
mod clipboard;
#[cfg(target_os = "windows")]
mod context;
#[cfg(target_os = "windows")]
mod permissions;
#[cfg(target_os = "windows")]
mod uia;
#[cfg(target_os = "windows")]
mod platform;
#[cfg(not(target_os = "windows"))]
mod stub;

#[cfg(test)]
mod stub_tests {
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn stub_hub_starts_without_capture_threads() {
        let hub = super::CaptureHub::new();
        hub.start().expect("stub hub should start");
        let (ax, ks, _) = hub.snapshots();
        assert!(ax.is_none());
        assert!(ks.is_none());
        hub.stop();
    }
}

#[cfg(target_os = "windows")]
pub use platform::*;
#[cfg(not(target_os = "windows"))]
pub use stub::*;

#[cfg(target_os = "linux")]
mod atspi_text;
#[cfg(target_os = "linux")]
mod clipboard;
#[cfg(target_os = "linux")]
mod context;
#[cfg(target_os = "linux")]
mod permissions;
#[cfg(target_os = "linux")]
mod platform;
#[cfg(not(target_os = "linux"))]
mod stub;

#[cfg(test)]
mod stub_tests {
    #[cfg(not(target_os = "linux"))]
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

#[cfg(target_os = "linux")]
pub use platform::*;
#[cfg(not(target_os = "linux"))]
pub use stub::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn is_launch_at_login_enabled() -> bool {
    #[cfg(target_os = "macos")]
    {
        return macos::is_enabled();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::is_enabled();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::is_enabled();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

pub fn set_launch_at_login(enabled: bool) -> anyhow::Result<bool> {
    #[cfg(target_os = "macos")]
    {
        return macos::set_enabled(enabled);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::set_enabled(enabled);
    }
    #[cfg(target_os = "linux")]
    {
        return linux::set_enabled(enabled);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = enabled;
        Ok(false)
    }
}

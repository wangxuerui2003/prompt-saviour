#[cfg(target_os = "macos")]
mod ax;
#[cfg(target_os = "macos")]
mod clipboard;
#[cfg(target_os = "macos")]
mod context;
#[cfg(target_os = "macos")]
mod keystroke;
#[cfg(target_os = "macos")]
mod permissions;
#[cfg(target_os = "macos")]
mod platform;
#[cfg(not(target_os = "macos"))]
mod stub;

#[cfg(target_os = "macos")]
pub use platform::*;
#[cfg(not(target_os = "macos"))]
pub use stub::*;

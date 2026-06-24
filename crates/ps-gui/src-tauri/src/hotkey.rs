use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tracing::{info, warn};

pub fn register_global_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    unregister_global_hotkey(app)?;

    let shortcut = hotkey.trim();
    if shortcut.is_empty() {
        return Ok(());
    }

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
        .map_err(|e| e.to_string())?;

    info!(hotkey = %shortcut, "registered global hotkey");
    Ok(())
}

pub fn unregister_global_hotkey(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())
}

pub fn apply_hotkey_from_config(app: &AppHandle) {
    let config = ps_core::AppConfig::load().unwrap_or_default();
    if let Err(err) = register_global_hotkey(app, &config.global_hotkey) {
        warn!(?err, "failed to register global hotkey");
    }
}

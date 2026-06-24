use tauri::{AppHandle, Emitter, Manager};

use crate::hotkey;
use crate::autostart;
use crate::state::{refresh_tray_menu, AppState};

pub fn apply_config_effects(app: &AppHandle, config: &ps_core::AppConfig) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        state
            .daemon
            .set_paused(config.capture_paused)
            .map_err(|e| e.to_string())?;
    }

    autostart::set_launch_at_login(config.launch_at_login).map_err(|e| e.to_string())?;
    hotkey::register_global_hotkey(app, &config.global_hotkey)?;
    refresh_tray_menu(app, &config.ui_language).map_err(|e| e.to_string())?;
    let _ = app.emit("ui-language-changed", &config.ui_language);
    Ok(())
}

pub fn bootstrap_from_config(app: &AppHandle) -> Result<(), String> {
    let config = ps_core::AppConfig::load().map_err(|e| e.to_string())?;
    if config.launch_at_login && !autostart::is_launch_at_login_enabled() {
        autostart::set_launch_at_login(true).map_err(|e| e.to_string())?;
    }
    hotkey::apply_hotkey_from_config(app);
    refresh_tray_menu(app, &config.ui_language).map_err(|e| e.to_string())?;
    Ok(())
}

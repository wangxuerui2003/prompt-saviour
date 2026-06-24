mod autostart;
pub mod commands;
mod hotkey;
mod logging;
mod services;
mod state;
mod watchdog;

use std::sync::Arc;

use tauri::Manager;

use crate::logging::init_file_logging;
use crate::services::bootstrap_from_config;
use crate::state::{init_tray, setup_daemon_events, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_guard = init_file_logging().expect("failed to init file logging");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let state = AppState::new(log_guard);
            let daemon = Arc::clone(&state.daemon);
            let watchdog = Arc::clone(&state.watchdog);
            setup_daemon_events(app.handle(), Arc::clone(&daemon), watchdog);
            daemon.start().map_err(|e| {
                tracing::error!(?e, "failed to start capture daemon");
                e
            })?;
            state.watchdog.start(app.handle().clone());
            app.manage(state);
            init_tray(app.handle())?;
            bootstrap_from_config(app.handle()).map_err(|e| {
                tracing::error!(?e, "failed to apply startup config");
                e
            })?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_current_prompt,
            commands::get_daemon_status,
            commands::list_drafts,
            commands::search_drafts,
            commands::get_draft,
            commands::delete_draft,
            commands::delete_all_drafts,
            commands::pin_draft,
            commands::recover_draft,
            commands::copy_text,
            commands::get_config,
            commands::save_config,
            commands::set_capture_paused,
            commands::get_permissions,
            commands::refresh_permissions,
            commands::prompt_for_permissions,
            commands::open_accessibility_settings,
            commands::open_input_monitoring_settings,
            commands::open_data_dir,
            commands::get_system_info,
            commands::mark_slot_submitted,
            commands::export_draft_text,
            commands::get_executable_path,
            commands::get_frontmost_session,
            commands::inject_draft_for_test,
            commands::get_recent_logs,
            commands::get_launch_at_login_status,
            commands::set_launch_at_login,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                if let Some(state) = app.try_state::<AppState>() {
                    state.daemon.stop();
                    state.watchdog.stop();
                    let _ = crate::hotkey::unregister_global_hotkey(&app.app_handle());
                }
            }
        });
}

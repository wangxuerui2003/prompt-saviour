use std::path::PathBuf;
use std::process::Command;
use arboard::Clipboard;
use ps_core::{open_default_store, AppConfig, DraftRecord};
use ps_daemon::{CurrentPromptUpdate, DaemonStatus};
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::services::apply_config_effects;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct PermissionsView {
    pub platform: String,
    pub gui_capture: bool,
    pub input_monitoring: bool,
    pub executable_path: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfoView {
    pub status: DaemonStatus,
    pub permissions: PermissionsView,
}

#[tauri::command]
pub fn get_current_prompt(state: State<'_, AppState>) -> CurrentPromptUpdate {
    state.daemon.current_prompt()
}

#[tauri::command]
pub fn get_daemon_status(state: State<'_, AppState>) -> Result<DaemonStatus, String> {
    state.daemon.status().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_drafts(limit: Option<usize>) -> Result<Vec<DraftRecord>, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    store
        .list_recent(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_drafts(query: String, limit: Option<usize>) -> Result<Vec<DraftRecord>, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    if query.trim().is_empty() {
        return store.list_recent(limit.unwrap_or(50)).map_err(|e| e.to_string());
    }
    store
        .search(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_draft(id: i64) -> Result<Option<DraftRecord>, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    store.get_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_draft(id: i64) -> Result<bool, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    store.delete_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_drafts() -> Result<usize, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    store.delete_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pin_draft(id: i64, pinned: bool) -> Result<bool, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    store.set_pinned(id, pinned).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_draft(id: Option<i64>) -> Result<DraftRecord, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    let draft = match id {
        Some(id) => store
            .get_by_id(id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("draft #{id} not found"))?,
        None => store
            .get_latest()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no drafts available".to_string())?,
    };

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(draft.content.clone())
        .map_err(|e| e.to_string())?;
    Ok(draft)
}

pub fn recover_latest_draft(_app: AppHandle) -> Result<DraftRecord, String> {
    recover_draft(None)
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    AppConfig::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(config: AppConfig, app: AppHandle) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    apply_config_effects(&app, &config)
}

#[tauri::command]
pub fn set_capture_paused(paused: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.daemon.set_paused(paused).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_permissions() -> Result<PermissionsView, String> {
    Ok(build_permissions_view())
}

#[tauri::command]
pub fn refresh_permissions() -> Result<PermissionsView, String> {
    Ok(build_permissions_view())
}

#[tauri::command]
pub fn prompt_for_permissions() -> Result<PermissionsView, String> {
    #[cfg(target_os = "macos")]
    {
        ps_macos::prompt_for_permissions();
    }
    #[cfg(target_os = "windows")]
    {
        ps_windows::prompt_for_permissions();
    }
    #[cfg(target_os = "linux")]
    {
        ps_linux::prompt_for_permissions();
    }
    Ok(build_permissions_view())
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "ms-settings:privacy"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open")
            .arg("gnome-control-center")
            .spawn();
    }
    Ok(())
}

#[tauri::command]
pub fn open_input_monitoring_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "ms-settings:privacy"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        for target in [
            "gnome-control-center privacy",
            "kcmshell5 kcm_input",
            "systemsettings5",
        ] {
            if Command::new("sh")
                .args(["-c", &format!("xdg-open {target}")])
                .spawn()
                .is_ok()
            {
                break;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_data_dir() -> Result<(), String> {
    let dir = ps_core::data_dir().map_err(|e| e.to_string())?;
    open_path_in_finder(&dir)
}

#[tauri::command]
pub fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfoView, String> {
    let status = state.daemon.status().map_err(|e| e.to_string())?;
    Ok(SystemInfoView {
        status,
        permissions: build_permissions_view(),
    })
}

#[tauri::command]
pub fn mark_slot_submitted(slot_key: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut slots = state.submitted_slots.lock();
    if !slots.iter().any(|s| s == &slot_key) {
        slots.push(slot_key);
    }
    Ok(())
}

#[tauri::command]
pub fn export_draft_text(id: i64) -> Result<String, String> {
    let store = open_default_store().map_err(|e| e.to_string())?;
    let draft = store
        .get_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("draft #{id} not found"))?;
    Ok(draft.content)
}

#[tauri::command]
pub fn get_executable_path() -> Result<String, String> {
    Ok(executable_path().display().to_string())
}

#[tauri::command]
pub fn get_frontmost_session() -> Result<Option<ps_core::SessionContext>, String> {
    #[cfg(target_os = "macos")]
    {
        return Ok(ps_macos::frontmost_session());
    }
    #[cfg(target_os = "windows")]
    {
        return Ok(ps_windows::frontmost_session());
    }
    #[cfg(target_os = "linux")]
    {
        return Ok(ps_linux::frontmost_session());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(None)
    }
}

#[tauri::command]
pub fn inject_draft_for_test(text: String, app: String, bundle: String) -> Result<(), String> {
    ps_core::write_inject_file(&text, &app, &bundle).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_logs(lines: Option<usize>) -> Result<String, String> {
    crate::logging::read_recent_logs(lines.unwrap_or(200)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_launch_at_login_status() -> Result<bool, String> {
    Ok(crate::autostart::is_launch_at_login_enabled())
}

#[tauri::command]
pub fn set_launch_at_login(enabled: bool) -> Result<bool, String> {
    crate::autostart::set_launch_at_login(enabled).map_err(|e| e.to_string())
}

fn build_permissions_view() -> PermissionsView {
    let platform = std::env::consts::OS.to_string();
    #[cfg(target_os = "macos")]
    let (gui_capture, input_monitoring) = {
        let perms = ps_macos::check_permissions();
        (perms.accessibility, perms.input_monitoring)
    };
    #[cfg(target_os = "windows")]
    let (gui_capture, input_monitoring) = {
        let perms = ps_windows::check_permissions();
        (perms.ui_automation, perms.input_monitoring)
    };
    #[cfg(target_os = "linux")]
    let (gui_capture, input_monitoring) = {
        let perms = ps_linux::check_permissions();
        (perms.at_spi, perms.input_monitoring)
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let (gui_capture, input_monitoring) = (false, false);

    let data_dir = ps_core::data_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    PermissionsView {
        platform,
        gui_capture,
        input_monitoring,
        executable_path: executable_path().display().to_string(),
        data_dir,
    }
}

fn executable_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("prompt-saviour"))
}

fn open_path_in_finder(path: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        return Err(format!("open folder is not implemented on {}", std::env::consts::OS));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    static ENV_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn list_drafts_command_works() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("PROMPT_SAVIOUR_HOME", dir.path());
        let drafts = list_drafts(Some(10)).unwrap();
        assert!(drafts.is_empty());
    }
}

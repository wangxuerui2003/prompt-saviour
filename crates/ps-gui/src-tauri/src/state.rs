use std::sync::Arc;

use parking_lot::Mutex;
use ps_daemon::CaptureDaemon;
use tauri::{AppHandle, Emitter, Manager};

use crate::logging::FileLogGuard;
use crate::watchdog::AgentWatchdog;

pub struct AppState {
    pub daemon: Arc<CaptureDaemon>,
    pub submitted_slots: Mutex<Vec<String>>,
    pub watchdog: Arc<AgentWatchdog>,
    pub _log_guard: FileLogGuard,
}

impl AppState {
    pub fn new(log_guard: FileLogGuard) -> Self {
        Self {
            daemon: Arc::new(CaptureDaemon::new()),
            submitted_slots: Mutex::new(Vec::new()),
            watchdog: Arc::new(AgentWatchdog::new()),
            _log_guard: log_guard,
        }
    }
}

struct TrayLabels {
    show: &'static str,
    pause: &'static str,
    resume: &'static str,
    copy: &'static str,
    quit: &'static str,
    tooltip: &'static str,
}

fn tray_labels(locale: &str) -> TrayLabels {
    if locale == "zh-CN" {
        TrayLabels {
            show: "打开 Prompt Saviour",
            pause: "暂停保护",
            resume: "恢复保护",
            copy: "复制最新 Draft",
            quit: "退出",
            tooltip: "Prompt Saviour",
        }
    } else {
        TrayLabels {
            show: "Open Prompt Saviour",
            pause: "Pause Protection",
            resume: "Resume Protection",
            copy: "Copy Latest Draft",
            quit: "Quit",
            tooltip: "Prompt Saviour",
        }
    }
}

pub fn setup_daemon_events(app: &AppHandle, daemon: Arc<CaptureDaemon>, watchdog: Arc<AgentWatchdog>) {
    let handle = app.clone();
    daemon.set_update_callback(Arc::new(move |update| {
        if let Some(snapshot) = &update.snapshot {
            watchdog.track_snapshot(snapshot);
        }
        let _ = handle.emit("current-prompt-update", &update);
    }));
}

pub fn init_tray(app: &AppHandle) -> tauri::Result<()> {
    let config = ps_core::AppConfig::load().unwrap_or_default();
    build_tray(app, &config.ui_language)
}

pub fn refresh_tray_menu(app: &AppHandle, locale: &str) -> tauri::Result<()> {
    build_tray(app, locale)
}

fn build_tray(app: &AppHandle, locale: &str) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let labels = tray_labels(locale);
    let paused = app
        .try_state::<AppState>()
        .map(|s| s.daemon.is_paused())
        .unwrap_or(false);
    let pause_label = if paused {
        labels.resume
    } else {
        labels.pause
    };

    let show = MenuItem::with_id(app, "show", labels.show, true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", pause_label, true, None::<&str>)?;
    let copy = MenuItem::with_id(app, "copy", labels.copy, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &pause, &copy, &quit])?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(labels.tooltip))?;
        return Ok(());
    }

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip(labels.tooltip)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "pause" => {
                if let Some(state) = app.try_state::<AppState>() {
                    let paused = !state.daemon.is_paused();
                    let _ = state.daemon.set_paused(paused);
                    if let Ok(config) = ps_core::AppConfig::load() {
                        let _ = refresh_tray_menu(app, &config.ui_language);
                    }
                }
            }
            "copy" => {
                let _ = crate::commands::recover_latest_draft(app.clone());
            }
            "quit" => {
                if let Some(state) = app.try_state::<AppState>() {
                    state.daemon.stop();
                    state.watchdog.stop();
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

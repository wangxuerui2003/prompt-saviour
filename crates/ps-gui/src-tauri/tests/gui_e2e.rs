//! GUI backend E2E: daemon service + storage commands used by Tauri.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use ps_core::{open_default_store, write_inject_file, AppConfig, DraftStore};
use ps_daemon::CaptureDaemon;
use prompt_saviour_gui_lib::commands;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_guard() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn with_home<F: FnOnce()>(dir: &Path, f: F) {
    let _guard = env_guard();
    std::env::set_var("PROMPT_SAVIOUR_HOME", dir);
    f();
    std::env::remove_var("PROMPT_SAVIOUR_HOME");
}

#[test]
fn gui_config_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let mut cfg = AppConfig::load().unwrap();
        cfg.capture_paused = true;
        cfg.ax_poll_ms = 300;
        cfg.ui_language = "zh-CN".into();
        cfg.save().unwrap();
        let loaded = AppConfig::load().unwrap();
        assert!(loaded.capture_paused);
        assert_eq!(loaded.ax_poll_ms, 300);
        assert_eq!(loaded.ui_language, "zh-CN");
    });
}

#[test]
fn recent_logs_command_reads_file() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let log_path = dir.path().join("prompt-saviour.log");
        std::fs::write(&log_path, "line one\nline two\n").unwrap();
        let logs = commands::get_recent_logs(Some(10)).unwrap();
        assert!(logs.contains("line one"));
    });
}

#[test]
fn launch_agent_status_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let before = commands::get_launch_at_login_status().unwrap();
        assert!(!before);
        let enabled = commands::set_launch_at_login(true).expect("set launch at login");
        assert!(enabled);
        assert!(commands::get_launch_at_login_status().expect("status"));
        commands::set_launch_at_login(false).expect("disable launch at login");
        assert!(!commands::get_launch_at_login_status().unwrap());
    });
}

#[test]
fn gui_storage_search_pin_delete() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let store = DraftStore::open(dir.path().join("drafts.db")).unwrap();
        let snap = ps_core::build_snapshot(
            ps_core::SessionContext {
                bundle_id: "com.test.gui".into(),
                app_name: "GUI Test".into(),
                pid: 1,
                window_title: "W".into(),
            },
            "GUI history search prompt content".into(),
            ps_core::CaptureSource::Merged,
        );
        store.upsert_snapshot(&snap).unwrap();
        let drafts = store.search("search prompt", 10).unwrap();
        assert_eq!(drafts.len(), 1);
        let id = drafts[0].id;
        store.set_pinned(id, true).unwrap();
        let pinned = store.get_by_id(id).unwrap().unwrap();
        assert!(pinned.pinned);
        assert!(store.delete_by_id(id).unwrap());
        assert!(store.list_recent(10).unwrap().is_empty());
    });
}

#[test]
fn capture_daemon_persists_injected_prompt() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let daemon = Arc::new(CaptureDaemon::new());
        let updates = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&updates);
        daemon.set_update_callback(Arc::new(move |update| {
            captured.lock().unwrap().push(update);
        }));
        daemon.start().expect("daemon should start");

        let text = "GUI daemon inject e2e prompt text";
        write_inject_file(text, "TextEdit", "com.apple.TextEdit").unwrap();

        let mut persisted = false;
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(100));
            let store = open_default_store().unwrap();
            if let Some(latest) = store.get_latest().unwrap() {
                if latest.content.contains("GUI daemon inject") {
                    persisted = true;
                    break;
                }
            }
        }

        daemon.stop();
        assert!(persisted, "injected prompt should be persisted by daemon");
        let updates = updates.lock().unwrap();
        assert!(!updates.is_empty(), "daemon should emit prompt updates");
    });
}

#[test]
fn permissions_view_has_paths() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let view = commands::get_permissions().unwrap();
        assert!(!view.executable_path.is_empty());
        assert!(view.data_dir.contains(dir.path().to_str().unwrap()));
    });
}

#[test]
fn daemon_status_reports_db() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let daemon = CaptureDaemon::new();
        let status = daemon.status().unwrap();
        assert!(status.db_path.contains("drafts.db"));
        assert_eq!(status.draft_count, 0);
    });
}

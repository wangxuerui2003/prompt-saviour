//! End-to-end pipeline test: inject → merge → debounce → storage → recover

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use ps_core::{
    build_snapshot, AppConfig, CaptureSource, DebounceEngine, DraftStore, MergeEngine,
    SessionContext,
};

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

fn test_session(app: &str) -> SessionContext {
    SessionContext {
        bundle_id: "com.apple.TextEdit".into(),
        app_name: app.into(),
        pid: 9999,
        window_title: "Untitled".into(),
    }
}

#[test]
fn pipeline_persists_gui_prompt_after_debounce() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let store = DraftStore::open(dir.path().join("drafts.db")).unwrap();
        let mut debounce = DebounceEngine::new(50);

        let ax = build_snapshot(
            test_session("TextEdit"),
            "This is an end-to-end test prompt for prompt-saviour".into(),
            CaptureSource::Accessibility,
        );

        let merged = MergeEngine::merge(Some(&ax), None, None).unwrap();
        debounce.observe(merged);
        std::thread::sleep(std::time::Duration::from_millis(60));

        let ready = debounce.poll_ready().expect("debounce should fire");
        assert!(store.upsert_snapshot(&ready).unwrap());

        let drafts = store.list_recent(10).unwrap();
        assert_eq!(drafts.len(), 1);
        assert!(drafts[0].content.contains("end-to-end test prompt"));
    });
}

#[test]
fn pipeline_terminal_prefers_keystroke_track() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let store = DraftStore::open(dir.path().join("drafts.db")).unwrap();
        let mut debounce = DebounceEngine::new(10);

        let session = SessionContext {
            bundle_id: "pid:1234".into(),
            app_name: "iTerm2".into(),
            pid: 1234,
            window_title: "claude".into(),
        };

        let ks = build_snapshot(
            session.clone(),
            "terminal agent prompt typed in iterm window".into(),
            CaptureSource::Keystroke,
        );
        let ax = build_snapshot(
            session,
            "short".into(),
            CaptureSource::Accessibility,
        );

        let merged = MergeEngine::merge(Some(&ax), Some(&ks), None).unwrap();
        debounce.observe(merged);
        std::thread::sleep(std::time::Duration::from_millis(15));
        let ready = debounce.poll_ready().unwrap();
        store.upsert_snapshot(&ready).unwrap();

        let latest = store.get_latest().unwrap().unwrap();
        assert!(latest.content.contains("terminal agent prompt"));
    });
}

#[test]
fn config_respects_prompt_saviour_home() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let cfg = AppConfig::load().unwrap();
        assert_eq!(cfg.debounce_ms, 500);
        assert!(dir.path().join("config.json").exists());
    });
}

#[test]
fn recover_roundtrip_content() {
    let dir = tempfile::tempdir().unwrap();
    with_home(dir.path(), || {
        let store = DraftStore::open(dir.path().join("drafts.db")).unwrap();
        let snap = build_snapshot(
            test_session("Cursor"),
            "recover me after crash please".into(),
            CaptureSource::Merged,
        );
        store.upsert_snapshot(&snap).unwrap();
        let latest = store.get_latest().unwrap().unwrap();
        assert_eq!(latest.content, "recover me after crash please");
    });
}

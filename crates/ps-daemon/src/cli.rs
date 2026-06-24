use anyhow::{Context, Result};
use arboard::Clipboard;
use chrono::Local;
use ps_core::{
    build_snapshot, open_default_store, write_inject_file, AppConfig, CaptureSource,
    DebounceEngine, DraftRecord, MergeEngine, SessionContext,
};

pub fn list_drafts(limit: usize) -> Result<()> {
    let store = open_default_store()?;
    let drafts = store.list_recent(limit)?;
    if drafts.is_empty() {
        println!("No drafts saved yet.");
        return Ok(());
    }
    for draft in drafts {
        print_draft_line(&draft);
    }
    Ok(())
}

pub fn recover_draft(id: Option<i64>) -> Result<()> {
    let store = open_default_store()?;
    let draft = match id {
        Some(id) => store
            .get_by_id(id)?
            .with_context(|| format!("draft #{id} not found"))?,
        None => store
            .get_latest()?
            .context("no drafts available")?,
    };

    let mut clipboard = Clipboard::new().context("open clipboard")?;
    clipboard
        .set_text(draft.content.clone())
        .context("copy draft to clipboard")?;

    println!("Recovered draft #{} to clipboard ({} chars).", draft.id, draft.char_count);
    print_draft_line(&draft);
    Ok(())
}

pub fn status() -> Result<()> {
    let store = open_default_store()?;
    let config = AppConfig::load().unwrap_or_default();
    println!("Database: {}", store.db_path().display());
    println!("Config:   {}", ps_core::config::default_config_path()?.display());
    println!("Inject:   {}", ps_core::inject_file_path()?.display());
    println!("AX poll:  {} ms", config.ax_poll_ms);
    println!("Debounce: {} ms", config.debounce_ms);
    Ok(())
}

pub fn inject_draft(text: &str, app: &str, bundle: &str) -> Result<()> {
    write_inject_file(text, app, bundle)?;
    println!(
        "Queued inject payload ({} chars) — running daemon will pick it up within ~100ms.",
        text.chars().count()
    );
    Ok(())
}

pub fn smoke_test(text: &str) -> Result<()> {
    let config = AppConfig::load().unwrap_or_default();
    let store = open_default_store()?;
    let mut debounce = DebounceEngine::new(config.debounce_ms);

    let ax = build_snapshot(
        SessionContext {
            bundle_id: "com.apple.TextEdit".into(),
            app_name: "TextEdit".into(),
            pid: 1,
            window_title: "Smoke".into(),
        },
        text.to_string(),
        CaptureSource::Accessibility,
    );

    let merged = MergeEngine::merge(Some(&ax), None, None)
        .context("merge smoke snapshot")?;
    debounce.observe(merged);

    std::thread::sleep(std::time::Duration::from_millis(config.debounce_ms + 50));
    let ready = debounce
        .poll_ready()
        .context("debounce did not produce snapshot")?;
    store.upsert_snapshot(&ready)?;

    let latest = store.get_latest()?.context("no draft after smoke")?;
    println!("Smoke OK: draft #{} ({} chars)", latest.id, latest.char_count);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn doctor() -> Result<()> {
    use ps_macos::{check_permissions, prompt_for_permissions};

    let before = check_permissions();
    println!("Accessibility / Input Monitoring: {}", yes_no(before.accessibility));
    if !before.accessibility {
        println!("Opening permission prompt…");
        let after = prompt_for_permissions();
        println!("After prompt: {}", yes_no(after.accessibility));
        println!("If still denied, enable prompt-saviour in System Settings → Privacy & Security → Accessibility and Input Monitoring.");
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn doctor() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let perms = ps_windows::check_permissions();
        println!("UI Automation: {}", yes_no(perms.ui_automation));
        println!("Input monitoring: {}", yes_no(perms.input_monitoring));
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        let perms = ps_linux::check_permissions();
        println!("AT-SPI: {}", yes_no(perms.at_spi));
        println!("Input monitoring: {}", yes_no(perms.input_monitoring));
        return Ok(());
    }
    println!(
        "Doctor: capture platform is {} — use smoke/list/recover for storage checks.",
        std::env::consts::OS
    );
    Ok(())
}

fn yes_no(v: bool) -> &'static str {
    if v { "granted" } else { "missing" }
}

fn print_draft_line(draft: &DraftRecord) {
    let local_time = draft.updated_at.with_timezone(&Local);
    let preview: String = draft.content.chars().take(72).collect();
    let preview = preview.replace('\n', " ↵ ");
    println!(
        "#{:<5} {:<16} {:<4} chars {:>5}  {}  {}",
        draft.id,
        source_label(draft.source),
        draft.char_count,
        local_time.format("%m-%d %H:%M"),
        draft.app_name,
        preview
    );
}

fn source_label(source: CaptureSource) -> &'static str {
    match source {
        CaptureSource::Accessibility => "ax",
        CaptureSource::Keystroke => "keys",
        CaptureSource::Clipboard => "clip",
        CaptureSource::Merged => "merge",
    }
}

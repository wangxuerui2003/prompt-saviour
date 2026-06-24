use std::path::PathBuf;

use crate::config;
use crate::draft::CaptureSnapshot;
use crate::merge::build_snapshot;
use crate::draft::{CaptureSource, SessionContext};

pub fn inject_file_path() -> anyhow::Result<PathBuf> {
    Ok(config::data_dir()?.join("inject.json"))
}

pub fn write_inject_file(text: &str, app_name: &str, bundle_id: &str) -> anyhow::Result<()> {
    let path = inject_file_path()?;
    let payload = InjectPayload {
        text: text.to_string(),
        app_name: app_name.to_string(),
        bundle_id: bundle_id.to_string(),
        window_title: app_name.to_string(),
    };
    std::fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
    Ok(())
}

pub fn take_inject_snapshot() -> anyhow::Result<Option<CaptureSnapshot>> {
    let path = inject_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    let _ = std::fs::remove_file(&path);
    let payload: InjectPayload = serde_json::from_str(&raw)?;
    if payload.text.trim().chars().count() < 8 {
        return Ok(None);
    }
    Ok(Some(build_snapshot(
        SessionContext {
            bundle_id: payload.bundle_id,
            app_name: payload.app_name,
            pid: 0,
            window_title: payload.window_title,
        },
        payload.text,
        CaptureSource::Accessibility,
    )))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InjectPayload {
    text: String,
    app_name: String,
    bundle_id: String,
    #[serde(default)]
    window_title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("PROMPT_SAVIOUR_HOME", dir.path());
        write_inject_file("inject file test prompt", "TextEdit", "com.apple.TextEdit").unwrap();
        let snap = take_inject_snapshot().unwrap().unwrap();
        assert!(snap.content.contains("inject file test prompt"));
        assert!(take_inject_snapshot().unwrap().is_none());
        std::env::remove_var("PROMPT_SAVIOUR_HOME");
    }
}

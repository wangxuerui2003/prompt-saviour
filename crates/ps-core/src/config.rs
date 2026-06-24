use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::draft::CaptureSnapshot;

pub fn snapshot_fingerprint(snapshot: &CaptureSnapshot) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    snapshot.slot.key.hash(&mut hasher);
    snapshot.content.hash(&mut hasher);
    hasher.finish()
}

pub struct DebounceEngine {
    pending: Option<(CaptureSnapshot, Instant)>,
    last_persisted: Option<u64>,
    debounce: Duration,
}

impl DebounceEngine {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            pending: None,
            last_persisted: None,
            debounce: Duration::from_millis(debounce_ms),
        }
    }

    /// Feed a freshly merged snapshot. Timer resets only when slot+content changes.
    pub fn observe(&mut self, snapshot: CaptureSnapshot) {
        let fingerprint = snapshot_fingerprint(&snapshot);
        let reset_timer = match &self.pending {
            Some((existing, _)) => snapshot_fingerprint(existing) != fingerprint,
            None => true,
        };
        if reset_timer {
            self.pending = Some((snapshot, Instant::now()));
        }
    }

    /// Returns a snapshot ready to persist after debounce idle, skipping duplicates.
    pub fn poll_ready(&mut self) -> Option<CaptureSnapshot> {
        let (_, since) = self.pending.as_ref()?;
        if since.elapsed() < self.debounce {
            return None;
        }
        let snapshot = self.pending.take()?.0;
        let fingerprint = snapshot_fingerprint(&snapshot);
        if self.last_persisted == Some(fingerprint) {
            return None;
        }
        self.last_persisted = Some(fingerprint);
        Some(snapshot)
    }
}

pub fn resolve_data_dir() -> anyhow::Result<PathBuf> {
    if let Ok(custom) = std::env::var("PROMPT_SAVIOUR_HOME") {
        let dir = PathBuf::from(custom);
        std::fs::create_dir_all(&dir)?;
        return Ok(dir);
    }
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not resolve home directory"))?
        .join(".prompt-saviour");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn data_dir() -> anyhow::Result<PathBuf> {
    resolve_data_dir()
}

pub fn default_db_path() -> anyhow::Result<PathBuf> {
    Ok(data_dir()?.join("drafts.db"))
}

pub fn default_config_path() -> anyhow::Result<PathBuf> {
    Ok(data_dir()?.join("config.json"))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_poll_ms")]
    pub ax_poll_ms: u64,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default = "default_max_drafts")]
    pub max_drafts: u32,
    #[serde(default)]
    pub excluded_bundle_ids: Vec<String>,
    #[serde(default)]
    pub capture_paused: bool,
    #[serde(default = "default_launch_at_login")]
    pub launch_at_login: bool,
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
    #[serde(default = "default_crash_toast")]
    pub crash_toast_enabled: bool,
    #[serde(default = "default_recover_action")]
    pub recover_action: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
}

fn default_poll_ms() -> u64 {
    400
}

fn default_debounce_ms() -> u64 {
    500
}

fn default_retention_days() -> u32 {
    30
}

fn default_max_drafts() -> u32 {
    500
}

fn default_launch_at_login() -> bool {
    true
}

fn default_global_hotkey() -> String {
    "CommandOrControl+Shift+R".into()
}

fn default_crash_toast() -> bool {
    true
}

fn default_recover_action() -> String {
    "clipboard".into()
}

fn default_ui_language() -> String {
    "system".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ax_poll_ms: default_poll_ms(),
            debounce_ms: default_debounce_ms(),
            retention_days: default_retention_days(),
            max_drafts: default_max_drafts(),
            excluded_bundle_ids: Vec::new(),
            capture_paused: false,
            launch_at_login: default_launch_at_login(),
            global_hotkey: default_global_hotkey(),
            crash_toast_enabled: default_crash_toast(),
            recover_action: default_recover_action(),
            ui_language: default_ui_language(),
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = default_config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = default_config_path()?;
        let raw = serde_json::to_string_pretty(self)?;
        std::fs::write(path, raw)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft::{CaptureSource, SessionContext};
    use crate::merge::build_snapshot;

    fn snap(content: &str) -> CaptureSnapshot {
        build_snapshot(
            SessionContext {
                bundle_id: "com.test.app".into(),
                app_name: "Test".into(),
                pid: 42,
                window_title: "Win".into(),
            },
            content.into(),
            CaptureSource::Merged,
        )
    }

    #[test]
    fn debounce_waits_for_idle() {
        let mut engine = DebounceEngine::new(100);
        engine.observe(snap("hello world prompt"));
        assert!(engine.poll_ready().is_none());
        std::thread::sleep(Duration::from_millis(120));
        assert!(engine.poll_ready().is_some());
        assert!(engine.poll_ready().is_none());
    }

    #[test]
    fn debounce_dedupes_identical_content() {
        let mut engine = DebounceEngine::new(10);
        engine.observe(snap("duplicate content here"));
        std::thread::sleep(Duration::from_millis(15));
        assert!(engine.poll_ready().is_some());
        engine.observe(snap("duplicate content here"));
        std::thread::sleep(Duration::from_millis(15));
        assert!(engine.poll_ready().is_none());
    }

    #[test]
    fn debounce_resets_on_content_change() {
        let mut engine = DebounceEngine::new(50);
        engine.observe(snap("first prompt text"));
        std::thread::sleep(Duration::from_millis(20));
        engine.observe(snap("first prompt text updated"));
        assert!(engine.poll_ready().is_none());
        std::thread::sleep(Duration::from_millis(60));
        let ready = engine.poll_ready().unwrap();
        assert!(ready.content.contains("updated"));
    }
}

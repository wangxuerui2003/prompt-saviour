use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    Accessibility,
    Keystroke,
    Clipboard,
    Merged,
}

impl CaptureSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accessibility => "accessibility",
            Self::Keystroke => "keystroke",
            Self::Clipboard => "clipboard",
            Self::Merged => "merged",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub bundle_id: String,
    pub app_name: String,
    pub pid: u32,
    pub window_title: String,
}

impl SessionContext {
    pub fn slot_key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.bundle_id.as_bytes());
        hasher.update(b":");
        hasher.update(self.pid.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(self.window_title.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftSlot {
    pub key: String,
    pub context: SessionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRecord {
    pub id: i64,
    pub slot_key: String,
    pub content: String,
    pub source: CaptureSource,
    pub app_name: String,
    pub bundle_id: String,
    pub window_title: String,
    pub char_count: usize,
    pub updated_at: DateTime<Utc>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSnapshot {
    pub slot: DraftSlot,
    pub content: String,
    pub source: CaptureSource,
    pub captured_at: DateTime<Utc>,
}

impl CaptureSnapshot {
    pub fn is_meaningful(&self, min_chars: usize) -> bool {
        self.content.trim().len() >= min_chars
    }
}

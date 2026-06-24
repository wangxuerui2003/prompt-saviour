use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::{debug, info};

use crate::config::AppConfig;
use crate::draft::{CaptureSnapshot, CaptureSource, DraftRecord};

const MIN_DRAFT_CHARS: usize = 8;

pub struct DraftStore {
    conn: Connection,
    retention_days: u32,
    max_drafts: u32,
}

impl DraftStore {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Self::init_schema(&conn        )?;
        Self::migrate_schema(&conn)?;
        let config = AppConfig::load().unwrap_or_default();
        Ok(Self {
            conn,
            retention_days: config.retention_days,
            max_drafts: config.max_drafts,
        })
    }

    fn init_schema(conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS drafts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slot_key TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT NOT NULL,
                app_name TEXT NOT NULL,
                bundle_id TEXT NOT NULL,
                window_title TEXT NOT NULL,
                char_count INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_drafts_slot ON drafts(slot_key);
            CREATE INDEX IF NOT EXISTS idx_drafts_updated ON drafts(updated_at DESC);
            ",
        )?;
        Ok(())
    }

    fn migrate_schema(conn: &Connection) -> anyhow::Result<()> {
        let has_pinned: bool = conn
            .prepare("SELECT pinned FROM drafts LIMIT 0")
            .is_ok();
        if !has_pinned {
            conn.execute(
                "ALTER TABLE drafts ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        Ok(())
    }

    /// Persist snapshot. Returns true if a new row was written.
    pub fn upsert_snapshot(&self, snapshot: &CaptureSnapshot) -> anyhow::Result<bool> {
        let content = snapshot.content.trim();
        if content.chars().count() < MIN_DRAFT_CHARS {
            return Ok(false);
        }

        if self.latest_content_for_slot(&snapshot.slot.key)? == Some(content.to_string()) {
            return Ok(false);
        }

        let updated_at = snapshot.captured_at.to_rfc3339();
        let source = snapshot.source.as_str();
        let ctx = &snapshot.slot.context;

        self.conn.execute(
            "INSERT INTO drafts (slot_key, content, source, app_name, bundle_id, window_title, char_count, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot.slot.key,
                content,
                source,
                ctx.app_name,
                ctx.bundle_id,
                ctx.window_title,
                content.chars().count(),
                updated_at,
            ],
        )?;

        self.conn.execute(
            "DELETE FROM drafts
             WHERE slot_key = ?1
               AND id NOT IN (
                 SELECT id FROM drafts
                 WHERE slot_key = ?1
                 ORDER BY updated_at DESC
                 LIMIT 5
               )",
            params![snapshot.slot.key],
        )?;

        self.prune_old()?;
        debug!(
            app = %ctx.app_name,
            chars = content.chars().count(),
            source,
            "draft saved"
        );
        Ok(true)
    }

    fn latest_content_for_slot(&self, slot_key: &str) -> anyhow::Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM drafts WHERE slot_key = ?1 ORDER BY updated_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![slot_key], |row| row.get(0))?;
        Ok(rows.next().transpose()?)
    }

    fn prune_old(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "DELETE FROM drafts
             WHERE pinned = 0
               AND updated_at < datetime('now', ?1)",
            params![format!("-{} days", self.retention_days)],
        )?;
        self.conn.execute(
            "DELETE FROM drafts
             WHERE pinned = 0
               AND id NOT IN (
               SELECT id FROM drafts
               ORDER BY updated_at DESC
               LIMIT ?1
             )",
            params![self.max_drafts],
        )?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> anyhow::Result<Vec<DraftRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, slot_key, content, source, app_name, bundle_id, window_title, char_count, updated_at, pinned
             FROM drafts
             ORDER BY updated_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_record)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<DraftRecord>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, slot_key, content, source, app_name, bundle_id, window_title, char_count, updated_at, pinned
             FROM drafts
             WHERE content LIKE ?1 OR app_name LIKE ?1 OR bundle_id LIKE ?1
             ORDER BY updated_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64], row_to_record)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete_by_id(&self, id: i64) -> anyhow::Result<bool> {
        let changed = self
            .conn
            .execute("DELETE FROM drafts WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    pub fn delete_all(&self) -> anyhow::Result<usize> {
        let changed = self.conn.execute("DELETE FROM drafts WHERE pinned = 0", [])?;
        Ok(changed)
    }

    pub fn set_pinned(&self, id: i64, pinned: bool) -> anyhow::Result<bool> {
        let changed = self.conn.execute(
            "UPDATE drafts SET pinned = ?1 WHERE id = ?2",
            params![pinned as i32, id],
        )?;
        Ok(changed > 0)
    }

    pub fn db_size_bytes(&self) -> anyhow::Result<u64> {
        let path = self.db_path();
        Ok(std::fs::metadata(path)?.len())
    }

    pub fn get_latest(&self) -> anyhow::Result<Option<DraftRecord>> {
        Ok(self.list_recent(1)?.into_iter().next())
    }

    pub fn get_by_id(&self, id: i64) -> anyhow::Result<Option<DraftRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, slot_key, content, source, app_name, bundle_id, window_title, char_count, updated_at, pinned
             FROM drafts
             WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_record)?;
        Ok(rows.next().transpose()?)
    }

    pub fn db_path(&self) -> PathBuf {
        self.conn
            .path()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("drafts.db"))
    }

    pub fn count(&self) -> anyhow::Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM drafts", [], |row| row.get(0))?;
        Ok(count)
    }
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<DraftRecord> {
    let source_raw: String = row.get(3)?;
    let source = match source_raw.as_str() {
        "accessibility" => CaptureSource::Accessibility,
        "keystroke" => CaptureSource::Keystroke,
        "clipboard" => CaptureSource::Clipboard,
        _ => CaptureSource::Merged,
    };
    let updated_at_raw: String = row.get(8)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_raw)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let pinned: i32 = row.get(9)?;

    Ok(DraftRecord {
        id: row.get(0)?,
        slot_key: row.get(1)?,
        content: row.get(2)?,
        source,
        app_name: row.get(4)?,
        bundle_id: row.get(5)?,
        window_title: row.get(6)?,
        char_count: row.get(7)?,
        updated_at,
        pinned: pinned != 0,
    })
}

pub fn open_default_store() -> anyhow::Result<DraftStore> {
    let path = crate::config::default_db_path()?;
    info!(path = %path.display(), "opening draft store");
    DraftStore::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft::{CaptureSource, SessionContext};
    use crate::merge::build_snapshot;

    fn snap(content: &str) -> CaptureSnapshot {
        build_snapshot(
            SessionContext {
                bundle_id: "com.test".into(),
                app_name: "Test".into(),
                pid: 1,
                window_title: "W".into(),
            },
            content.into(),
            CaptureSource::Merged,
        )
    }

    #[test]
    fn skips_duplicate_content_for_slot() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("test.db");
        let store = DraftStore::open(&db).unwrap();
        assert!(store.upsert_snapshot(&snap("hello duplicate")).unwrap());
        assert!(!store.upsert_snapshot(&snap("hello duplicate")).unwrap());
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn stores_content_updates() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("test.db");
        let store = DraftStore::open(&db).unwrap();
        assert!(store.upsert_snapshot(&snap("version one xx")).unwrap());
        assert!(store.upsert_snapshot(&snap("version two xx")).unwrap());
        assert_eq!(store.count().unwrap(), 2);
    }
}

use crate::draft::{CaptureSnapshot, CaptureSource, DraftSlot, SessionContext};

const MIN_DRAFT_CHARS: usize = 8;

pub struct MergeEngine;

impl MergeEngine {
    pub fn merge(
        ax: Option<&CaptureSnapshot>,
        keystroke: Option<&CaptureSnapshot>,
        clipboard: Option<&CaptureSnapshot>,
    ) -> Option<CaptureSnapshot> {
        let mut candidates: Vec<&CaptureSnapshot> = Vec::new();
        if let Some(s) = ax {
            candidates.push(s);
        }
        if let Some(s) = keystroke {
            candidates.push(s);
        }
        if let Some(s) = clipboard {
            candidates.push(s);
        }

        if candidates.is_empty() {
            return None;
        }

        let slot = pick_slot(&candidates)?;
        let content = pick_content(&slot.context, ax, keystroke, clipboard)?;
        if content.trim().chars().count() < MIN_DRAFT_CHARS {
            return None;
        }

        Some(CaptureSnapshot {
            slot,
            content,
            source: CaptureSource::Merged,
            captured_at: chrono::Utc::now(),
        })
    }
}

fn pick_slot(candidates: &[&CaptureSnapshot]) -> Option<DraftSlot> {
    candidates
        .iter()
        .max_by_key(|s| s.captured_at)
        .map(|s| s.slot.clone())
}

fn pick_content(
    context: &SessionContext,
    ax: Option<&CaptureSnapshot>,
    keystroke: Option<&CaptureSnapshot>,
    clipboard: Option<&CaptureSnapshot>,
) -> Option<String> {
    if is_terminal_session(context) {
        return pick_longest([keystroke, ax, clipboard]);
    }

    // GUI: prefer AX snapshot when available
    if let Some(ax_snap) = ax {
        if ax_snap.content.trim().chars().count() >= MIN_DRAFT_CHARS {
            return Some(ax_snap.content.clone());
        }
    }

    pick_longest([ax, keystroke, clipboard])
}

fn pick_longest(sources: [Option<&CaptureSnapshot>; 3]) -> Option<String> {
    let mut best: Option<String> = None;
    for snap in sources.into_iter().flatten() {
        let len = snap.content.chars().count();
        match &best {
            None => best = Some(snap.content.clone()),
            Some(current) if len > current.chars().count() => best = Some(snap.content.clone()),
            _ => {}
        }
    }
    best
}

pub fn is_terminal_session(ctx: &SessionContext) -> bool {
    is_terminal_bundle(&ctx.bundle_id) || is_terminal_app_name(&ctx.app_name)
}

fn is_terminal_bundle(bundle_id: &str) -> bool {
    matches!(
        bundle_id,
        "com.apple.Terminal"
            | "com.googlecode.iterm2"
            | "io.wez.wezterm"
            | "com.github.wez.wezterm"
            | "org.alacritty"
            | "dev.warp.Warp-Stable"
            | "net.kovidgoyal.kitty"
    ) || bundle_id.contains("terminal")
}

fn is_terminal_app_name(app_name: &str) -> bool {
    matches!(
        app_name,
        "Terminal" | "iTerm2" | "WezTerm" | "Warp" | "Alacritty" | "kitty"
    ) || app_name.to_ascii_lowercase().contains("terminal")
}

pub fn build_snapshot(
    context: SessionContext,
    content: String,
    source: CaptureSource,
) -> CaptureSnapshot {
    let key = context.slot_key();
    CaptureSnapshot {
        slot: DraftSlot { key, context },
        content,
        source,
        captured_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(bundle: &str, app: &str) -> SessionContext {
        SessionContext {
            bundle_id: bundle.to_string(),
            app_name: app.to_string(),
            pid: 1,
            window_title: "Window".into(),
        }
    }

    fn snap_with(ctx: SessionContext, content: &str, source: CaptureSource) -> CaptureSnapshot {
        build_snapshot(ctx, content.to_string(), source)
    }

    fn snap(bundle: &str, content: &str, source: CaptureSource) -> CaptureSnapshot {
        snap_with(ctx(bundle, "App"), content, source)
    }

    #[test]
    fn gui_prefers_accessibility_snapshot() {
        let ax = snap(
            "com.openai.codex",
            "full prompt from textarea with edits",
            CaptureSource::Accessibility,
        );
        let ks = snap(
            "com.openai.codex",
            "partial keystroke buffer",
            CaptureSource::Keystroke,
        );
        let merged = MergeEngine::merge(Some(&ax), Some(&ks), None).unwrap();
        assert!(merged.content.contains("full prompt"));
    }

    #[test]
    fn terminal_prefers_longest_buffer() {
        let ks = snap_with(
            ctx("com.googlecode.iterm2", "iTerm2"),
            "long terminal prompt typed in cli",
            CaptureSource::Keystroke,
        );
        let ax = snap_with(
            ctx("com.googlecode.iterm2", "iTerm2"),
            "short",
            CaptureSource::Accessibility,
        );
        let merged = MergeEngine::merge(Some(&ax), Some(&ks), None).unwrap();
        assert!(merged.content.contains("long terminal"));
    }

    #[test]
    fn terminal_detected_by_app_name_when_bundle_is_pid() {
        assert!(is_terminal_session(&ctx("pid:1234", "iTerm2")));
    }
}

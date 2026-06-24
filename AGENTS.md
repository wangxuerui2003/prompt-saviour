# AGENTS.md - Prompt Saviour

Instructions for AI coding agents working in this repository.

## Project summary

**Prompt Saviour** is a local-only tool that silently backs up in-progress coding agent prompts (GUI chat + Terminal CLI) and lets users recover them after a harness crash.

- **Current delivery**: Rust CLI + Tauri 2 GUI on **macOS / Windows / Linux**
- **Target delivery**: Signed installers + polish on all platforms
- **Data**: `~/.prompt-saviour/` (SQLite WAL), overridable via `PROMPT_SAVIOUR_HOME`

Read [docs/README.md](docs/README.md) before large changes.

## Repository layout

```
crates/ps-core/      Platform-agnostic: draft, merge, debounce, storage, inject, config
crates/ps-input/     Shared rdev keystroke listener (Windows + Linux)
crates/ps-macos/     macOS AX polling, keystroke buffer, permissions, context
crates/ps-windows/   Windows UIA + ps-input keystroke
crates/ps-linux/     Linux AT-SPI + ps-input keystroke
crates/ps-daemon/    CLI binary entrypoint (prompt-saviour) + CaptureDaemon lib
crates/ps-gui/       Tauri 2 App - tray, Current Prompt, permissions, history, settings
docs/                Product, architecture, permissions, CLI, roadmap
scripts/e2e.sh       Cross-platform E2E: tests + smoke + GUI launch + inject
scripts/e2e_macos.sh Optional macOS live TextEdit capture
scripts/e2e_windows.sh Optional Windows live Notepad capture
scripts/e2e_linux.sh   Optional Linux live gedit capture
```

## Architecture constraints

1. **Dual-track capture**: AX/UIA for GUI apps; keystroke pass-through for Terminal / fallback
2. **Never replace IME** - observe only, pass-through hooks
3. **Merge in ps-core** - GUI prefers AX snapshot; terminal prefers longest keystroke buffer
4. **DebounceEngine** - reset timer only when slot+content fingerprint changes
5. **Local-only** - no network calls; personal tool, coverage over privacy theater
6. **Cross-platform core** - new OS code goes in `ps-macos` / `ps-windows` / `ps-linux`, not duplicated in GUI

## GUI product requirements (must implement in App)

All of the following belong in the **GUI**, not only in docs or CLI:

| Area | Requirement |
|------|-------------|
| **Current Prompt** | Live full text, char count, source app, capture source, last updated |
| **Permissions** | Per-OS status cards, deep link to Settings, copy binary/.app path, refresh |
| **History** | Same data as `list` / `recover` |
| **Dashboard** | Protection on/off, foreground app, prompt preview |
| **Settings** | Maps to `config.json` |
| **Crash recovery** | Toast when watched agent process exits |

Full spec: [docs/product/gui-app.md](docs/product/gui-app.md)

## Permissions (all platforms)

Users must grant capture permissions to the **exact binary or .app path** (release vs debug differ).

| OS | Guide | CLI check |
|----|-------|-----------|
| macOS | [docs/permissions/macos.md](docs/permissions/macos.md) | `prompt-saviour doctor` |
| Windows | [docs/permissions/windows.md](docs/permissions/windows.md) | `prompt-saviour doctor` |
| Linux | [docs/permissions/linux.md](docs/permissions/linux.md) | `prompt-saviour doctor` |

No-permission self-test: `prompt-saviour smoke`

## Commands (current CLI)

```bash
cargo build --release
prompt-saviour run          # capture daemon (all platforms)
prompt-saviour list
prompt-saviour recover [id]
prompt-saviour doctor
prompt-saviour status
prompt-saviour smoke [--text "..."]
prompt-saviour inject "text" [--app] [--bundle]
bash scripts/e2e.sh
```

GUI dev:

```bash
cd crates/ps-gui && npm install && npm run tauri dev
```

Reference: [docs/cli/reference.md](docs/cli/reference.md)

## Testing expectations

Before claiming a bug fixed:

1. Reproduce via E2E or smoke closest to user experience
2. Run `cargo test`
3. Run `bash scripts/e2e_macos.sh` on macOS when touching capture/storage

Details: [docs/development/testing.md](docs/development/testing.md)

## Coding conventions

- Rust 2021, minimize scope, match existing crate patterns
- User-facing **app strings** (GUI): English + zh-CN later; agent replies to user in **Simplified Chinese** per user preference
- Code identifiers and APIs stay English
- Do not auto-edit CHANGELOG if auto-generated
- Do not commit unless user asks
- Long Markdown: one sentence per line (project doc style)

## Common pitfalls (already fixed once)

- Debounce resetting every loop iteration - use `DebounceEngine`, fingerprint slot+content
- Storage INSERT on identical content - check latest row per slot
- Continuous clipboard watcher - removed; Cmd+V path only via keystroke handler
- Terminal detection via `pid:` bundle - also check `app_name` (iTerm2, Terminal, etc.)
- `inject` CLI: text is positional arg, not `--text`

## When implementing GUI (Tauri)

1. Reuse `ps-core` + platform crates; do not reimplement merge/storage in TypeScript
2. Push `CaptureSnapshot` to frontend via Tauri events for Current Prompt page
3. Permissions page calls same Rust APIs as `doctor` + show install path
4. Single data dir shared with CLI
5. Tray icon: pause/resume, preview, open window

## Roadmap priority

See [docs/roadmap.md](docs/roadmap.md).
Phase 1: Tauri shell + Current Prompt + Permissions + History on macOS.

## Links

- [Features index](docs/features/index.md)
- [Capture pipeline](docs/architecture/capture-pipeline.md)
- [Storage](docs/data/storage.md)

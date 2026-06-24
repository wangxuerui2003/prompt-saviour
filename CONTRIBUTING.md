# Contributing

Thanks for your interest in Prompt Saviour.

## Before you start

1. Search [existing issues](https://github.com/wangxuerui2003/prompt-saviour/issues) for duplicates.
2. For large changes, open an issue first to discuss approach.
3. Read [AGENTS.md](AGENTS.md) if you use AI coding tools in this repo.

## Development setup

```bash
cargo build --release
cd crates/ps-gui && npm install
npm run tauri dev
```

Platform-specific Tauri dependencies:

- **macOS**: Xcode Command Line Tools
- **Linux**: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`
- **Windows**: WebView2 (pre-installed on recent Windows / CI runners)

## Pull requests

1. Fork and create a feature branch from `main`.
2. Keep changes focused - one logical fix or feature per PR.
3. Run tests locally:

   ```bash
   cargo test --workspace
   bash scripts/e2e.sh
   ```

4. Update docs when behavior or CLI flags change.
5. Write a clear PR description: problem, solution, test plan.

## Commit messages

Use imperative mood and explain **why**, not only what:

```
fix debounce reset when slot fingerprint unchanged

The timer was restarting every poll loop even when content was identical.
```

## Releases

Maintainers cut releases by pushing a semver tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The [release workflow](.github/workflows/release.yml) builds GUI installers and CLI binaries for macOS (arm64 + x64), Windows, and Linux, then publishes a GitHub Release.

## CI notes (GitHub Free vs Pro)

This is a **public** repository.
GitHub Actions minutes on standard hosted runners are **unlimited** for public repos on both Free and Pro accounts.

If you fork as a **private** repo, note that macOS runners consume minutes at 10x and Windows at 2x on Free/Pro plans.

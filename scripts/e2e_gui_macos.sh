#!/usr/bin/env bash
# Cross-platform E2E entrypoint (macOS / Linux / Windows via Git Bash)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$ROOT/scripts/e2e.sh"

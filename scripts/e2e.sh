#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLI_BIN="${CLI_BIN:-$ROOT/target/release/prompt-saviour}"
GUI_BIN="${GUI_BIN:-$ROOT/target/debug/prompt-saviour-gui}"
E2E_HOME="$(mktemp -d /tmp/prompt-saviour-e2e.XXXXXX)"
OS_NAME="$(uname -s)"
DAEMON_OK=0
GUI_START_OK=0

cleanup() {
  if [[ -n "${DAEMON_PID:-}" ]] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill -INT "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  if [[ -n "${GUI_PID:-}" ]] && kill -0 "$GUI_PID" 2>/dev/null; then
    kill -TERM "$GUI_PID" 2>/dev/null || true
    for _ in $(seq 1 20); do
      kill -0 "$GUI_PID" 2>/dev/null || break
      sleep 0.25
    done
    kill -9 "$GUI_PID" 2>/dev/null || true
    wait "$GUI_PID" 2>/dev/null || true
  fi
  rm -rf "$E2E_HOME"
}
trap cleanup EXIT

echo "==> Building workspace (release CLI + debug GUI)"
cargo build --release --manifest-path "$ROOT/Cargo.toml" --quiet
cargo build -p prompt-saviour-gui --manifest-path "$ROOT/Cargo.toml" --quiet
cd "$ROOT/crates/ps-gui" && npm run build --silent

echo "==> E2E home: $E2E_HOME (OS: $OS_NAME)"
export PROMPT_SAVIOUR_HOME="$E2E_HOME"

echo "==> Rust integration tests"
cargo test -p prompt-saviour-gui --test gui_e2e --quiet
cargo test -p ps-daemon --test pipeline_e2e --quiet
cargo test --workspace --quiet
echo "PASS: rust tests"

echo "==> CLI smoke"
PROMPT_TEXT="Cross-platform E2E smoke $(date +%s)"
"$CLI_BIN" smoke --text "$PROMPT_TEXT"
"$CLI_BIN" list | grep -q "Cross-platform E2E smoke"
echo "PASS: cli smoke"

echo "==> GUI launch smoke"
if [[ -x "$GUI_BIN" ]]; then
  "$GUI_BIN" &
  GUI_PID=$!
  sleep 2
  if kill -0 "$GUI_PID" 2>/dev/null; then
    GUI_START_OK=1
    echo "PASS: gui process stayed alive"
  else
    echo "FAIL: gui process exited early"
    exit 1
  fi
  kill -TERM "$GUI_PID" 2>/dev/null || true
  for _ in $(seq 1 20); do
    kill -0 "$GUI_PID" 2>/dev/null || break
    sleep 0.25
  done
  kill -9 "$GUI_PID" 2>/dev/null || true
  wait "$GUI_PID" 2>/dev/null || true
  unset GUI_PID
else
  echo "SKIP: GUI binary not found"
fi

echo "==> Daemon inject channel"
"$CLI_BIN" run &
DAEMON_PID=$!
sleep 0.8
INJECT_TEXT="Cross-platform inject test $(date +%s)"
"$CLI_BIN" inject "$INJECT_TEXT" --app TestApp --bundle com.test.app
sleep 1.2
if "$CLI_BIN" list | grep -q "Cross-platform inject test"; then
  DAEMON_OK=1
  echo "PASS: daemon inject"
else
  echo "FAIL: inject did not appear"
  exit 1
fi
kill -INT "$DAEMON_PID" 2>/dev/null || true
wait "$DAEMON_PID" 2>/dev/null || true
unset DAEMON_PID

if [[ "$OS_NAME" == "Darwin" ]]; then
  echo "==> Optional macOS live capture"
  bash "$ROOT/scripts/e2e_macos.sh" || true
elif [[ "$OS_NAME" == "Linux" ]]; then
  echo "==> Optional Linux live capture"
  bash "$ROOT/scripts/e2e_linux.sh" || true
elif [[ "$OS_NAME" == MINGW* ]] || [[ "$OS_NAME" == MSYS* ]] || [[ "$OS_NAME" == CYGWIN* ]] || [[ "${OS:-}" == "Windows_NT" ]]; then
  echo "==> Optional Windows live capture"
  bash "$ROOT/scripts/e2e_windows.sh" || true
fi

echo "==> E2E summary: os=$OS_NAME rust=pass gui_start=${GUI_START_OK} daemon_inject=${DAEMON_OK}"

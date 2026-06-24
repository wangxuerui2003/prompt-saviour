#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/prompt-saviour.exe}"
if [[ ! -x "$BIN" ]]; then
  BIN="${BIN:-$ROOT/target/release/prompt-saviour}"
fi
E2E_HOME="$(mktemp -d "${TMPDIR:-/tmp}/prompt-saviour-e2e.XXXXXX")"
PROMPT_TEXT="E2E capture test from Notepad $(date +%s)"
LIVE_OK=0
INJECT_OK=0

cleanup() {
  if [[ -n "${DAEMON_PID:-}" ]] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill -INT "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  rm -rf "$E2E_HOME"
}
trap cleanup EXIT

echo "==> Building release binary"
cargo build --release --manifest-path "$ROOT/Cargo.toml" --quiet

echo "==> E2E home: $E2E_HOME"
export PROMPT_SAVIOUR_HOME="$E2E_HOME"

echo "==> Smoke test (no permissions required)"
"$BIN" smoke --text "$PROMPT_TEXT"
LIST_OUT="$("$BIN" list)"
echo "$LIST_OUT"
echo "$LIST_OUT" | grep -q "E2E capture test from Notepad"
"$BIN" recover >/dev/null
echo "PASS: smoke pipeline"

echo "==> Daemon inject test"
"$BIN" run &
DAEMON_PID=$!
sleep 1
INJECT_TEXT="Inject channel test prompt $(date +%s)"
"$BIN" inject "$INJECT_TEXT" --app Notepad --bundle com.microsoft.notepad
sleep 1.5
INJECT_LIST="$("$BIN" list)"
echo "$INJECT_LIST"
echo "$INJECT_LIST" | grep -q "Inject channel test prompt"
INJECT_OK=1
echo "PASS: daemon inject channel"
kill -INT "$DAEMON_PID" 2>/dev/null || taskkill //PID "$DAEMON_PID" //F 2>/dev/null || true
wait "$DAEMON_PID" 2>/dev/null || true
unset DAEMON_PID

echo "==> Optional live Notepad capture (needs UI Automation + input hook)"
PERMS="$("$BIN" doctor 2>&1 || true)"
if echo "$PERMS" | grep -q "granted"; then
  "$BIN" run &
  DAEMON_PID=$!
  sleep 1
  if command -v powershell.exe >/dev/null 2>&1; then
    powershell.exe -NoProfile -Command "
      Add-Type -AssemblyName System.Windows.Forms
      Start-Process notepad.exe
      Start-Sleep -Milliseconds 800
      [System.Windows.Forms.SendKeys]::SendWait('${PROMPT_TEXT} live')
    " && sleep 2
    LIVE_LIST="$("$BIN" list)"
    echo "$LIVE_LIST"
    if echo "$LIVE_LIST" | grep -q "E2E capture test from Notepad"; then
      LIVE_OK=1
      echo "PASS: live Notepad capture"
    else
      echo "WARN: live capture did not appear in draft list"
    fi
  else
    echo "SKIP: powershell.exe not available for live capture"
  fi
  kill -INT "$DAEMON_PID" 2>/dev/null || taskkill //PID "$DAEMON_PID" //F 2>/dev/null || true
  wait "$DAEMON_PID" 2>/dev/null || true
else
  echo "SKIP: prompt-saviour permissions not granted for live capture"
fi

echo "==> E2E summary: smoke=pass inject=${INJECT_OK} live=${LIVE_OK}"

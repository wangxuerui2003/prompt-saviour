#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/prompt-saviour}"
E2E_HOME="$(mktemp -d /tmp/prompt-saviour-e2e.XXXXXX)"
PROMPT_TEXT="E2E capture test prompt from TextEdit $(date +%s)"
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
echo "$LIST_OUT" | grep -q "E2E capture test prompt"
"$BIN" recover >/dev/null
echo "PASS: smoke pipeline"

echo "==> Daemon inject test"
"$BIN" run &
DAEMON_PID=$!
sleep 0.5
INJECT_TEXT="Inject channel test prompt $(date +%s)"
"$BIN" inject "$INJECT_TEXT" --app TextEdit --bundle com.apple.TextEdit
sleep 1
INJECT_LIST="$("$BIN" list)"
echo "$INJECT_LIST"
echo "$INJECT_LIST" | grep -q "Inject channel test prompt"
INJECT_OK=1
echo "PASS: daemon inject channel"
kill -INT "$DAEMON_PID"
wait "$DAEMON_PID" 2>/dev/null || true
unset DAEMON_PID

echo "==> Optional live TextEdit capture (needs Accessibility + Input Monitoring)"
PERMS="$("$BIN" doctor 2>&1 || true)"
if echo "$PERMS" | grep -q "granted"; then
  "$BIN" run &
  DAEMON_PID=$!
  sleep 0.5
  if osascript <<EOF
tell application "TextEdit"
  activate
  make new document
  delay 0.3
end tell
tell application "System Events"
  keystroke "$PROMPT_TEXT live"
end tell
EOF
  then
    sleep 2
    LIVE_LIST="$("$BIN" list)"
    echo "$LIVE_LIST"
    if echo "$LIVE_LIST" | grep -q "E2E capture test prompt"; then
      LIVE_OK=1
      echo "PASS: live TextEdit capture"
    else
      echo "WARN: live capture did not appear in draft list"
    fi
  else
    echo "SKIP: osascript keystroke blocked (grant Automation/Accessibility to osascript or Terminal)"
  fi
  kill -INT "$DAEMON_PID" 2>/dev/null || true
  wait "$DAEMON_PID" 2>/dev/null || true
else
  echo "SKIP: prompt-saviour permissions not granted for live capture"
fi

echo "==> E2E summary: smoke=pass inject=pass live=${LIVE_OK}"

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/prompt-saviour}"
E2E_HOME="$(mktemp -d /tmp/prompt-saviour-e2e.XXXXXX)"
PROMPT_TEXT="E2E capture test from gedit $(date +%s)"
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
echo "$LIST_OUT" | grep -q "E2E capture test from gedit"
"$BIN" recover >/dev/null
echo "PASS: smoke pipeline"

echo "==> Daemon inject test"
"$BIN" run &
DAEMON_PID=$!
sleep 0.8
INJECT_TEXT="Inject channel test prompt $(date +%s)"
"$BIN" inject "$INJECT_TEXT" --app gedit --bundle org.gnome.gedit
sleep 1.2
INJECT_LIST="$("$BIN" list)"
echo "$INJECT_LIST"
echo "$INJECT_LIST" | grep -q "Inject channel test prompt"
INJECT_OK=1
echo "PASS: daemon inject channel"
kill -INT "$DAEMON_PID" 2>/dev/null || true
wait "$DAEMON_PID" 2>/dev/null || true
unset DAEMON_PID

echo "==> Optional live gedit capture (needs AT-SPI session + input group on Wayland)"
PERMS="$("$BIN" doctor 2>&1 || true)"
if echo "$PERMS" | grep -q "granted"; then
  if command -v gedit >/dev/null 2>&1 && command -v xdotool >/dev/null 2>&1; then
    "$BIN" run &
    DAEMON_PID=$!
    sleep 0.8
    gedit --new-document >/dev/null 2>&1 &
    sleep 0.8
    xdotool type --delay 12 "${PROMPT_TEXT} live" || true
    sleep 2
    LIVE_LIST="$("$BIN" list)"
    echo "$LIVE_LIST"
    if echo "$LIVE_LIST" | grep -q "E2E capture test from gedit"; then
      LIVE_OK=1
      echo "PASS: live gedit capture"
    else
      echo "WARN: live capture did not appear in draft list"
    fi
    kill -INT "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
    pkill -x gedit 2>/dev/null || true
  else
    echo "SKIP: gedit or xdotool not installed for live capture"
  fi
else
  echo "SKIP: prompt-saviour permissions not granted for live capture"
fi

echo "==> E2E summary: smoke=pass inject=${INJECT_OK} live=${LIVE_OK}"

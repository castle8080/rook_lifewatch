#!/bin/sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DAEMON="$SCRIPT_DIR/rook_lw_daemon"

if [ ! -x "$DAEMON" ]; then
  echo "Error: expected executable '$DAEMON' (same directory as this script)." >&2
  exit 1
fi

# Check if rook_lw_daemon is already running (requested: use ps).
EXISTING_PIDS=$(ps -eo pid=,comm= | awk '$2=="rook_lw_daemon"{print $1}')
if [ -n "${EXISTING_PIDS:-}" ]; then
  echo "Error: rook_lw_daemon is already running (pid(s): ${EXISTING_PIDS})." >&2
  exit 1
fi

VAR_DIR="var"
IMG_DIR="$VAR_DIR/images"
LOG_DIR="$VAR_DIR/logs"

mkdir -p "$IMG_DIR" "$LOG_DIR"

TMP_LOG="$LOG_DIR/rook_lw_daemon_starting_$$.log"
: > "$TMP_LOG"

# Start detached so it survives terminal close.
if command -v setsid >/dev/null 2>&1; then
  setsid "$DAEMON" >>"$TMP_LOG" 2>&1 < /dev/null &
else
  nohup "$DAEMON" >>"$TMP_LOG" 2>&1 < /dev/null &
fi

PID=$!
FINAL_LOG="$LOG_DIR/rook_lw_daemon_${PID}.log"

# Renaming keeps the same underlying file open; output continues into FINAL_LOG.
mv -f "$TMP_LOG" "$FINAL_LOG"

echo "Started rook_lw_daemon (pid $PID). Logging to $FINAL_LOG"
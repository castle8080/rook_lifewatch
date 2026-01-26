#!/bin/sh
set -e

ps -eo pid,comm | egrep 'rook_lw_daemon' | while read pid cmd; do
    echo "Killing: $pid -> $cmd"
    kill $pid
done
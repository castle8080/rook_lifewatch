#!/bin/sh
set -e

ps -eo pid,comm | egrep 'rook_lw_(admin|daemon)' | while read pid cmd; do
    echo "Killing: $pid -> $cmd"
    kill $pid
done
